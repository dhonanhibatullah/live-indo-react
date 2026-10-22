use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, Province},
};

use super::{Pagination, RepositoryFuture};

/// Persistence for Indonesian province master data.
///
/// Kemendagri codes remain reserved after deletion and are immutable after
/// creation. Uniqueness violations are reported as
/// `PROVINCE_KEMENDAGRI_CODE_CONFLICT`.
pub trait ProvinceRepository: Send + Sync {
    /// Insert a province and return the persisted state.
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Province>;

    /// Return a province by ID, including a deleted row.
    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Province>>;

    /// Return a province by Kemendagri code, including a deleted row.
    fn find_by_kemendagri_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<Province>>;

    /// List provinces that match the filter; deleted rows are excluded unless requested.
    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Province>>;

    /// Update a province that is not deleted and return its new state.
    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Province>;

    /// Delete a province without releasing its code.
    fn delete<'a>(&'a self, context: &'a AppContext, id: i64) -> RepositoryFuture<'a, ()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateArgs {
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateArgs {
    pub name: Option<String>,
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    /// `None` leaves the date unchanged; `Some(None)` clears it.
    pub boundary_reference_date: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ListArgs {
    pub is_active: Option<bool>,
    pub name_contains: Option<String>,
    pub include_deleted: bool,
    pub pagination: Pagination,
}
