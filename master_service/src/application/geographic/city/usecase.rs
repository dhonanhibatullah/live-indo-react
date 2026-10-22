use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{repository::CityRepository, utility::Logger},
        models::{AppContext, RepositoryError, entity::City},
        usecases::{UsecaseFuture, geographic::GetCity},
    },
};

pub struct CityQueries {
    cities: Arc<dyn CityRepository>,
    logger: Arc<dyn Logger>,
}

impl CityQueries {
    pub fn new(cities: Arc<dyn CityRepository>, logger: Arc<dyn Logger>) -> Self {
        Self { cities, logger }
    }
}

impl GetCity for CityQueries {
    fn get_city_by_id<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, City> {
        Box::pin(async move {
            let city = self.cities.find_by_id(context, id).await?;
            let Some(city) = city else {
                let error = RepositoryError::CityNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "geographic/city/get_by_id",
                    &error,
                );
                return Err(error.into());
            };
            Ok(city)
        })
    }

    fn get_city_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, City> {
        Box::pin(async move {
            let city = self.cities.find_by_kemendagri_code(context, code).await?;
            let Some(city) = city else {
                let error = RepositoryError::CityNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "geographic/city/get_by_kemendagri_code",
                    &error,
                );
                return Err(error.into());
            };
            Ok(city)
        })
    }
}
