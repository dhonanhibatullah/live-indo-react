use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    management_indonesia_boundary_service_server::{
        ManagementIndonesiaBoundaryService, ManagementIndonesiaBoundaryServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::management::indonesia_boundary::ManageIndonesiaBoundary;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct IndonesiaBoundaryServer {
    usecase: Arc<dyn ManageIndonesiaBoundary>,
}

impl IndonesiaBoundaryServer {
    pub fn new(usecase: Arc<dyn ManageIndonesiaBoundary>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ManagementIndonesiaBoundaryService for IndonesiaBoundaryServer {
    async fn update_indonesia_boundary(
        &self,
        request: Request<proto::UpdateIndonesiaBoundaryRequest>,
    ) -> Result<Response<proto::UpdateIndonesiaBoundaryResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::update_indonesia_boundary(request.into_inner())?;
        let boundary = self
            .usecase
            .update_indonesia_boundary(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::UpdateIndonesiaBoundaryResponse {
            boundary: Some(entity::indonesia_boundary(boundary)),
        }))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn ManageIndonesiaBoundary>,
) -> ManagementIndonesiaBoundaryServiceServer<IndonesiaBoundaryServer> {
    ManagementIndonesiaBoundaryServiceServer::new(IndonesiaBoundaryServer::new(usecase))
}
