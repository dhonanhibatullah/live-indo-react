use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    management_topic_service_server::{ManagementTopicService, ManagementTopicServiceServer},
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::management::topic::ManageTopic;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct TopicServer {
    usecase: Arc<dyn ManageTopic>,
}

impl TopicServer {
    pub fn new(usecase: Arc<dyn ManageTopic>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ManagementTopicService for TopicServer {
    async fn create_topic(
        &self,
        request: Request<proto::CreateTopicRequest>,
    ) -> Result<Response<proto::CreateTopicResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::create_topic(request.into_inner());
        let topic = self
            .usecase
            .create_topic(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::CreateTopicResponse {
            topic: Some(entity::topic(topic)),
        }))
    }

    async fn update_topic(
        &self,
        request: Request<proto::UpdateTopicRequest>,
    ) -> Result<Response<proto::UpdateTopicResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::update_topic(request.into_inner());
        let topic = self
            .usecase
            .update_topic(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::UpdateTopicResponse {
            topic: Some(entity::topic(topic)),
        }))
    }

    async fn delete_topic(
        &self,
        request: Request<proto::DeleteTopicRequest>,
    ) -> Result<Response<proto::DeleteTopicResponse>, Status> {
        let context = app_context(&request)?;
        self.usecase
            .delete_topic(&context, request.into_inner().id)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::DeleteTopicResponse {}))
    }
}

pub(crate) fn server(usecase: Arc<dyn ManageTopic>) -> ManagementTopicServiceServer<TopicServer> {
    ManagementTopicServiceServer::new(TopicServer::new(usecase))
}
