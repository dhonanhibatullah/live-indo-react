use std::sync::Arc;

use pgdt::Pgdt;

use crate::{
    domain::{
        contracts::{
            caching::Cache,
            repository::{
                CityRepository, IndonesiaBoundaryRepository, PollingMethodRepository,
                ProvinceRepository, TopicRepository,
            },
            utility::{Logger, Validator, app_context::AppContextFactory},
        },
        models::AppEnv,
    },
    infrastructure::{
        caching::MokaCache,
        repository::{
            CachedCity, CachedIndonesiaBoundary, CachedPollingMethod, CachedProvince, CachedTopic,
            PostgresCity, PostgresIndonesiaBoundary, PostgresPollingMethod, PostgresProvince,
            PostgresTopic,
        },
        utility::{UuidAppContextFactory, validator::RegexValidator},
    },
};

pub struct Infrastructure {
    pub app_context_factory: Arc<dyn AppContextFactory>,
    pub validator: Arc<dyn Validator>,
    pub topics: Arc<dyn TopicRepository>,
    pub polling_methods: Arc<dyn PollingMethodRepository>,
    pub provinces: Arc<dyn ProvinceRepository>,
    pub cities: Arc<dyn CityRepository>,
    pub indonesia_boundary: Arc<dyn IndonesiaBoundaryRepository>,
}

pub fn build(database: Pgdt, logger: Arc<dyn Logger>, env: &AppEnv) -> Infrastructure {
    let cache: Arc<dyn Cache> = Arc::new(MokaCache::new(
        env.cache_max_capacity,
        env.cache_individual_ttl,
        env.cache_list_ttl,
    ));

    let app_context_factory: Arc<dyn AppContextFactory> = Arc::new(UuidAppContextFactory::new());
    let validator: Arc<dyn Validator> = Arc::new(RegexValidator::new(logger.clone()));

    let topics: Arc<dyn TopicRepository> = Arc::new(CachedTopic::new(
        Arc::new(PostgresTopic::new(database.clone(), logger.clone())),
        cache.clone(),
    ));
    let polling_methods: Arc<dyn PollingMethodRepository> = Arc::new(CachedPollingMethod::new(
        Arc::new(PostgresPollingMethod::new(database.clone(), logger.clone())),
        cache.clone(),
    ));
    let provinces: Arc<dyn ProvinceRepository> = Arc::new(CachedProvince::new(
        Arc::new(PostgresProvince::new(database.clone(), logger.clone())),
        cache.clone(),
    ));
    let cities: Arc<dyn CityRepository> = Arc::new(CachedCity::new(
        Arc::new(PostgresCity::new(database.clone(), logger.clone())),
        cache.clone(),
    ));
    let indonesia_boundary: Arc<dyn IndonesiaBoundaryRepository> =
        Arc::new(CachedIndonesiaBoundary::new(
            Arc::new(PostgresIndonesiaBoundary::new(database, logger.clone())),
            cache,
        ));

    Infrastructure {
        app_context_factory,
        validator,
        topics,
        polling_methods,
        provinces,
        cities,
        indonesia_boundary,
    }
}
