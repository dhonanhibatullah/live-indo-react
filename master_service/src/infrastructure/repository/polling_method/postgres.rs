use std::sync::Arc;

use futures_util::TryStreamExt;
use pgdt::Pgdt;

use crate::domain::{
    contracts::{
        repository::polling_method::{ListArgs, PollingMethodRepository, UpdateArgs},
        utility::Logger,
    },
    models::{AppContext, RepositoryError, entity::PollingMethod},
};

use super::{
    super::shared::{
        error::{OperationError, finish, require_affected},
        scan,
    },
    postgres_query as query,
};

pub struct PostgresPollingMethod {
    database: Pgdt,
    logger: Arc<dyn Logger>,
}

impl PostgresPollingMethod {
    pub fn new(database: Pgdt, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

impl PollingMethodRepository for PostgresPollingMethod {
    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> crate::domain::contracts::repository::RepositoryFuture<'a, Option<PollingMethod>> {
        Box::pin(async move {
            let result: Result<Option<PollingMethod>, OperationError> = async {
                let mut query = query::find_by_id(id);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::polling_method::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/polling_method/find_by_id",
                result,
                || RepositoryError::PollingMethodNotFound,
            )
        })
    }

    fn find_by_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> crate::domain::contracts::repository::RepositoryFuture<'a, Option<PollingMethod>> {
        Box::pin(async move {
            let result: Result<Option<PollingMethod>, OperationError> = async {
                let mut query = query::find_by_code(code);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::polling_method::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/polling_method/find_by_code",
                result,
                || RepositoryError::PollingMethodNotFound,
            )
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> crate::domain::contracts::repository::RepositoryFuture<'a, Vec<PollingMethod>> {
        Box::pin(async move {
            let result: Result<Vec<PollingMethod>, OperationError> = async {
                let mut query = query::list(&filter);
                let mut rows = self.database.query(context, query.build());
                let mut items = Vec::new();
                while let Some(row) = rows.try_next().await? {
                    items.push(scan::polling_method::decode(&row)?);
                }
                Ok(items)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/polling_method/list",
                result,
                || RepositoryError::PollingMethodNotFound,
            )
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> crate::domain::contracts::repository::RepositoryFuture<'a, PollingMethod> {
        Box::pin(async move {
            let result: Result<PollingMethod, OperationError> = async {
                let mut query = query::update(id, &changes);
                let row = self.database.query_row(context, query.build()).await?;
                Ok(scan::polling_method::decode(&row)?)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/polling_method/update",
                result,
                || RepositoryError::PollingMethodNotFound,
            )
        })
    }

    fn delete<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> crate::domain::contracts::repository::RepositoryFuture<'a, ()> {
        Box::pin(async move {
            let result: Result<(), OperationError> = async {
                let mut query = query::delete(id);
                require_affected(
                    self.database
                        .exec(context, query.build())
                        .await?
                        .rows_affected(),
                    || RepositoryError::PollingMethodNotFound,
                )
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/polling_method/delete",
                result,
                || RepositoryError::PollingMethodNotFound,
            )
        })
    }
}
