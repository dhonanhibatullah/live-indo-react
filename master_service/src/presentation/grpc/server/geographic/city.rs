use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    geographic_city_service_server::{GeographicCityService, GeographicCityServiceServer},
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::geographic::GetCity;
use crate::presentation::grpc::{
    mapper::{entity, error},
    server::shared::app_context,
};

pub struct CityServer {
    usecase: Arc<dyn GetCity>,
}

impl CityServer {
    pub fn new(usecase: Arc<dyn GetCity>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl GeographicCityService for CityServer {
    async fn get_city_by_id(
        &self,
        request: Request<proto::GetCityByIdRequest>,
    ) -> Result<Response<proto::GetCityByIdResponse>, Status> {
        let context = app_context(&request)?;
        let city = self
            .usecase
            .get_city_by_id(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetCityByIdResponse {
            city: Some(entity::city(city)),
        }))
    }

    async fn get_city_by_kemendagri_code(
        &self,
        request: Request<proto::GetCityByKemendagriCodeRequest>,
    ) -> Result<Response<proto::GetCityByKemendagriCodeResponse>, Status> {
        let context = app_context(&request)?;
        let code = request.into_inner().kemendagri_code;
        let city = self
            .usecase
            .get_city_by_kemendagri_code(&context, &code)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetCityByKemendagriCodeResponse {
            city: Some(entity::city(city)),
        }))
    }
}

pub(crate) fn server(usecase: Arc<dyn GetCity>) -> GeographicCityServiceServer<CityServer> {
    GeographicCityServiceServer::new(CityServer::new(usecase))
}
