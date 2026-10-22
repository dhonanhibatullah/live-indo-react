use crate::domain::models::AppContext;

use super::super::UsecaseFuture;

/// Check a WGS84 coordinate against the national boundary.
pub trait IsInsideIndonesia: Send + Sync {
    fn is_inside_indonesia<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> UsecaseFuture<'a, bool>;
}
