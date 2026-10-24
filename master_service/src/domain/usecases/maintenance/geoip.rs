use crate::domain::models::AppContext;

use super::super::UsecaseFuture;

/// Ensure the local GeoIP database is ready and current.
pub trait RefreshGeoIp: Send + Sync {
    /// Update immediately when unready; otherwise check for a newer release first.
    /// Both an already-current database and a successful refresh return unit.
    /// Provider failures preserve their codes and sources. Scheduling, retries and
    /// replacement safety belong to callers and the service adapter respectively.
    fn refresh_geoip<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()>;
}
