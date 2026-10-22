use std::sync::Arc;

use crate::domain::{
    contracts::{
        repository::{IndonesiaBoundaryRepository, indonesia_boundary as boundary_contract},
        utility::{Logger, Validator},
    },
    models::{AppContext, entity::IndonesiaBoundary, logger::Meta},
    usecases::{
        UsecaseFuture,
        management::indonesia_boundary::{ManageIndonesiaBoundary, UpdateIndonesiaBoundary},
    },
};

pub struct IndonesiaBoundaryManagement {
    boundary: Arc<dyn IndonesiaBoundaryRepository>,
    validator: Arc<dyn Validator>,
    logger: Arc<dyn Logger>,
}

impl IndonesiaBoundaryManagement {
    pub fn new(
        boundary: Arc<dyn IndonesiaBoundaryRepository>,
        validator: Arc<dyn Validator>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            boundary,
            validator,
            logger,
        }
    }
}

impl ManageIndonesiaBoundary for IndonesiaBoundaryManagement {
    fn update_indonesia_boundary<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateIndonesiaBoundary,
    ) -> UsecaseFuture<'a, IndonesiaBoundary> {
        Box::pin(async move {
            if let Some(source) = input.boundary_source.as_deref() {
                self.validator.province_boundary_source(context, source)?;
            }

            let boundary = self
                .boundary
                .update(
                    context,
                    boundary_contract::UpdateArgs {
                        boundary: input.boundary,
                        boundary_source: input.boundary_source,
                        boundary_reference_date: input.boundary_reference_date,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/indonesia_boundary/update",
                "Indonesia boundary updated",
                Some(&Meta::from([("boundary_id".into(), boundary.id.into())])),
            );
            Ok(boundary)
        })
    }
}
