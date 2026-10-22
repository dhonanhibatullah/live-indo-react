use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

use futures_util::task::noop_waker;
use master_service::{
    domain::{
        contracts::repository::{
            RepositoryFuture, TopicRepository,
            topic::{self as topic_contract, ListArgs},
        },
        models::{AppContext, AppTransaction, RepositoryError, entity::Topic},
    },
    infrastructure::{caching::MokaCache, repository::CachedTopic},
};

use super::support::{pagination, topic, topic_create, topic_update};

struct FakeTopicRepository {
    topic: Mutex<Topic>,
    listed: Vec<Topic>,
    find_by_id_calls: AtomicUsize,
    find_by_slug_calls: AtomicUsize,
    list_calls: AtomicUsize,
    create_calls: AtomicUsize,
    update_calls: AtomicUsize,
    delete_calls: AtomicUsize,
    fail_find_by_id: AtomicBool,
    pending_update: AtomicBool,
}

impl FakeTopicRepository {
    fn new() -> Self {
        let topic = topic();
        Self {
            topic: Mutex::new(topic.clone()),
            listed: vec![topic],
            find_by_id_calls: AtomicUsize::new(0),
            find_by_slug_calls: AtomicUsize::new(0),
            list_calls: AtomicUsize::new(0),
            create_calls: AtomicUsize::new(0),
            update_calls: AtomicUsize::new(0),
            delete_calls: AtomicUsize::new(0),
            fail_find_by_id: AtomicBool::new(false),
            pending_update: AtomicBool::new(false),
        }
    }

    fn find_by_id_calls(&self) -> usize {
        self.find_by_id_calls.load(Ordering::SeqCst)
    }

    fn find_by_slug_calls(&self) -> usize {
        self.find_by_slug_calls.load(Ordering::SeqCst)
    }

    fn list_calls(&self) -> usize {
        self.list_calls.load(Ordering::SeqCst)
    }

    fn fail_next_find_by_id(&self) {
        self.fail_find_by_id.store(true, Ordering::SeqCst);
    }

    fn pend_update(&self) {
        self.pending_update.store(true, Ordering::SeqCst);
    }
}

impl TopicRepository for FakeTopicRepository {
    fn create<'a>(
        &'a self,
        _context: &'a AppContext,
        _input: topic_contract::CreateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        self.create_calls.fetch_add(1, Ordering::SeqCst);
        let topic = self.topic.lock().expect("topic lock").clone();
        Box::pin(async move { Ok(topic) })
    }

    fn find_by_id<'a>(
        &'a self,
        _context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        self.find_by_id_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_find_by_id.swap(false, Ordering::SeqCst) {
            return Box::pin(async { Err(RepositoryError::TopicNotFound) });
        }

        let found = {
            let topic = self.topic.lock().expect("topic lock");
            (topic.id == id).then(|| topic.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn find_by_slug<'a>(
        &'a self,
        _context: &'a AppContext,
        slug: &'a str,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        self.find_by_slug_calls.fetch_add(1, Ordering::SeqCst);
        let found = {
            let topic = self.topic.lock().expect("topic lock");
            (topic.slug == slug).then(|| topic.clone())
        };
        Box::pin(async move { Ok(found) })
    }

    fn list<'a>(
        &'a self,
        _context: &'a AppContext,
        _filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Topic>> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        let listed = self.listed.clone();
        Box::pin(async move { Ok(listed) })
    }

    fn update<'a>(
        &'a self,
        _context: &'a AppContext,
        _id: i64,
        changes: topic_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        self.update_calls.fetch_add(1, Ordering::SeqCst);
        if self.pending_update.load(Ordering::SeqCst) {
            return Box::pin(std::future::pending::<Result<Topic, RepositoryError>>());
        }

        let updated = {
            let mut topic = self.topic.lock().expect("topic lock");
            if let Some(name) = changes.name {
                topic.name = name;
            }
            topic.clone()
        };
        Box::pin(async move { Ok(updated) })
    }

    fn delete<'a>(&'a self, _context: &'a AppContext, _id: i64) -> RepositoryFuture<'a, ()> {
        self.delete_calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

fn cached(repository: Arc<FakeTopicRepository>) -> CachedTopic {
    CachedTopic::new(
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
        polling_method_id: None,
        is_active: None,
        name_contains: Some(name.into()),
        include_deleted: false,
        pagination: pagination(10, 0),
    }
}

#[tokio::test]
async fn reads_use_cache_and_transactions_bypass_it() {
    let repository = Arc::new(FakeTopicRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    assert!(cached.find_by_id(&context, 1).await.unwrap().is_some());
    assert!(cached.find_by_id(&context, 1).await.unwrap().is_some());
    assert_eq!(repository.find_by_id_calls(), 1);

    assert!(
        cached
            .find_by_slug(&context, "topic")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        cached
            .find_by_slug(&context, "topic")
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(repository.find_by_slug_calls(), 1);

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
    let repository = Arc::new(FakeTopicRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.list(&context, list_args("one")).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    cached.list(&context, list_args("two")).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();

    assert_eq!(repository.list_calls(), 2);
}

#[tokio::test]
async fn missing_and_failed_reads_are_not_cached() {
    let repository = Arc::new(FakeTopicRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    assert!(
        cached
            .find_by_id(&context, i64::MAX)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        cached
            .find_by_id(&context, i64::MAX)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(repository.find_by_id_calls(), 2);

    repository.fail_next_find_by_id();
    assert!(matches!(
        cached.find_by_id(&context, 1).await,
        Err(RepositoryError::TopicNotFound)
    ));
    assert!(cached.find_by_id(&context, 1).await.unwrap().is_some());
    assert_eq!(repository.find_by_id_calls(), 4);
}

#[tokio::test]
async fn writes_invalidate_cached_reads() {
    let repository = Arc::new(FakeTopicRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    let updated = cached.update(&context, 1, topic_update()).await.unwrap();
    assert_eq!(updated.name, "Updated Topic");
    let reloaded = cached
        .find_by_id(&context, 1)
        .await
        .unwrap()
        .expect("topic exists");
    assert_eq!(reloaded.name, "Updated Topic");
    assert_eq!(repository.find_by_id_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.create(&context, topic_create()).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 2);

    cached.list(&context, list_args("one")).await.unwrap();
    cached.delete(&context, 1).await.unwrap();
    cached.list(&context, list_args("one")).await.unwrap();
    assert_eq!(repository.list_calls(), 3);
}

#[tokio::test]
async fn canceled_write_invalidates_cache() {
    let repository = Arc::new(FakeTopicRepository::new());
    let cached = cached(Arc::clone(&repository));
    let context = AppContext::default();

    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 1);

    repository.pend_update();
    let mut write = Box::pin(cached.update(&context, 1, topic_update()));
    let waker = noop_waker();
    let mut task_context = Context::from_waker(&waker);
    assert!(matches!(
        write.as_mut().poll(&mut task_context),
        Poll::Pending
    ));
    drop(write);

    cached.find_by_id(&context, 1).await.unwrap();
    assert_eq!(repository.find_by_id_calls(), 2);
}

#[test]
fn decorator_is_object_safe() {
    let repository = Arc::new(FakeTopicRepository::new());
    let _: Arc<dyn TopicRepository> = Arc::new(cached(repository));
}
