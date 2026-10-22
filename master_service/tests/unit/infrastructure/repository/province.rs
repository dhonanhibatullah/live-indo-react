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
            ProvinceRepository, RepositoryFuture,
            province::{self as province_contract, ListArgs},
        },
        models::{AppContext, AppTransaction, entity::Province},
    },
    infrastructure::{caching::MokaCache, repository::CachedProvince},
};

use super::support::{pagination, province, province_create, province_update};

struct FakeProvinceRepository {
    province: Mutex<Province>,
    listed: Vec<Province>,
    find_by_id_calls: AtomicUsize,
    find_by_code_calls: AtomicUsize,
    list_calls: AtomicUsize,
}

impl FakeProvinceRepository {
    fn new() -> Self {
        let province = province();
        Self {
            province: Mutex::new(province.clone()),
            listed: vec![province],
            find_by_id_calls: AtomicUsize::new(0),
            find_by_code_calls: AtomicUsize::new(0),
            list_calls: AtomicUsize::new(0),
        }
    }

    fn find_by_id_calls(&self) -> usize {
        self.find_by_id_calls.load(Ordering::SeqCst)
    }

    fn find_by_code_calls(&self) -> usize {
        self.find_by_code_calls.load(Ordering::SeqCst)
    }

    fn list_calls(&self) -> usize {
        self.list_calls.load(Ordering::SeqCst)
    }
}

impl ProvinceRepository for FakeProvinceRepository {
    fn create<'a>(
        &'a self,
        _context: &'a AppContext,
        _input: province_contract::CreateArgs,
    ) -> RepositoryFuture<'a, Province> {
        let province = self.province.lock().expect("province lock").clone();
        Box::pin(async move { Ok(province) })
    }

    fn find_by_id<'a>(
        &'a self,
        _context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        self.find_by_id_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let province = self.province.lock().expect("province lock");
            (province.id == id).then(|| province.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        _context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<Province>> {
        self.find_by_code_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let province = self.province.lock().expect("province lock");
            (province.kemendagri_code == code).then(|| province.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn list<'a>(
        &'a self,
        _context: &'a AppContext,
        _filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Province>> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        let listed = self.listed.clone();
        Box::pin(async move { Ok(listed) })
    }

    fn update<'a>(
        &'a self,
        _context: &'a AppContext,
        _id: i64,
        changes: province_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, Province> {
        let updated = {
            let mut province = self.province.lock().expect("province lock");
            if let Some(name) = changes.name {
                province.name = name;
            }
            province.clone()
        };
        Box::pin(async move { Ok(updated) })
    }

    fn delete<'a>(&'a self, _context: &'a AppContext, _id: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn cached(repository: Arc<FakeProvinceRepository>) -> CachedProvince {
    CachedProvince::new(
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
        is_active: None,
        name_contains: Some(name.into()),
        include_deleted: false,
        pagination: pagination(10, 0),
    }
}

#[tokio::test]
async fn reads_use_cache_and_transactions_bypass_it() {
    let repository = Arc::new(FakeProvinceRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 1);

    cached
        .find_by_kemendagri_code(&context, "11")
        .await
        .unwrap();
    cached
        .find_by_kemendagri_code(&context, "11")
        .await
        .unwrap();
    assert_eq!(repository.find_by_code_calls(), 1);

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
    let repository = Arc::new(FakeProvinceRepository::new());
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
    let repository = Arc::new(FakeProvinceRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    cached.update(&context, 1, province_update()).await.unwrap();
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.create(&context, province_create()).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.delete(&context, 1).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 3);
}
