use std::{future::Future, net::IpAddr, pin::Pin};

use crate::domain::models::{AppContext, entity::GeoIp, error::GeoIpError};

/// Local lookup with explicit remote database maintenance. Object-safe for injection.
/// No automatic scheduler, country admission policy or transaction participation.
pub trait GeoIpService: Send + Sync {
    /// True when a validated local City reader is loaded; no network access.
    fn is_ready<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool>;
    /// True when no usable local database exists or the remote release changed.
    /// Provider failures are errors, not a false "already current" result.
    fn check_for_update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool>;
    /// Download if absent/outdated, validate, replace atomically and reload.
    /// A failed update preserves the usable database. Serializes updates per
    /// adapter instance. Cancellation before installation leaves it unchanged;
    /// an already-started atomic installation may finish after cancellation.
    fn update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, ()>;
    /// None means the IP has no database record. NotReady means no reader is loaded.
    fn get_location<'a>(
        &'a self,
        context: &'a AppContext,
        ip: IpAddr,
    ) -> GeoIpFuture<'a, Option<GeoIp>>;
}

pub type GeoIpFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, GeoIpError>> + Send + 'a>>;
