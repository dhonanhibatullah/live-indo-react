use std::{error::Error, sync::Arc};

use crate::{
    domain::{
        contracts::{
            caching::Cache,
            repository::{
                CityRepository, IndonesiaBoundaryRepository, PollingMethodRepository,
                ProvinceRepository, TopicRepository,
            },
            service::GeoIpService,
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
        service::geoip::HttpsGeoIp,
        utility::{UuidAppContextFactory, validator::RegexValidator},
    },
};

pub struct Infrastructure {
    pub geoip: Arc<dyn GeoIpService>,
    pub app_context_factory: Arc<dyn AppContextFactory>,
    pub validator: Arc<dyn Validator>,
    pub topics: Arc<dyn TopicRepository>,
    pub polling_methods: Arc<dyn PollingMethodRepository>,
    pub provinces: Arc<dyn ProvinceRepository>,
    pub cities: Arc<dyn CityRepository>,
    pub indonesia_boundary: Arc<dyn IndonesiaBoundaryRepository>,
}

/// Load credentials off the executor and initialize a local reader without downloading.
async fn geoip(
    env: &AppEnv,
    logger: Arc<dyn Logger>,
) -> Result<Arc<dyn GeoIpService>, Box<dyn Error + Send + Sync>> {
    let path = env.geoip_conf_path.clone();
    let configuration =
        tokio::task::spawn_blocking(move || crate::config::geoip_conf::load_from_path(path))
            .await??;
    Ok(Arc::new(
        HttpsGeoIp::new(configuration, env.geoip_database_path.clone(), logger).await?,
    ))
}

pub async fn build(
    drivers: &super::driver::Drivers,
    env: &AppEnv,
) -> Result<Infrastructure, Box<dyn Error + Send + Sync>> {
    let logger = drivers.logger.clone();
    let database = drivers.database.clone();
    let geoip = geoip(env, logger.clone()).await?;
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

    Ok(Infrastructure {
        geoip,
        app_context_factory,
        validator,
        topics,
        polling_methods,
        provinces,
        cities,
        indonesia_boundary,
    })
}
