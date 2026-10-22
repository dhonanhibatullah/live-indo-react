use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    topic_polling_method_service_server::{
        TopicPollingMethodService, TopicPollingMethodServiceServer,
    },
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::topic::PollingMethodCatalog;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct PollingMethodServer {
    usecase: Arc<dyn PollingMethodCatalog>,
}

impl PollingMethodServer {
    pub fn new(usecase: Arc<dyn PollingMethodCatalog>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl TopicPollingMethodService for PollingMethodServer {
    async fn list_available_polling_methods(
        &self,
        request: Request<proto::ListAvailablePollingMethodsRequest>,
    ) -> Result<Response<proto::ListAvailablePollingMethodsResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::list_polling_methods(request.into_inner())?;
        let methods = self
            .usecase
            .list_available_polling_methods(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::ListAvailablePollingMethodsResponse {
            polling_methods: methods.into_iter().map(entity::polling_method).collect(),
        }))
    }

    async fn get_polling_method_by_code(
        &self,
        request: Request<proto::GetPollingMethodByCodeRequest>,
    ) -> Result<Response<proto::GetPollingMethodByCodeResponse>, Status> {
        let context = app_context(&request)?;
        let code = request.into_inner().code;
        let method = self
            .usecase
            .get_polling_method_by_code(&context, &code)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetPollingMethodByCodeResponse {
            polling_method: Some(entity::polling_method(method)),
        }))
    }
}

pub(crate) fn server(
    usecase: Arc<dyn PollingMethodCatalog>,
) -> TopicPollingMethodServiceServer<PollingMethodServer> {
    TopicPollingMethodServiceServer::new(PollingMethodServer::new(usecase))
}
