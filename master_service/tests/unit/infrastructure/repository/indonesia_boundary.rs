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
            IndonesiaBoundaryRepository, RepositoryFuture, indonesia_boundary::UpdateArgs,
        },
        models::{AppContext, AppTransaction, entity::IndonesiaBoundary},
    },
    infrastructure::{caching::MokaCache, repository::CachedIndonesiaBoundary},
};

use super::support::indonesia_boundary;

struct FakeIndonesiaBoundaryRepository {
    boundary: Mutex<IndonesiaBoundary>,
    get_calls: AtomicUsize,
    update_calls: AtomicUsize,
    inside_calls: AtomicUsize,
}

impl FakeIndonesiaBoundaryRepository {
    fn new() -> Self {
        Self {
            boundary: Mutex::new(indonesia_boundary()),
            get_calls: AtomicUsize::new(0),
            update_calls: AtomicUsize::new(0),
            inside_calls: AtomicUsize::new(0),
        }
    }

    fn get_calls(&self) -> usize {
        self.get_calls.load(Ordering::SeqCst)
    }

    fn inside_calls(&self) -> usize {
        self.inside_calls.load(Ordering::SeqCst)
    }
}

impl IndonesiaBoundaryRepository for FakeIndonesiaBoundaryRepository {
    fn get<'a>(
        &'a self,
        _context: &'a AppContext,
    ) -> RepositoryFuture<'a, Option<IndonesiaBoundary>> {
        self.get_calls.fetch_add(1, Ordering::SeqCst);
        let boundary = self.boundary.lock().expect("boundary lock").clone();
        Box::pin(async move { Ok(Some(boundary)) })
    }

    fn update<'a>(
        &'a self,
        _context: &'a AppContext,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, IndonesiaBoundary> {
        self.update_calls.fetch_add(1, Ordering::SeqCst);
        let updated = {
            let mut boundary = self.boundary.lock().expect("boundary lock");
            if let Some(source) = changes.boundary_source {
                boundary.boundary_source = source;
            }
            if let Some(date) = changes.boundary_reference_date {
                boundary.boundary_reference_date = date;
            }
            boundary.clone()
        };
        Box::pin(async move { Ok(updated) })
    }

    fn is_inside<'a>(
        &'a self,
        _context: &'a AppContext,
        longitude: f64,
        _latitude: f64,
    ) -> RepositoryFuture<'a, bool> {
        self.inside_calls.fetch_add(1, Ordering::SeqCst);
        let inside = (100.0..=120.0).contains(&longitude);
        Box::pin(async move { Ok(inside) })
    }
}

fn cached(repository: Arc<FakeIndonesiaBoundaryRepository>) -> CachedIndonesiaBoundary {
    CachedIndonesiaBoundary::new(
        repository,
        Arc::new(MokaCache::new(
            64,
            Duration::from_secs(60),
            Duration::from_secs(5),
        )),
    )
}

#[tokio::test]
async fn boundary_and_containment_reads_are_cached_and_writes_invalidate() {
    let repository = Arc::new(FakeIndonesiaBoundaryRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.get(&context).await.unwrap();
    cached.get(&context).await.unwrap();
    assert_eq!(repository.get_calls(), 1);

    assert!(cached.is_inside(&context, 110.0, 0.0).await.unwrap());
    assert!(cached.is_inside(&context, 110.0, 0.0).await.unwrap());
    assert_eq!(repository.inside_calls(), 1);
    assert!(!cached.is_inside(&context, 0.0, 0.0).await.unwrap());
    assert!(!cached.is_inside(&context, 0.0, 0.0).await.unwrap());
    assert_eq!(repository.inside_calls(), 2);

    let updated = cached
        .update(
            &context,
            UpdateArgs {
                boundary: None,
                boundary_source: Some("updated source".into()),
                boundary_reference_date: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.boundary_source, "updated source");

    let reloaded = cached
        .get(&context)
        .await
        .unwrap()
        .expect("boundary exists");
    assert_eq!(reloaded.boundary_source, "updated source");
    assert_eq!(repository.get_calls(), 2);
    cached.is_inside(&context, 110.0, 0.0).await.unwrap();
    assert_eq!(repository.inside_calls(), 3);
}

#[tokio::test]
async fn transactional_reads_bypass_the_boundary_cache() {
    let repository = Arc::new(FakeIndonesiaBoundaryRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext {
        transaction: Some(AppTransaction::new(())),
        ..AppContext::default()
    };

    cached.get(&context).await.unwrap();
    cached.get(&context).await.unwrap();
    assert_eq!(repository.get_calls(), 2);

    cached.is_inside(&context, 110.0, 0.0).await.unwrap();
    cached.is_inside(&context, 110.0, 0.0).await.unwrap();
    assert_eq!(repository.inside_calls(), 2);
}
