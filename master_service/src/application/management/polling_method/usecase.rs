use std::sync::Arc;

use crate::domain::{
    contracts::{
        repository::{PollingMethodRepository, polling_method as polling_contract},
        utility::{Logger, Validator},
    },
    models::{AppContext, entity::PollingMethod, logger::Meta},
    usecases::{
        UsecaseFuture,
        management::polling_method::{ManagePollingMethod, UpdatePollingMethod},
    },
};

pub struct PollingMethodManagement {
    polling_methods: Arc<dyn PollingMethodRepository>,
    validator: Arc<dyn Validator>,
    logger: Arc<dyn Logger>,
}

impl PollingMethodManagement {
    pub fn new(
        polling_methods: Arc<dyn PollingMethodRepository>,
        validator: Arc<dyn Validator>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            polling_methods,
            validator,
            logger,
        }
    }
}

impl ManagePollingMethod for PollingMethodManagement {
    fn update_polling_method<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdatePollingMethod,
    ) -> UsecaseFuture<'a, PollingMethod> {
        Box::pin(async move {
            if let Some(name) = input.name.as_deref() {
                self.validator.polling_method_name(context, name)?;
            }

            let method = self
                .polling_methods
                .update(
                    context,
                    input.id,
                    polling_contract::UpdateArgs {
                        name: input.name,
                        description: input.description,
                        is_active: input.is_active,
                    },
                )
                .await?;
            self.logger.info(
                context,
                "management/polling_method/update",
                "polling method updated",
                Some(&Meta::from([(
                    "polling_method_id".into(),
                    method.id.into(),
                )])),
            );
            Ok(method)
        })
    }

    fn delete_polling_method<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            self.polling_methods.delete(context, id).await?;
            self.logger.info(
                context,
                "management/polling_method/delete",
                "polling method deleted",
                Some(&Meta::from([("polling_method_id".into(), id.into())])),
            );
            Ok(())
        })
    }
}
