use std::sync::Arc;

use futures_util::TryStreamExt;
use pgdt::Pgdt;

use crate::domain::{
    contracts::{
        repository::{
            RepositoryFuture,
            topic::{CreateArgs, ListArgs, TopicRepository, UpdateArgs},
        },
        utility::Logger,
    },
    models::{AppContext, RepositoryError, entity::Topic},
};

use super::{
    super::shared::{
        error::{OperationError, finish, require_affected},
        scan,
    },
    postgres_query as query,
};

pub struct PostgresTopic {
    database: Pgdt,
    logger: Arc<dyn Logger>,
}

impl PostgresTopic {
    pub fn new(database: Pgdt, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

impl TopicRepository for PostgresTopic {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        Box::pin(async move {
            let result: Result<Topic, OperationError> = async {
                let mut query = query::create(&input);
                let row = self.database.query_row(context, query.build()).await?;
                Ok(scan::topic::decode(&row)?)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/create",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        Box::pin(async move {
            let result: Result<Option<Topic>, OperationError> = async {
                let mut query = query::find_by_id(id);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::topic::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/find_by_id",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }

    fn find_by_slug<'a>(
        &'a self,
        context: &'a AppContext,
        slug: &'a str,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        Box::pin(async move {
            let result: Result<Option<Topic>, OperationError> = async {
                let mut query = query::find_by_slug(slug);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::topic::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/find_by_slug",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Topic>> {
        Box::pin(async move {
            let result: Result<Vec<Topic>, OperationError> = async {
                let mut query = query::list(&filter);
                let mut rows = self.database.query(context, query.build());
                let mut items = Vec::new();
                while let Some(row) = rows.try_next().await? {
                    items.push(scan::topic::decode(&row)?);
                }
                Ok(items)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/list",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        Box::pin(async move {
            let result: Result<Topic, OperationError> = async {
                let mut query = query::update(id, &changes);
                let row = self.database.query_row(context, query.build()).await?;
                Ok(scan::topic::decode(&row)?)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/update",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }

    fn delete<'a>(&'a self, context: &'a AppContext, id: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async move {
            let result: Result<(), OperationError> = async {
                let mut query = query::delete(id);
                require_affected(
                    self.database
                        .exec(context, query.build())
                        .await?
                        .rows_affected(),
                    || RepositoryError::TopicNotFound,
                )
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/topic/delete",
                result,
                || RepositoryError::TopicNotFound,
            )
        })
    }
}
