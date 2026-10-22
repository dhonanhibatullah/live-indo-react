use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    management_province_service_server::{
        ManagementProvinceService, ManagementProvinceServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::management::province::ManageProvince;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct ProvinceServer {
    usecase: Arc<dyn ManageProvince>,
}

impl ProvinceServer {
    pub fn new(usecase: Arc<dyn ManageProvince>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ManagementProvinceService for ProvinceServer {
    async fn create_province(
        &self,
        request: Request<proto::CreateProvinceRequest>,
    ) -> Result<Response<proto::CreateProvinceResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::create_province(request.into_inner())?;
        let province = self
            .usecase
            .create_province(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::CreateProvinceResponse {
            province: Some(entity::province(province)),
        }))
    }

    async fn update_province(
        &self,
        request: Request<proto::UpdateProvinceRequest>,
    ) -> Result<Response<proto::UpdateProvinceResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::update_province(request.into_inner())?;
        let province = self
            .usecase
            .update_province(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::UpdateProvinceResponse {
            province: Some(entity::province(province)),
        }))
    }

    async fn delete_province(
        &self,
        request: Request<proto::DeleteProvinceRequest>,
    ) -> Result<Response<proto::DeleteProvinceResponse>, Status> {
        let context = app_context(&request)?;
        self.usecase
            .delete_province(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::DeleteProvinceResponse {}))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn ManageProvince>,
) -> ManagementProvinceServiceServer<ProvinceServer> {
    ManagementProvinceServiceServer::new(ProvinceServer::new(usecase))
}
