use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use master_service::{
    domain::{
        contracts::repository::{
            CityRepository, RepositoryFuture,
            city::{self as city_contract, ListArgs},
        },
        models::{AppContext, AppTransaction, entity::City},
    },
    infrastructure::{caching::MokaCache, repository::CachedCity},
};

use super::support::{city, city_create, city_update, pagination};

struct FakeCityRepository {
    city: Mutex<City>,
    listed: Vec<City>,
    find_by_id_calls: AtomicUsize,
    find_by_code_calls: AtomicUsize,
    find_by_coord_calls: AtomicUsize,
    list_calls: AtomicUsize,
}

impl FakeCityRepository {
    fn new() -> Self {
        let city = city();
        Self {
            city: Mutex::new(city.clone()),
            listed: vec![city],
            find_by_id_calls: AtomicUsize::new(0),
            find_by_code_calls: AtomicUsize::new(0),
            find_by_coord_calls: AtomicUsize::new(0),
            list_calls: AtomicUsize::new(0),
        }
    }

    fn find_by_id_calls(&self) -> usize {
        self.find_by_id_calls.load(Ordering::SeqCst)
    }

    fn find_by_code_calls(&self) -> usize {
        self.find_by_code_calls.load(Ordering::SeqCst)
    }

    fn find_by_coord_calls(&self) -> usize {
        self.find_by_coord_calls.load(Ordering::SeqCst)
    }

    fn list_calls(&self) -> usize {
        self.list_calls.load(Ordering::SeqCst)
    }
}

impl CityRepository for FakeCityRepository {
    fn create<'a>(
        &'a self,
        _context: &'a AppContext,
        _input: city_contract::CreateArgs,
    ) -> RepositoryFuture<'a, City> {
        let city = self.city.lock().expect("city lock").clone();
        Box::pin(async move { Ok(city) })
    }

    fn find_by_id<'a>(
        &'a self,
        _context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<City>> {
        self.find_by_id_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let city = self.city.lock().expect("city lock");
            (city.id == id).then(|| city.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        _context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<City>> {
        self.find_by_code_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let city = self.city.lock().expect("city lock");
            (city.kemendagri_code == code).then(|| city.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn find_city_by_coord<'a>(
        &'a self,
        _context: &'a AppContext,
        _longitude: f64,
        _latitude: f64,
    ) -> RepositoryFuture<'a, Option<City>> {
        self.find_by_coord_calls.fetch_add(1, Ordering::SeqCst);
        let city = self.city.lock().expect("city lock").clone();
        Box::pin(async move { Ok(Some(city)) })
    }

    fn list<'a>(
        &'a self,
        _context: &'a AppContext,
        _filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<City>> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        let listed = self.listed.clone();
        Box::pin(async move { Ok(listed) })
    }

    fn update<'a>(
        &'a self,
        _context: &'a AppContext,
        _id: i64,
        changes: city_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, City> {
        let updated = {
            let mut city = self.city.lock().expect("city lock");
            if let Some(name) = changes.name {
                city.name = name;
            }
            city.clone()
        };
        Box::pin(async move { Ok(updated) })
    }

    fn delete<'a>(&'a self, _context: &'a AppContext, _id: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn cached(repository: Arc<FakeCityRepository>) -> CachedCity {
    CachedCity::new(
        repository,
        Arc::new(MokaCache::new(
            64,
            Duration::from_secs(60),
            Duration::from_secs(5),
        )),
    )
}

fn list_args(name: &str) -> ListArgs {
    ListArgs {
        province_id: None,
        kind: None,
        is_active: None,
        name_contains: Some(name.into()),
        include_deleted: false,
        pagination: pagination(10, 0),
    }
}

#[tokio::test]
async fn reads_use_cache_and_transactions_bypass_it() {
    let repository = Arc::new(FakeCityRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 1);

    cached
        .find_by_kemendagri_code(&context, "11.01")
        .await
        .unwrap();
    cached
        .find_by_kemendagri_code(&context, "11.01")
        .await
        .unwrap();
    assert_eq!(repository.find_by_code_calls(), 1);

    cached
        .find_city_by_coord(&context, 106.5, -6.5)
        .await
        .unwrap();
    cached
        .find_city_by_coord(&context, 106.5, -6.5)
        .await
        .unwrap();
    assert_eq!(repository.find_by_coord_calls(), 1);
    cached
        .find_city_by_coord(&context, 107.5, -7.5)
        .await
        .unwrap();
    assert_eq!(repository.find_by_coord_calls(), 2);

    let transaction_context = AppContext {
        transaction: Some(AppTransaction::new(())),
        ..AppContext::default()
    };
    cached.find_by_id(&transaction_context, 1).await.unwrap();
    cached.find_by_id(&transaction_context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 3);
}

#[tokio::test]
async fn list_filters_use_distinct_keys() {
    let repository = Arc::new(FakeCityRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.list(&context, list_args("one")).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    cached.list(&context, list_args("two")).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();

    assert_eq!(repository.list_calls(), 2);
}

#[tokio::test]
async fn writes_invalidate_cached_reads() {
    let repository = Arc::new(FakeCityRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    cached.update(&context, 1, city_update()).await.unwrap();
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.create(&context, city_create()).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.delete(&context, 1).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 3);
}
