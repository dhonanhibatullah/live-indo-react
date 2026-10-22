use std::sync::Arc;

use futures_util::TryStreamExt;
use pgdt::Pgdt;

use crate::domain::{
    contracts::{
        repository::{
            RepositoryFuture,
            province::{CreateArgs, ListArgs, ProvinceRepository, UpdateArgs},
        },
        utility::Logger,
    },
    models::{AppContext, RepositoryError, entity::Province},
};

use super::{
    super::shared::{
        error::{OperationError, finish, require_affected},
        scan,
    },
    postgres_query as query,
};

pub struct PostgresProvince {
    database: Pgdt,
    logger: Arc<dyn Logger>,
}

impl PostgresProvince {
    pub fn new(database: Pgdt, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

impl ProvinceRepository for PostgresProvince {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Province> {
        Box::pin(async move {
            let result: Result<Province, OperationError> = async {
                let mut query = query::create(&input)?;
                let row = self.database.query_row(context, query.build()).await?;
                scan::province::decode(&row)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/create",
                result,
                || RepositoryError::ProvinceNotFound,
            )
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        Box::pin(async move {
            let result: Result<Option<Province>, OperationError> = async {
                let mut query = query::find_by_id(id);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::province::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/find_by_id",
                result,
                || RepositoryError::ProvinceNotFound,
            )
        })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<Province>> {
        Box::pin(async move {
            let result: Result<Option<Province>, OperationError> = async {
                let mut query = query::find_by_kemendagri_code(code);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::province::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/find_by_kemendagri_code",
                result,
                || RepositoryError::ProvinceNotFound,
            )
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Province>> {
        Box::pin(async move {
            let result: Result<Vec<Province>, OperationError> = async {
                let mut query = query::list(&filter);
                let mut rows = self.database.query(context, query.build());
                let mut items = Vec::new();
                while let Some(row) = rows.try_next().await? {
                    items.push(scan::province::decode(&row)?);
                }
                Ok(items)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/list",
                result,
                || RepositoryError::ProvinceNotFound,
            )
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Province> {
        Box::pin(async move {
            let result: Result<Province, OperationError> = async {
                let mut query = query::update(id, &changes)?;
                let row = self.database.query_row(context, query.build()).await?;
                scan::province::decode(&row)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/update",
                result,
                || RepositoryError::ProvinceNotFound,
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
                    || RepositoryError::ProvinceNotFound,
                )
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/province/delete",
                result,
                || RepositoryError::ProvinceNotFound,
            )
        })
    }
}
