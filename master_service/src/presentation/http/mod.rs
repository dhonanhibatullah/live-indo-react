use std::sync::Arc;

use crate::domain::usecases::{
    geographic::{GetCity, GetProvince, GetRegionByCoord, IsInsideIndonesia},
    management::{
        ManageCity, ManageIndonesiaBoundary, ManagePollingMethod, ManageProvince, ManageTopic,
    },
    misc::app_context::CreateAppContext,
    topic::{PollingMethodCatalog, TopicCatalog},
};

pub mod dto;
pub mod handler;
pub mod middleware;
pub mod routes;

/// Usecase dependencies supplied by composition.
pub struct Dependencies {
    pub create_app_context: Arc<dyn CreateAppContext>,
    pub manage_topic: Arc<dyn ManageTopic>,
    pub manage_polling_method: Arc<dyn ManagePollingMethod>,
    pub manage_province: Arc<dyn ManageProvince>,
    pub manage_city: Arc<dyn ManageCity>,
    pub manage_indonesia_boundary: Arc<dyn ManageIndonesiaBoundary>,
    pub get_province: Arc<dyn GetProvince>,
    pub get_city: Arc<dyn GetCity>,
    pub get_region: Arc<dyn GetRegionByCoord>,
    pub is_inside_indonesia: Arc<dyn IsInsideIndonesia>,
    pub polling_method_catalog: Arc<dyn PollingMethodCatalog>,
    pub topic_catalog: Arc<dyn TopicCatalog>,
}
