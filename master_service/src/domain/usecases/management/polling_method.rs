use crate::domain::models::{AppContext, entity::PollingMethod};

use super::super::UsecaseFuture;

/// Strict management operations for the seeded polling-method catalog.
///
/// Method creation remains owned by database seeds; codes are immutable.
pub trait ManagePollingMethod: Send + Sync {
    fn update_polling_method<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdatePollingMethod,
    ) -> UsecaseFuture<'a, PollingMethod>;

    fn delete_polling_method<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> UsecaseFuture<'a, ()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePollingMethod {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
