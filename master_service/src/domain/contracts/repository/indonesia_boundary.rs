use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, IndonesiaBoundary},
};

use super::RepositoryFuture;

/// Persistence for the singleton Indonesian national boundary.
///
/// Reads include the row regardless of audit state. The row is created by the
/// master database seed; this contract exposes retrieval, replacement, and
/// national-boundary containment.
pub trait IndonesiaBoundaryRepository: Send + Sync {
    /// Return the singleton boundary, or `None` when it has not been seeded.
    fn get<'a>(
        &'a self,
        context: &'a AppContext,
    ) -> RepositoryFuture<'a, Option<IndonesiaBoundary>>;

    /// Replace mutable boundary fields and return the updated state.
    ///
    /// Empty update arguments return the current row without changing
    /// `updated_at`. A missing singleton returns `INDONESIA_BOUNDARY_NOT_FOUND`.
    fn update<'a>(
        &'a self,
        context: &'a AppContext,
        changes: UpdateArgs,
    ) -> RepositoryFuture<'a, IndonesiaBoundary>;

    /// Return whether a WGS84 point is inside the national boundary.
    ///
    /// Boundary points are included. A missing singleton returns `false`.
    /// Invalid longitude or latitude values return `BAD_ARGS`.
    fn is_inside<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> RepositoryFuture<'a, bool>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateArgs {
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    /// `None` leaves the date unchanged; `Some(None)` clears it.
    pub boundary_reference_date: Option<Option<NaiveDate>>,
}
