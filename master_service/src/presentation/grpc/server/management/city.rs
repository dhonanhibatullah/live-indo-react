use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    management_city_service_server::{ManagementCityService, ManagementCityServiceServer},
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::management::city::ManageCity;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct CityServer {
    usecase: Arc<dyn ManageCity>,
}

impl CityServer {
    pub fn new(usecase: Arc<dyn ManageCity>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ManagementCityService for CityServer {
    async fn create_city(
        &self,
        request: Request<proto::CreateCityRequest>,
    ) -> Result<Response<proto::CreateCityResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::create_city(request.into_inner())?;
        let city = self
            .usecase
            .create_city(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::CreateCityResponse {
            city: Some(entity::city(city)),
        }))
    }

    async fn update_city(
        &self,
        request: Request<proto::UpdateCityRequest>,
    ) -> Result<Response<proto::UpdateCityResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::update_city(request.into_inner())?;
        let city = self
            .usecase
            .update_city(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::UpdateCityResponse {
            city: Some(entity::city(city)),
        }))
    }

    async fn delete_city(
        &self,
        request: Request<proto::DeleteCityRequest>,
    ) -> Result<Response<proto::DeleteCityResponse>, Status> {
        let context = app_context(&request)?;
        self.usecase
            .delete_city(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::DeleteCityResponse {}))
    }
}

pub(crate) fn server(usecase: Arc<dyn ManageCity>) -> ManagementCityServiceServer<CityServer> {
    ManagementCityServiceServer::new(CityServer::new(usecase))
}
