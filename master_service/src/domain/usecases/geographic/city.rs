use crate::domain::models::{AppContext, entity::City};

use super::super::UsecaseFuture;

/// Retrieve a city or regency for geographic display.
pub trait GetCity: Send + Sync {
    fn get_city_by_id<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, City>;

    fn get_city_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, City>;
}
