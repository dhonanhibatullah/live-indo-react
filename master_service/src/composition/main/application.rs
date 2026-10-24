use std::sync::Arc;

use crate::composition::main::infrastructure::Infrastructure;

use crate::{
    application::{
        geographic::{CityQueries, IndonesiaQueries, ProvinceQueries, RegionQueries},
        maintenance::GeoIpMaintenance,
        management::{
            CityManagement, IndonesiaBoundaryManagement, PollingMethodManagement,
            ProvinceManagement, TopicManagement,
        },
        misc::AppContextService,
        topic::{PollingMethodQueries, TopicQueries},
    },
    domain::{
        contracts::utility::Logger,
        usecases::{
            geographic::{GetCity, GetProvince, GetRegion, IsInsideIndonesia},
            maintenance::RefreshGeoIp,
            management::{
                ManageCity, ManageIndonesiaBoundary, ManagePollingMethod, ManageProvince,
                ManageTopic,
            },
            misc::app_context::CreateAppContext,
            topic::{PollingMethodCatalog, TopicCatalog},
        },
    },
};

pub struct Application {
    pub refresh_geoip: Arc<dyn RefreshGeoIp>,
    pub create_app_context: Arc<dyn CreateAppContext>,
    pub manage_topic: Arc<dyn ManageTopic>,
    pub manage_polling_method: Arc<dyn ManagePollingMethod>,
    pub manage_province: Arc<dyn ManageProvince>,
    pub manage_city: Arc<dyn ManageCity>,
    pub manage_indonesia_boundary: Arc<dyn ManageIndonesiaBoundary>,
    pub get_province: Arc<dyn GetProvince>,
    pub get_city: Arc<dyn GetCity>,
    pub get_region: Arc<dyn GetRegion>,
    pub is_inside_indonesia: Arc<dyn IsInsideIndonesia>,
    pub polling_method_catalog: Arc<dyn PollingMethodCatalog>,
    pub topic_catalog: Arc<dyn TopicCatalog>,
}

pub fn build(infrastructure: Infrastructure, logger: Arc<dyn Logger>) -> Application {
    let refresh_geoip: Arc<dyn RefreshGeoIp> = Arc::new(GeoIpMaintenance::new(
        infrastructure.geoip.clone(),
        logger.clone(),
    ));
    let create_app_context: Arc<dyn CreateAppContext> =
        Arc::new(AppContextService::new(infrastructure.app_context_factory));

    let manage_topic: Arc<dyn ManageTopic> = Arc::new(TopicManagement::new(
        infrastructure.topics.clone(),
        infrastructure.polling_methods.clone(),
        infrastructure.validator.clone(),
        logger.clone(),
    ));
    let manage_polling_method: Arc<dyn ManagePollingMethod> =
        Arc::new(PollingMethodManagement::new(
            infrastructure.polling_methods.clone(),
            infrastructure.validator.clone(),
            logger.clone(),
        ));
    let manage_province: Arc<dyn ManageProvince> = Arc::new(ProvinceManagement::new(
        infrastructure.provinces.clone(),
        infrastructure.validator.clone(),
        logger.clone(),
    ));
    let manage_city: Arc<dyn ManageCity> = Arc::new(CityManagement::new(
        infrastructure.cities.clone(),
        infrastructure.provinces.clone(),
        infrastructure.validator.clone(),
        logger.clone(),
    ));
    let manage_indonesia_boundary: Arc<dyn ManageIndonesiaBoundary> =
        Arc::new(IndonesiaBoundaryManagement::new(
            infrastructure.indonesia_boundary.clone(),
            infrastructure.validator.clone(),
            logger.clone(),
        ));

    let get_province: Arc<dyn GetProvince> = Arc::new(ProvinceQueries::new(
        infrastructure.provinces.clone(),
        logger.clone(),
    ));
    let get_city: Arc<dyn GetCity> = Arc::new(CityQueries::new(
        infrastructure.cities.clone(),
        logger.clone(),
    ));
    let get_region: Arc<dyn GetRegion> = Arc::new(
        RegionQueries::new(
            infrastructure.cities.clone(),
            infrastructure.provinces.clone(),
            logger.clone(),
        )
        .with_geoip(infrastructure.geoip),
    );
    let is_inside_indonesia: Arc<dyn IsInsideIndonesia> =
        Arc::new(IndonesiaQueries::new(infrastructure.indonesia_boundary));
    let polling_method_catalog: Arc<dyn PollingMethodCatalog> = Arc::new(
        PollingMethodQueries::new(infrastructure.polling_methods, logger.clone()),
    );
    let topic_catalog: Arc<dyn TopicCatalog> =
        Arc::new(TopicQueries::new(infrastructure.topics, logger));

    Application {
        refresh_geoip,
        create_app_context,
        manage_topic,
        manage_polling_method,
        manage_province,
        manage_city,
        manage_indonesia_boundary,
        get_province,
        get_city,
        get_region,
        is_inside_indonesia,
        polling_method_catalog,
        topic_catalog,
    }
}
