use std::future::Future;

use crate::domain::models::{AppContext, entity::PollingMethod, error::RepositoryError};

use super::Pagination;

/// Persistence for the seeded polling-method catalog.
///
/// Polling methods are created by database seeds; this contract only reads,
/// updates descriptive fields, or deletes them. Codes are immutable.
pub trait PollingMethodRepository: Send + Sync {
    /// Return a method by ID, including a deleted row.
    fn find_by_id(
        &self,
        context: &AppContext,
        id: i64,
    ) -> impl Future<Output = Result<Option<PollingMethod>, RepositoryError>> + Send;

    /// Return a method by code, including a deleted row.
    fn find_by_code(
        &self,
        context: &AppContext,
        code: &str,
    ) -> impl Future<Output = Result<Option<PollingMethod>, RepositoryError>> + Send;

    /// List methods that match the filter; deleted rows are excluded unless requested.
    fn list(
        &self,
        context: &AppContext,
        filter: ListArgs,
    ) -> impl Future<Output = Result<Vec<PollingMethod>, RepositoryError>> + Send;

    /// Update a method that is not deleted and return its new state.
    fn update(
        &self,
        context: &AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> impl Future<Output = Result<PollingMethod, RepositoryError>> + Send;

    /// Delete a method without releasing its code.
    fn delete(
        &self,
        context: &AppContext,
        id: i64,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
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
