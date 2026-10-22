use crate::domain::models::{AppContext, entity::Topic};

use super::{Pagination, RepositoryFuture};

/// Persistence for nationwide reaction topics.
///
/// Slugs remain reserved after deletion and are immutable after creation.
/// Uniqueness violations are reported as `TOPIC_SLUG_CONFLICT`; a missing
/// polling method is reported as `TOPIC_POLLING_METHOD_NOT_FOUND`.
pub trait TopicRepository: Send + Sync {
    /// Insert a topic and return the persisted state.
    fn create<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateArgs,
    ) -> RepositoryFuture<'a, Topic>;

    /// Return a topic by ID, including a deleted row.
    fn find_by_id<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
    ) -> RepositoryFuture<'a, Option<Topic>>;

    /// Return a topic by slug, including a deleted row.
    fn find_by_slug<'a>(
        &'a self,
        context: &'a AppContext,
        slug: &'a str,
    ) -> RepositoryFuture<'a, Option<Topic>>;

    /// List topics that match the filter; deleted rows are excluded unless requested.
    fn list<'a>(
        &'a self,
        context: &'a AppContext,
        filter: ListArgs,
    ) -> RepositoryFuture<'a, Vec<Topic>>;

    /// Update a topic that is not deleted and return its new state.
    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        id: i64,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, Topic>;

    /// Delete a topic without releasing its slug.
    fn delete<'a>(&'a self, context: &'a AppContext, id: i64) -> RepositoryFuture<'a, ()>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateArgs {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub polling_method_id: i64,
    pub selection_options: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateArgs {
    pub name: Option<String>,
    pub description: Option<String>,
    pub polling_method_id: Option<i64>,
    pub selection_options: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListArgs {
    pub polling_method_id: Option<i64>,
    pub is_active: Option<bool>,
    pub name_contains: Option<String>,
    pub include_deleted: bool,
    pub pagination: Pagination,
}
