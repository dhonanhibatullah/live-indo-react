use std::{any::Any, sync::Arc};

use crate::domain::{
    contracts::{
        caching::{Cache, CacheKind, CacheValue},
        repository::{
            PollingMethodRepository, RepositoryFuture,
            polling_method::{ListArgs, UpdateArgs},
        },
    },
    models::{AppContext, entity::PollingMethod},
};

use super::super::shared::cache::CacheNamespace;

/// Cache-aside decorator for polling-method reads.
pub struct CachedPollingMethod {
    repository: Arc<dyn PollingMethodRepository>,
    cache: Arc<dyn Cache>,
    namespace: CacheNamespace,
}

impl CachedPollingMethod {
    pub fn new(repository: Arc<dyn PollingMethodRepository>, cache: Arc<dyn Cache>) -> Self {
        Self {
            repository,
            cache,
            namespace: CacheNamespace::new("polling_method"),
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

impl PollingMethodRepository for CachedPollingMethod {
    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        if context.transaction.is_some() {
            return self.repository.find_by_id(context, id);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.id_key(generation, id);

        Box::pin(async move {
            if let Some(method) = self.cached::<PollingMethod>(context, &key) {
                return Ok(Some(method));
            }

            let method = self.repository.find_by_id(context, id).await?;
            if let Some(value) = &method {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(method)
        })
    }

    fn find_by_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        if context.transaction.is_some() {
            return self.repository.find_by_code(context, code);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.text_key(generation, "code", code);

        Box::pin(async move {
            if let Some(method) = self.cached::<PollingMethod>(context, &key) {
                return Ok(Some(method));
            }

            let method = self.repository.find_by_code(context, code).await?;
            if let Some(value) = &method {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(method)
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<PollingMethod>> {
        if context.transaction.is_some() {
            return self.repository.list(context, filter);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(generation, "list", &filter);

        Box::pin(async move {
            if let Some(methods) = self.cached::<Vec<PollingMethod>>(context, &key) {
                return Ok(methods);
            }

            let methods = self.repository.list(context, filter).await?;
            self.store(context, &key, CacheKind::List, methods.clone(), generation);

            Ok(methods)
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, PollingMethod> {
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
