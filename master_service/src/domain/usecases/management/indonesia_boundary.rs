use chrono::NaiveDate;

use crate::domain::models::{
    AppContext,
    entity::{Boundary, IndonesiaBoundary},
};

use super::super::UsecaseFuture;

/// Strict management operations for the seeded singleton national boundary.
pub trait ManageIndonesiaBoundary: Send + Sync {
    fn update_indonesia_boundary<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateIndonesiaBoundary,
    ) -> UsecaseFuture<'a, IndonesiaBoundary>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateIndonesiaBoundary {
    pub boundary: Option<Boundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<Option<NaiveDate>>,
}
