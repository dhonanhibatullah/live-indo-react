use std::sync::Arc;

use pgdt::{Pgdt, sqlx::Row};

use crate::domain::{
    contracts::{
        repository::{
            RepositoryFuture,
            indonesia_boundary::{IndonesiaBoundaryRepository, UpdateArgs},
        },
        utility::Logger,
    },
    models::{AppContext, RepositoryError, entity::IndonesiaBoundary},
};

use super::{
    super::shared::{
        error::{OperationError, finish},
        geo, scan,
    },
    postgres_query as query,
};

pub struct PostgresIndonesiaBoundary {
    database: Pgdt,
    logger: Arc<dyn Logger>,
}

impl PostgresIndonesiaBoundary {
    pub fn new(database: Pgdt, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

impl IndonesiaBoundaryRepository for PostgresIndonesiaBoundary {
    fn get<'a>(
        &'a self,
        context: &'a AppContext,
    ) -> RepositoryFuture<'a, Option<IndonesiaBoundary>> {
        Box::pin(async move {
            let result: Result<Option<IndonesiaBoundary>, OperationError> = async {
                match self
                    .database
                    .query_optional(context, query::get().build())
                    .await?
                {
                    Some(row) => Ok(Some(scan::indonesia_boundary::decode(&row)?)),
                    None => Ok(None),
                }
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/indonesia_boundary/get",
                result,
                || RepositoryError::IndonesiaBoundaryNotFound,
            )
        })
    }

    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, IndonesiaBoundary> {
        Box::pin(async move {
            let result: Result<IndonesiaBoundary, OperationError> = async {
                let mut query = query::update(&changes)?;
                let row = self.database.query_row(context, query.build()).await?;
                scan::indonesia_boundary::decode(&row)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/indonesia_boundary/update",
                result,
                || RepositoryError::IndonesiaBoundaryNotFound,
            )
        })
    }

    fn is_inside<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> RepositoryFuture<'a, bool> {
        Box::pin(async move {
            let result: Result<bool, OperationError> = async {
                if !geo::valid_coordinate(longitude, latitude) {
                    return Err(RepositoryError::BadArgs.into());
                }
                let row = self
                    .database
                    .query_row(context, query::is_inside(longitude, latitude).build())
                    .await?;
                Ok(row.try_get::<bool, _>("inside")?)
            }
            .await;
            finish(
                self.logger.as_ref(),
                context,
                "repository/indonesia_boundary/is_inside",
                result,
                || RepositoryError::IndonesiaBoundaryNotFound,
            )
        })
    }
}
