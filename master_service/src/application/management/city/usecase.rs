use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{CityRepository, ProvinceRepository, city as city_contract},
            utility::{Logger, Validator},
        },
        models::{AppContext, RepositoryError, entity::City, logger::Meta},
        usecases::{
            UsecaseFuture,
            management::city::{CreateCity, ManageCity, UpdateCity},
        },
    },
};

pub struct CityManagement {
    cities: Arc<dyn CityRepository>,
    provinces: Arc<dyn ProvinceRepository>,
    validator: Arc<dyn Validator>,
    logger: Arc<dyn Logger>,
}

impl CityManagement {
    pub fn new(
        cities: Arc<dyn CityRepository>,
        provinces: Arc<dyn ProvinceRepository>,
        validator: Arc<dyn Validator>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            cities,
            provinces,
            validator,
            logger,
        }
    }
}

impl ManageCity for CityManagement {
    fn create_city<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateCity,
    ) -> UsecaseFuture<'a, City> {
        Box::pin(async move {
            self.validator
                .city_kemendagri_code(context, &input.kemendagri_code)?;
            self.validator.city_name(context, &input.name)?;
            self.validator
                .city_boundary_source(context, &input.boundary_source)?;

            let Some(province) = self
                .provinces
                .find_by_id(context, input.province_id)
                .await?
            else {
                let error = RepositoryError::CityProvinceNotFound;
                log_repository_error(
                    self.logger.as_ref(),
                    context,
                    "management/city/create",
                    &error,
                );
                return Err(error.into());
            };
            self.validator.city_province_code(
                context,
                &input.kemendagri_code,
                &province.kemendagri_code,
            )?;

            let city = self
                .cities
                .create(
                    context,
                    city_contract::CreateArgs {
                        province_id: input.province_id,
                        kemendagri_code: input.kemendagri_code,
                        kind: input.kind,
                        name: input.name,
                        boundary: input.boundary,
                        boundary_source: input.boundary_source,
                        boundary_reference_date: input.boundary_reference_date,
                        is_active: input.is_active,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/city/create",
                "city created",
                Some(&Meta::from([
                    ("city_id".into(), city.id.into()),
                    ("province_id".into(), city.province_id.into()),
                ])),
            );
            Ok(city)
        })
    }

    fn update_city<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateCity,
    ) -> UsecaseFuture<'a, City> {
        Box::pin(async move {
            if let Some(name) = input.name.as_deref() {
                self.validator.city_name(context, name)?;
            }
            if let Some(source) = input.boundary_source.as_deref() {
                self.validator.city_boundary_source(context, source)?;
            }

            let city = self
                .cities
                .update(
                    context,
                    input.id,
                    city_contract::UpdateArgs {
                        kind: input.kind,
                        name: input.name,
                        boundary: input.boundary,
                        boundary_source: input.boundary_source,
                        boundary_reference_date: input.boundary_reference_date,
                        is_active: input.is_active,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/city/update",
                "city updated",
                Some(&Meta::from([("city_id".into(), city.id.into())])),
            );
            Ok(city)
        })
    }

    fn delete_city<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            self.cities.delete(context, id).await?;
            self.logger.info(
                context,
                "management/city/delete",
                "city deleted",
                Some(&Meta::from([("city_id".into(), id.into())])),
            );
            Ok(())
        })
    }
}
