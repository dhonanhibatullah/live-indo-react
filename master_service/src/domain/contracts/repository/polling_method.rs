use crate::domain::models::{AppContext, entity::PollingMethod};

use super::{Pagination, RepositoryFuture};

/// Persistence for the seeded polling-method catalog.
///
/// Polling methods are created by database seeds; this contract only reads,
/// updates descriptive fields, or deletes them. Codes are immutable.
pub trait PollingMethodRepository: Send + Sync {
    /// Return a method by ID, including a deleted row.
    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<PollingMethod>>;

    /// Return a method by code, including a deleted row.
    fn find_by_code<'a>(
        &'a self,
        context: &'a AppContext,
        code: &'a str,
    ) -> RepositoryFuture<'a, Option<PollingMethod>>;

    /// List methods that match the filter; deleted rows are excluded unless requested.
    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<PollingMethod>>;

    /// Update a method that is not deleted and return its new state.
    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, PollingMethod>;

    /// Delete a method without releasing its code.
    fn delete<'a>(&'a self, context: &'a AppContext, id: i64) -> RepositoryFuture<'a, ()>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateArgs {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub is_active: Option<bool>,
    pub include_deleted: bool,
    pub pagination: Pagination,
}
