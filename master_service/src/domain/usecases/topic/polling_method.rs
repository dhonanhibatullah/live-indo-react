use crate::domain::models::{AppContext, entity::PollingMethod};

use super::super::{Page, UsecaseFuture};

/// Public read access to the available polling-method catalog.
pub trait PollingMethodCatalog: Send + Sync {
    fn list_available_polling_methods<'a>(
        &'a self,
        context: &'a AppContext,
        request: ListPollingMethods,
    ) -> UsecaseFuture<'a, Vec<PollingMethod>>;

    fn get_polling_method_by_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, PollingMethod>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListPollingMethods {
    pub page: Page,
}
