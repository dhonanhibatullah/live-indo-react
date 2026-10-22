use std::{any::Any, sync::Arc};

use crate::domain::{
    contracts::{
        caching::{Cache, CacheKind, CacheValue},
        repository::{
            RepositoryFuture,
            topic::{CreateArgs, ListArgs, TopicRepository, UpdateArgs},
        },
    },
    models::{AppContext, entity::Topic},
};

use super::super::shared::cache::CacheNamespace;

/// Cache-aside decorator for topic reads.
pub struct CachedTopic {
    repository: Arc<dyn TopicRepository>,
    cache: Arc<dyn Cache>,
    namespace: CacheNamespace,
}

impl CachedTopic {
    pub fn new(repository: Arc<dyn TopicRepository>, cache: Arc<dyn Cache>) -> Self {
        Self {
            repository,
            cache,
            namespace: CacheNamespace::new("topic"),
        }
    }

    fn cached<T>(&self, context: &AppContext, key: &str) -> Option<T>
    where
        T: Any + Send + Sync + Clone,
    {
        self.cache
            .get(context, key)
            .and_then(|value| value.downcast::<T>())
            .map(|value| value.as_ref().clone())
    }

    fn store<T>(&self, context: &AppContext, key: &str, kind: CacheKind, value: T, generation: u64)
    where
        T: Any + Send + Sync,
    {
        if self.namespace.generation() == generation {
            self.cache.set(context, key, kind, CacheValue::new(value));
        }
    }
}

impl TopicRepository for CachedTopic {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.create(context, input).await
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        if context.transaction.is_some() {
            return self.repository.find_by_id(context, id);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.id_key(generation, id);

        Box::pin(async move {
            if let Some(topic) = self.cached::<Topic>(context, &key) {
                return Ok(Some(topic));
            }

            let topic = self.repository.find_by_id(context, id).await?;
            if let Some(value) = &topic {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(topic)
        })
    }

    fn find_by_slug<'a>(
        &'a self,
        context: &'a AppContext,
        slug: &'a str,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        if context.transaction.is_some() {
            return self.repository.find_by_slug(context, slug);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.text_key(generation, "slug", slug);

        Box::pin(async move {
            if let Some(topic) = self.cached::<Topic>(context, &key) {
                return Ok(Some(topic));
            }

            let topic = self.repository.find_by_slug(context, slug).await?;
            if let Some(value) = &topic {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(topic)
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Topic>> {
        if context.transaction.is_some() {
            return self.repository.list(context, filter);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(generation, "list", &filter);

        Box::pin(async move {
            if let Some(topics) = self.cached::<Vec<Topic>>(context, &key) {
                return Ok(topics);
            }

            let topics = self.repository.list(context, filter).await?;
            self.store(context, &key, CacheKind::List, topics.clone(), generation);

            Ok(topics)
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.update(context, id, changes).await
        })
    }

    fn delete<'a>(&'a self, context: &'a AppContext, id: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.delete(context, id).await
        })
    }
}
