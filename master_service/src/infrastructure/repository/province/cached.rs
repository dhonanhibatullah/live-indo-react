use std::{any::Any, sync::Arc};

use crate::domain::{
    contracts::{
        caching::{Cache, CacheKind, CacheValue},
        repository::{
            ProvinceRepository, RepositoryFuture,
            province::{CreateArgs, ListArgs, UpdateArgs},
        },
    },
    models::{AppContext, entity::Province},
};

use super::super::shared::cache::CacheNamespace;

/// Cache-aside decorator for province reads.
pub struct CachedProvince {
    repository: Arc<dyn ProvinceRepository>,
    cache: Arc<dyn Cache>,
    namespace: CacheNamespace,
}

impl CachedProvince {
    pub fn new(repository: Arc<dyn ProvinceRepository>, cache: Arc<dyn Cache>) -> Self {
        Self {
            repository,
            cache,
            namespace: CacheNamespace::new("province"),
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

impl ProvinceRepository for CachedProvince {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Province> {
        Box::pin(async move {
            let _guard = self.namespace.invalidation_guard();
            self.repository.create(context, input).await
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        if context.transaction.is_some() {
            return self.repository.find_by_id(context, id);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.id_key(generation, id);

        Box::pin(async move {
            if let Some(province) = self.cached::<Province>(context, &key) {
                return Ok(Some(province));
            }

            let province = self.repository.find_by_id(context, id).await?;
            if let Some(value) = &province {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(province)
        })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<Province>> {
        if context.transaction.is_some() {
            return self.repository.find_by_kemendagri_code(context, code);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.text_key(generation, "kemendagri_code", code);

        Box::pin(async move {
            if let Some(province) = self.cached::<Province>(context, &key) {
                return Ok(Some(province));
            }

            let province = self
                .repository
                .find_by_kemendagri_code(context, code)
                .await?;
            if let Some(value) = &province {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(province)
        })
    }

    fn find_province_by_coord<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        if context.transaction.is_some() {
            return self
                .repository
                .find_province_by_coord(context, longitude, latitude);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(
            generation,
            "coord",
            &(longitude.to_bits(), latitude.to_bits()),
        );

        Box::pin(async move {
            if let Some(province) = self.cached::<Province>(context, &key) {
                return Ok(Some(province));
            }

            let province = self
                .repository
                .find_province_by_coord(context, longitude, latitude)
                .await?;
            if let Some(value) = &province {
                self.store(
                    context,
                    &key,
                    CacheKind::Individual,
                    value.clone(),
                    generation,
                );
            }

            Ok(province)
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Province>> {
        if context.transaction.is_some() {
            return self.repository.list(context, filter);
        }

        let generation = self.namespace.generation();
        let key = self.namespace.hash_key(generation, "list", &filter);

        Box::pin(async move {
            if let Some(provinces) = self.cached::<Vec<Province>>(context, &key) {
                return Ok(provinces);
            }

            let provinces = self.repository.list(context, filter).await?;
            self.store(
                context,
                &key,
                CacheKind::List,
                provinces.clone(),
                generation,
            );

            Ok(provinces)
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Province> {
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
