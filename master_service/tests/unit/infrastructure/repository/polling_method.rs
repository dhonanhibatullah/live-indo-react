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
            PollingMethodRepository, RepositoryFuture,
            polling_method::{self as polling_contract, ListArgs},
        },
        models::{AppContext, AppTransaction, entity::PollingMethod},
    },
    infrastructure::{caching::MokaCache, repository::CachedPollingMethod},
};

use super::support::{pagination, polling_method, polling_method_update};

struct FakePollingMethodRepository {
    method: Mutex<PollingMethod>,
    listed: Vec<PollingMethod>,
    find_by_id_calls: AtomicUsize,
    find_by_code_calls: AtomicUsize,
    list_calls: AtomicUsize,
}

impl FakePollingMethodRepository {
    fn new() -> Self {
        let method = polling_method();
        Self {
            method: Mutex::new(method.clone()),
            listed: vec![method],
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

impl PollingMethodRepository for FakePollingMethodRepository {
    fn find_by_id<'a>(
        &'a self,
        _context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        self.find_by_id_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let method = self.method.lock().expect("method lock");
            (method.id == id).then(|| method.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn find_by_code<'a>(
        &'a self,
        _context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        self.find_by_code_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let method = self.method.lock().expect("method lock");
            (method.code == code).then(|| method.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn list<'a>(
        &'a self,
        _context: &'a AppContext,
        _filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<PollingMethod>> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        let listed = self.listed.clone();
        Box::pin(async move { Ok(listed) })
    }

    fn update<'a>(
        &'a self,
        _context: &'a AppContext,
        _id: i64,
        changes: polling_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, PollingMethod> {
        let updated = {
            let mut method = self.method.lock().expect("method lock");
            if let Some(name) = changes.name {
                method.name = name;
            }
            method.clone()
        };
        Box::pin(async move { Ok(updated) })
    }

    fn delete<'a>(&'a self, _context: &'a AppContext, _id: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn cached(repository: Arc<FakePollingMethodRepository>) -> CachedPollingMethod {
    CachedPollingMethod::new(
        repository,
        Arc::new(MokaCache::new(
            64,
            Duration::from_secs(60),
            Duration::from_secs(5),
        )),
    )
}

fn list_args(is_active: Option<bool>) -> ListArgs {
    ListArgs {
        is_active,
        include_deleted: false,
        pagination: pagination(10, 0),
    }
}

#[tokio::test]
async fn reads_use_cache_and_transactions_bypass_it() {
    let repository = Arc::new(FakePollingMethodRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 1);

    cached
        .find_by_code(&context, "agree_disagree")
        .await
        .unwrap();
    cached
        .find_by_code(&context, "agree_disagree")
        .await
        .unwrap();
    assert_eq!(repository.find_by_code_calls(), 1);

    let transaction_context = AppContext {
        transaction: Some(AppTransaction::new(())),
        ..AppContext::default()
    };
    cached
        .find_by_code(&transaction_context, "agree_disagree")
        .await
        .unwrap();
    cached
        .find_by_code(&transaction_context, "agree_disagree")
        .await
        .unwrap();
    assert_eq!(repository.find_by_code_calls(), 3);
}

#[tokio::test]
async fn list_filters_use_distinct_keys() {
    let repository = Arc::new(FakePollingMethodRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.list(&context, list_args(Some(true))).await.unwrap();
    cached.list(&context, list_args(Some(true))).await.unwrap();
    cached.list(&context, list_args(None)).await.unwrap();
    cached.list(&context, list_args(Some(true))).await.unwrap();

    assert_eq!(repository.list_calls(), 2);
}

#[tokio::test]
async fn writes_invalidate_cached_reads() {
    let repository = Arc::new(FakePollingMethodRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    let updated = cached
        .update(&context, 1, polling_method_update())
        .await
        .unwrap();
    assert_eq!(updated.name, "Updated Method");
    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 2);

    cached.list(&context, list_args(None)).await.unwrap();
    cached.delete(&context, 1).await.unwrap();
    cached.list(&context, list_args(None)).await.unwrap();
    assert_eq!(repository.list_calls(), 2);
}
