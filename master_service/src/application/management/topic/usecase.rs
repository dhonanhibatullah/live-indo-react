use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{PollingMethodRepository, TopicRepository, topic as topic_contract},
            utility::{Logger, Validator},
        },
        models::{AppContext, RepositoryError, entity::Topic, logger::Meta},
        usecases::{
            UsecaseFuture,
            management::topic::{CreateTopic, ManageTopic, UpdateTopic},
        },
    },
};

pub struct TopicManagement {
    topics: Arc<dyn TopicRepository>,
    polling_methods: Arc<dyn PollingMethodRepository>,
    validator: Arc<dyn Validator>,
    logger: Arc<dyn Logger>,
}

impl TopicManagement {
    pub fn new(
        topics: Arc<dyn TopicRepository>,
        polling_methods: Arc<dyn PollingMethodRepository>,
        validator: Arc<dyn Validator>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            topics,
            polling_methods,
            validator,
            logger,
        }
    }
}

impl ManageTopic for TopicManagement {
    fn create_topic<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateTopic,
    ) -> UsecaseFuture<'a, Topic> {
        Box::pin(async move {
            self.validator.topic_slug(context, &input.slug)?;
            self.validator.topic_name(context, &input.name)?;

            let Some(method) = self
                .polling_methods
                .find_by_id(context, input.polling_method_id)
                .await?
            else {
                let error = RepositoryError::TopicPollingMethodNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "management/topic/create",
                    &error,
                );
                return Err(error.into());
            };
            self.validator.topic_selection_options(
                context,
                &method.code,
                &input.selection_options,
            )?;

            let topic = self
                .topics
                .create(
                    context,
                    topic_contract::CreateArgs {
                        slug: input.slug,
                        name: input.name,
                        description: input.description,
                        polling_method_id: input.polling_method_id,
                        selection_options: input.selection_options,
                        is_active: input.is_active,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/topic/create",
                "topic created",
                Some(&Meta::from([("topic_id".into(), topic.id.into())])),
            );
            Ok(topic)
        })
    }

    fn update_topic<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateTopic,
    ) -> UsecaseFuture<'a, Topic> {
        Box::pin(async move {
            if let Some(name) = input.name.as_deref() {
                self.validator.topic_name(context, name)?;
            }

            if input.polling_method_id.is_some() || input.selection_options.is_some() {
                let Some(current) = self.topics.find_by_id(context, input.id).await? else {
                    let error = RepositoryError::TopicNotFound;
                    log_repository_error(
                        self.logger.as_ref(),
                        context,
                        "management/topic/update",
                        &error,
                    );
                    return Err(error.into());
                };
                let method_id = input.polling_method_id.unwrap_or(current.polling_method_id);
                let Some(method) = self.polling_methods.find_by_id(context, method_id).await?
                else {
                    let error = RepositoryError::TopicPollingMethodNotFound;
                    log_repository_error(
                        self.logger.as_ref(),
                        context,
                        "management/topic/update",
                        &error,
                    );
                    return Err(error.into());
                };
                let options = input
                    .selection_options
                    .as_deref()
                    .unwrap_or(current.selection_options.as_slice());
                self.validator
                    .topic_selection_options(context, &method.code, options)?;
            }

            let topic = self
                .topics
                .update(
                    context,
                    input.id,
                    topic_contract::UpdateArgs {
                        name: input.name,
                        description: input.description,
                        polling_method_id: input.polling_method_id,
                        selection_options: input.selection_options,
                        is_active: input.is_active,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/topic/update",
                "topic updated",
                Some(&Meta::from([("topic_id".into(), topic.id.into())])),
            );
            Ok(topic)
        })
    }

    fn delete_topic<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            self.topics.delete(context, id).await?;
            self.logger.info(
                context,
                "management/topic/delete",
                "topic deleted",
                Some(&Meta::from([("topic_id".into(), id.into())])),
            );
            Ok(())
        })
    }
}
