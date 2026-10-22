use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto,
    topic_catalog_service_server::{TopicCatalogService, TopicCatalogServiceServer},
};
use tonic::{Request, Response, Status};

use crate::domain::usecases::topic::TopicCatalog;
use crate::presentation::grpc::{
    mapper::{entity, error, request},
    server::shared::app_context,
};

pub struct CatalogServer {
    usecase: Arc<dyn TopicCatalog>,
}

impl CatalogServer {
    pub fn new(usecase: Arc<dyn TopicCatalog>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl TopicCatalogService for CatalogServer {
    async fn list_active_topics(
        &self,
        request: Request<proto::ListActiveTopicsRequest>,
    ) -> Result<Response<proto::ListActiveTopicsResponse>, Status> {
        let context = app_context(&request)?;
        let input = request::list_topics(request.into_inner())?;
        let topics = self
            .usecase
            .list_active_topics(&context, input)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::ListActiveTopicsResponse {
            topics: topics.into_iter().map(entity::topic).collect(),
        }))
    }

    async fn get_topic_by_slug(
        &self,
        request: Request<proto::GetTopicBySlugRequest>,
    ) -> Result<Response<proto::GetTopicBySlugResponse>, Status> {
        let context = app_context(&request)?;
        let slug = request.into_inner().slug;
        let topic = self
            .usecase
            .get_topic_by_slug(&context, &slug)
            .await
            .map_err(error::usecase)?;
        Ok(Response::new(proto::GetTopicBySlugResponse {
            topic: Some(entity::topic(topic)),
        }))
    }
}

pub(crate) fn server(usecase: Arc<dyn TopicCatalog>) -> TopicCatalogServiceServer<CatalogServer> {
    TopicCatalogServiceServer::new(CatalogServer::new(usecase))
}
