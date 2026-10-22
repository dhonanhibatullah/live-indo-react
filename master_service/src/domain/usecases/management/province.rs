use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, Province},
};

use super::super::UsecaseFuture;

/// Strict management operations for Indonesian provinces.
pub trait ManageProvince: Send + Sync {
    fn create_province<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateProvince,
    ) -> UsecaseFuture<'a, Province>;

    fn update_province<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateProvince,
    ) -> UsecaseFuture<'a, Province>;

    fn delete_province<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateProvince {
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateProvince {
    pub id: i64,
    pub name: Option<String>,
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}
