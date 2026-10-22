use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{Pagination, TopicRepository, topic::ListArgs},
            utility::Logger,
        },
        models::{AppContext, RepositoryError, entity::Topic},
        usecases::{
            UsecaseFuture,
            topic::{ListTopics, TopicCatalog},
        },
    },
};

pub struct TopicQueries {
    topics: Arc<dyn TopicRepository>,
    logger: Arc<dyn Logger>,
}

impl TopicQueries {
    pub fn new(topics: Arc<dyn TopicRepository>, logger: Arc<dyn Logger>) -> Self {
        Self { topics, logger }
    }
}

impl TopicCatalog for TopicQueries {
    fn list_active_topics<'a>(
        &'a self,
        context: &'a AppContext,
        request: ListTopics,
    ) -> UsecaseFuture<'a, Vec<Topic>> {
        Box::pin(async move {
            self.topics
                .list(
                    context,
                    ListArgs {
                        polling_method_id: request.polling_method_id,
                        is_active: Some(true),
                        name_contains: request.name_contains,
                        include_deleted: false,
                        pagination: Pagination {
                            limit: request.page.limit,
                            offset: request.page.offset,
                        },
                    },
                )
                .await
                .map_err(Into::into)
        })
    }

    fn get_topic_by_slug<'a>(
        &'a self,
        context: &'a AppContext,
        slug: &'a str,
    ) -> UsecaseFuture<'a, Topic> {
        Box::pin(async move {
            let topic = self.topics.find_by_slug(context, slug).await?;
            let Some(topic) = topic else {
                let error = RepositoryError::TopicNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "topic/catalog/get_by_slug",
                    &error,
                );
                return Err(error.into());
            };
            Ok(topic)
        })
    }
}
