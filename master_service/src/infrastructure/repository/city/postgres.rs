use std::sync::Arc;

use futures_util::TryStreamExt;
use pgdt::Pgdt;

use crate::domain::{
    contracts::{
        repository::{
            RepositoryFuture,
            city::{CityRepository, CreateArgs, ListArgs, UpdateArgs},
        },
        utility::Logger,
    },
    models::{AppContext, RepositoryError, entity::City},
};

use super::{
    super::shared::{
        error::{OperationError, finish, require_affected},
        scan,
    },
    postgres_query as query,
};

pub struct PostgresCity {
    database: Pgdt,
    logger: Arc<dyn Logger>,
}

impl PostgresCity {
    pub fn new(database: Pgdt, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

impl CityRepository for PostgresCity {
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, City> {
        Box::pin(async move {
            let result: Result<City, OperationError> = async {
                let mut query = query::create(&input)?;
                let row = self.database.query_row(context, query.build()).await?;
                scan::city::decode(&row)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/create",
                result,
                || RepositoryError::CityNotFound,
            )
        })
    }

    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<City>> {
        Box::pin(async move {
            let result: Result<Option<City>, OperationError> = async {
                let mut query = query::find_by_id(id);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::city::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/find_by_id",
                result,
                || RepositoryError::CityNotFound,
            )
        })
    }

    fn find_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<City>> {
        Box::pin(async move {
            let result: Result<Option<City>, OperationError> = async {
                let mut query = query::find_by_kemendagri_code(code);
                match self.database.query_optional(context, query.build()).await? {
                    Some(row) => Ok(Some(scan::city::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/find_by_kemendagri_code",
                result,
                || RepositoryError::CityNotFound,
            )
        })
    }

    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<City>> {
        Box::pin(async move {
            let result: Result<Vec<City>, OperationError> = async {
                let mut query = query::list(&filter);
                let mut rows = self.database.query(context, query.build());
                let mut items = Vec::new();
                while let Some(row) = rows.try_next().await? {
                    items.push(scan::city::decode(&row)?);
                }
                Ok(items)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/list",
                result,
                || RepositoryError::CityNotFound,
            )
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, City> {
        Box::pin(async move {
            let result: Result<City, OperationError> = async {
                let mut query = query::update(id, &changes)?;
                let row = self.database.query_row(context, query.build()).await?;
                scan::city::decode(&row)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/update",
                result,
                || RepositoryError::CityNotFound,
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
                    || RepositoryError::CityNotFound,
                )
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/city/delete",
                result,
                || RepositoryError::CityNotFound,
            )
        })
    }
}
