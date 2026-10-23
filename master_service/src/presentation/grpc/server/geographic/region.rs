use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    geographic_region_service_server::{GeographicRegionService, GeographicRegionServiceServer},
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::geographic::GetRegion;
use crate::presentation::grpc::{
    mapper::{entity, error},
    server::shared::app_context,
};

pub struct RegionServer {
    usecase: Arc<dyn GetRegion>,
}

impl RegionServer {
    pub fn new(usecase: Arc<dyn GetRegion>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl GeographicRegionService for RegionServer {
    async fn get_region_by_coord(
        &self,
        request: Request<proto::GetRegionByCoordRequest>,
    ) -> Result<Response<proto::GetRegionByCoordResponse>, Status> {
        let context = app_context(&request)?;
        let input = request.into_inner();
        let region = self
            .usecase
            .get_region_by_coord(&context, input.longitude, input.latitude)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetRegionByCoordResponse {
            region: region.map(entity::region),
        }))
    }
}

pub(crate) fn server(usecase: Arc<dyn GetRegion>) -> GeographicRegionServiceServer<RegionServer> {
    GeographicRegionServiceServer::new(RegionServer::new(usecase))
}
