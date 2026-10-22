use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, City, CityKind},
};

use super::super::UsecaseFuture;

/// Strict management operations for cities and regencies.
pub trait ManageCity: Send + Sync {
    fn create_city<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateCity,
    ) -> UsecaseFuture<'a, City>;

    fn update_city<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateCity,
    ) -> UsecaseFuture<'a, City>;

    fn delete_city<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateCity {
    pub province_id: i64,
    pub kemendagri_code: String,
    pub kind: CityKind,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateCity {
    pub id: i64,
    pub kind: Option<CityKind>,
    pub name: Option<String>,
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}
