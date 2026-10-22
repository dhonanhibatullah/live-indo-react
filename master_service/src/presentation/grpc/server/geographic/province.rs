use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    geographic_province_service_server::{
        GeographicProvinceService, GeographicProvinceServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::geographic::GetProvince;
use crate::presentation::grpc::{
    mapper::{entity, error},
    server::shared::app_context,
};

pub struct ProvinceServer {
    usecase: Arc<dyn GetProvince>,
}

impl ProvinceServer {
    pub fn new(usecase: Arc<dyn GetProvince>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl GeographicProvinceService for ProvinceServer {
    async fn get_province_by_id(
        &self,
        request: Request<proto::GetProvinceByIdRequest>,
    ) -> Result<Response<proto::GetProvinceByIdResponse>, Status> {
        let context = app_context(&request)?;
        let province = self
            .usecase
            .get_province_by_id(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetProvinceByIdResponse {
            province: Some(entity::province(province)),
        }))
    }

    async fn get_province_by_kemendagri_code(
        &self,
        request: Request<proto::GetProvinceByKemendagriCodeRequest>,
    ) -> Result<Response<proto::GetProvinceByKemendagriCodeResponse>, Status> {
        let context = app_context(&request)?;
        let code = request.into_inner().kemendagri_code;
        let province = self
            .usecase
            .get_province_by_kemendagri_code(&context, &code)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetProvinceByKemendagriCodeResponse {
            province: Some(entity::province(province)),
        }))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn GetProvince>,
) -> GeographicProvinceServiceServer<ProvinceServer> {
    GeographicProvinceServiceServer::new(ProvinceServer::new(usecase))
}
