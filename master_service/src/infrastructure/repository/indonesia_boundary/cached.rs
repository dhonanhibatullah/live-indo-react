use std::{any::Any, sync::Arc};

use crate::domain::{
    contracts::{
        caching::{Cache, CacheKind, CacheValue},
        repository::{
            RepositoryFuture,
            indonesia_boundary::{IndonesiaBoundaryRepository, UpdateArgs},
        },
    },
    models::{AppContext, entity::IndonesiaBoundary},
};

use super::super::shared::cache::CacheNamespace;

/// Cache-aside decorator for the singleton national boundary.
pub struct CachedIndonesiaBoundary {
    repository: Arc<dyn IndonesiaBoundaryRepository>,
    cache: Arc<dyn Cache>,
    namespace: CacheNamespace,
}

impl CachedIndonesiaBoundary {
    pub fn new(repository: Arc<dyn IndonesiaBoundaryRepository>, cache: Arc<dyn Cache>) -> Self {
        Self {
            repository,
            cache,
            namespace: CacheNamespace::new("indonesia_boundary"),
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

impl IndonesiaBoundaryRepository for CachedIndonesiaBoundary {
    fn get<'a>(
        &'a self,
        context: &'a AppContext,
    ) -> RepositoryFuture<'a, Option<IndonesiaBoundary>> {
        if context.transaction.is_some() {
            return self.repository.get(context);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.id_key(generation, 1);

        Box::pin(async move {
            if let Some(boundary) = self.cached::<IndonesiaBoundary>(context, &key) {
                return Ok(Some(boundary));
            }

            let boundary = self.repository.get(context).await?;
            if let Some(value) = &boundary {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(boundary)
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, IndonesiaBoundary> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.update(context, changes).await
        })
    }

    fn is_inside<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> RepositoryFuture<'a, bool> {
        if context.transaction.is_some() {
            return self.repository.is_inside(context, longitude, latitude);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(
            generation,
            "inside",
            &(longitude.to_bits(), latitude.to_bits()),
        );

        Box::pin(async move {
            if let Some(inside) = self.cached::<bool>(context, &key) {
                return Ok(inside);
            }

            let inside = self
                .repository
                .is_inside(context, longitude, latitude)
                .await?;
            self.store(context, &key, CacheKind::Individual, inside, generation);

            Ok(inside)
        })
    }
}
