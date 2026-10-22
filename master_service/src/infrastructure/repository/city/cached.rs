use std::{any::Any, sync::Arc};

use crate::domain::{
    contracts::{
        caching::{Cache, CacheKind, CacheValue},
        repository::{
            CityRepository, RepositoryFuture,
            city::{CreateArgs, ListArgs, UpdateArgs},
        },
    },
    models::{AppContext, entity::City},
};

use super::super::shared::cache::CacheNamespace;

/// Cache-aside decorator for city/regency reads.
pub struct CachedCity {
    repository: Arc<dyn CityRepository>,
    cache: Arc<dyn Cache>,
    namespace: CacheNamespace,
}

impl CachedCity {
    pub fn new(repository: Arc<dyn CityRepository>, cache: Arc<dyn Cache>) -> Self {
        Self {
            repository,
            cache,
            namespace: CacheNamespace::new("city"),
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

impl CityRepository for CachedCity {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, City> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.create(context, input).await
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<City>> {
        if context.transaction.is_some() {
            return self.repository.find_by_id(context, id);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.id_key(generation, id);

        Box::pin(async move {
            if let Some(city) = self.cached::<City>(context, &key) {
                return Ok(Some(city));
            }

            let city = self.repository.find_by_id(context, id).await?;
            if let Some(value) = &city {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(city)
        })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<City>> {
        if context.transaction.is_some() {
            return self.repository.find_by_kemendagri_code(context, code);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.text_key(generation, "kemendagri_code", code);

        Box::pin(async move {
            if let Some(city) = self.cached::<City>(context, &key) {
                return Ok(Some(city));
            }

            let city = self
                .repository
                .find_by_kemendagri_code(context, code)
                .await?;
            if let Some(value) = &city {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(city)
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<City>> {
        if context.transaction.is_some() {
            return self.repository.list(context, filter);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(generation, "list", &filter);

        Box::pin(async move {
            if let Some(cities) = self.cached::<Vec<City>>(context, &key) {
                return Ok(cities);
            }

            let cities = self.repository.list(context, filter).await?;
            self.store(context, &key, CacheKind::List, cities.clone(), generation);

            Ok(cities)
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, City> {
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
