use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    geographic_indonesia_service_server::{
        GeographicIndonesiaService, GeographicIndonesiaServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::geographic::IsInsideIndonesia;
use crate::presentation::grpc::{mapper::error, server::shared::app_context};

pub struct IndonesiaServer {
    usecase: Arc<dyn IsInsideIndonesia>,
}

impl IndonesiaServer {
    pub fn new(usecase: Arc<dyn IsInsideIndonesia>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl GeographicIndonesiaService for IndonesiaServer {
    async fn is_inside_indonesia(
        &self,
        request: Request<proto::IsInsideIndonesiaRequest>,
    ) -> Result<Response<proto::IsInsideIndonesiaResponse>, Status> {
        let context = app_context(&request)?;
        let input = request.into_inner();
        let inside = self
            .usecase
            .is_inside_indonesia(&context, input.longitude, input.latitude)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::IsInsideIndonesiaResponse { inside }))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn IsInsideIndonesia>,
) -> GeographicIndonesiaServiceServer<IndonesiaServer> {
    GeographicIndonesiaServiceServer::new(IndonesiaServer::new(usecase))
}
