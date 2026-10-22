use std::future::Future;

use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, City, CityKind},
    error::Error,
};

use super::Pagination;

/// Persistence for city/regency master data.
///
/// Kemendagri codes remain reserved after deletion. A city's code and
/// parent province are immutable after creation. Uniqueness violations are
/// reported as `CONFLICT`.
pub trait CityRepository: Send + Sync {
    /// Insert a city/regency and return the persisted state.
    fn create(
        &self,
        context: &AppContext,
        input: CreateArgs,
    ) -> impl Future<Output = Result<City, Error>> + Send;

    /// Return a city/regency by ID, including a deleted row.
    fn find_by_id(
        &self,
        context: &AppContext,
        id: i64,
    ) -> impl Future<Output = Result<Option<City>, Error>> + Send;

    /// Return a city/regency by Kemendagri code, including a deleted row.
    fn find_by_kemendagri_code(
        &self,
        context: &AppContext,
        code: &str,
    ) -> impl Future<Output = Result<Option<City>, Error>> + Send;

    /// List cities/regencies that match the filter; deleted rows are excluded unless requested.
    fn list(
        &self,
        context: &AppContext,
        filter: ListArgs,
    ) -> impl Future<Output = Result<Vec<City>, Error>> + Send;

    /// Update a city/regency that is not deleted and return its new state.
    fn update(
        &self,
        context: &AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> impl Future<Output = Result<City, Error>> + Send;

    /// Delete a city/regency without releasing its code.
    fn delete(
        &self,
        context: &AppContext,
        id: i64,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateArgs {
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
pub struct UpdateArgs {
    pub kind: Option<CityKind>,
    pub name: Option<String>,
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    /// `None` leaves the date unchanged; `Some(None)` clears it.
    pub boundary_reference_date: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub province_id: Option<i64>,
    pub kind: Option<CityKind>,
    pub is_active: Option<bool>,
    pub name_contains: Option<String>,
    pub include_deleted: bool,
    pub pagination: Pagination,
}
