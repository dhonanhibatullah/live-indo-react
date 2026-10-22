use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{Pagination, PollingMethodRepository, polling_method::ListArgs},
            utility::Logger,
        },
        models::{AppContext, RepositoryError, entity::PollingMethod},
        usecases::{
            UsecaseFuture,
            topic::{ListPollingMethods, PollingMethodCatalog},
        },
    },
};

pub struct PollingMethodQueries {
    polling_methods: Arc<dyn PollingMethodRepository>,
    logger: Arc<dyn Logger>,
}

impl PollingMethodQueries {
    pub fn new(polling_methods: Arc<dyn PollingMethodRepository>, logger: Arc<dyn Logger>) -> Self {
        Self {
            polling_methods,
            logger,
        }
    }
}

impl PollingMethodCatalog for PollingMethodQueries {
    fn list_available_polling_methods<'a>(
        &'a self,
        context: &'a AppContext,
        request: ListPollingMethods,
    ) -> UsecaseFuture<'a, Vec<PollingMethod>> {
        Box::pin(async move {
            self.polling_methods
                .list(
                    context,
                    ListArgs {
                        is_active: Some(true),
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

    fn get_polling_method_by_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, PollingMethod> {
        Box::pin(async move {
            let method = self.polling_methods.find_by_code(context, code).await?;
            let Some(method) = method else {
                let error = RepositoryError::PollingMethodNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "topic/polling_method/get_by_code",
                    &error,
                );
                return Err(error.into());
            };
            Ok(method)
        })
    }
}
