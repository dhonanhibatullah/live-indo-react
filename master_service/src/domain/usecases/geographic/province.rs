use crate::domain::models::{AppContext, entity::Province};

use super::super::UsecaseFuture;

/// Retrieve a province for geographic display.
pub trait GetProvince: Send + Sync {
    fn get_province_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> UsecaseFuture<'a, Province>;

    fn get_province_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> UsecaseFuture<'a, Province>;
}
