use std::sync::Arc;

use crate::domain::{
    contracts::{
        repository::{ProvinceRepository, province as province_contract},
        utility::{Logger, Validator},
    },
    models::{AppContext, entity::Province, logger::Meta},
    usecases::{
        UsecaseFuture,
        management::province::{CreateProvince, ManageProvince, UpdateProvince},
    },
};

pub struct ProvinceManagement {
    provinces: Arc<dyn ProvinceRepository>,
    validator: Arc<dyn Validator>,
    logger: Arc<dyn Logger>,
}

impl ProvinceManagement {
    pub fn new(
        provinces: Arc<dyn ProvinceRepository>,
        validator: Arc<dyn Validator>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            provinces,
            validator,
            logger,
        }
    }
}

impl ManageProvince for ProvinceManagement {
    fn create_province<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateProvince,
    ) -> UsecaseFuture<'a, Province> {
        Box::pin(async move {
            self.validator
                .province_kemendagri_code(context, &input.kemendagri_code)?;
            self.validator.province_name(context, &input.name)?;
            self.validator
                .province_boundary_source(context, &input.boundary_source)?;

            let province = self
                .provinces
                .create(
                    context,
                    province_contract::CreateArgs {
                        kemendagri_code: input.kemendagri_code,
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
                "management/province/create",
                "province created",
                Some(&Meta::from([("province_id".into(), province.id.into())])),
            );
            Ok(province)
        })
    }

    fn update_province<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateProvince,
    ) -> UsecaseFuture<'a, Province> {
        Box::pin(async move {
            if let Some(name) = input.name.as_deref() {
                self.validator.province_name(context, name)?;
            }
            if let Some(source) = input.boundary_source.as_deref() {
                self.validator.province_boundary_source(context, source)?;
            }

            let province = self
                .provinces
                .update(
                    context,
                    input.id,
                    province_contract::UpdateArgs {
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
                "management/province/update",
                "province updated",
                Some(&Meta::from([("province_id".into(), province.id.into())])),
            );
            Ok(province)
        })
    }

    fn delete_province<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            self.provinces.delete(context, id).await?;
            self.logger.info(
                context,
                "management/province/delete",
                "province deleted",
                Some(&Meta::from([("province_id".into(), id.into())])),
            );
            Ok(())
        })
    }
}
