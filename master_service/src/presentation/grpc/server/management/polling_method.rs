use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    management_polling_method_service_server::{
        ManagementPollingMethodService, ManagementPollingMethodServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::management::polling_method::ManagePollingMethod;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct PollingMethodServer {
    usecase: Arc<dyn ManagePollingMethod>,
}

impl PollingMethodServer {
    pub fn new(usecase: Arc<dyn ManagePollingMethod>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ManagementPollingMethodService for PollingMethodServer {
    async fn update_polling_method(
        &self,
        request: Request<proto::UpdatePollingMethodRequest>,
    ) -> Result<Response<proto::UpdatePollingMethodResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::update_polling_method(request.into_inner());
        let method = self
            .usecase
            .update_polling_method(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::UpdatePollingMethodResponse {
            polling_method: Some(entity::polling_method(method)),
        }))
    }

    async fn delete_polling_method(
        &self,
        request: Request<proto::DeletePollingMethodRequest>,
    ) -> Result<Response<proto::DeletePollingMethodResponse>, Status> {
        let context = app_context(&request)?;
        self.usecase
            .delete_polling_method(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::DeletePollingMethodResponse {}))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn ManagePollingMethod>,
) -> ManagementPollingMethodServiceServer<PollingMethodServer> {
    ManagementPollingMethodServiceServer::new(PollingMethodServer::new(usecase))
}
