use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{repository::ProvinceRepository, utility::Logger},
        models::{AppContext, RepositoryError, entity::Province},
        usecases::{UsecaseFuture, geographic::GetProvince},
    },
};

pub struct ProvinceQueries {
    provinces: Arc<dyn ProvinceRepository>,
    logger: Arc<dyn Logger>,
}

impl ProvinceQueries {
    pub fn new(provinces: Arc<dyn ProvinceRepository>, logger: Arc<dyn Logger>) -> Self {
        Self { provinces, logger }
    }
}

impl GetProvince for ProvinceQueries {
    fn get_province_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> UsecaseFuture<'a, Province> {
        Box::pin(async move {
            let province = self.provinces.find_by_id(context, id).await?;
            let Some(province) = province else {
                let error = RepositoryError::ProvinceNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "geographic/province/get_by_id",
                    &error,
                );
                return Err(error.into());
            };
            Ok(province)
        })
    }

    fn get_province_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, Province> {
        Box::pin(async move {
            let province = self
                .provinces
                .find_by_kemendagri_code(context, code)
                .await?;
            let Some(province) = province else {
                let error = RepositoryError::ProvinceNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "geographic/province/get_by_kemendagri_code",
                    &error,
                );
                return Err(error.into());
            };
            Ok(province)
        })
    }
}
