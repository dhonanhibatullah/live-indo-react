use crate::domain::models::AppContext;

use super::{CacheKind, CacheValue};

/// A best-effort cache for master-data reads.
///
/// Implementations treat missing, expired, and mismatched entries as misses.
/// Cache failures are not part of the domain error vocabulary. Callers must not
/// cache data from an active transaction.
pub trait Cache: Send + Sync {
    /// Return a cached value. Absent and expired entries both return `None`.
    fn get(&self, context: &AppContext, key: &str) -> Option<CacheValue>;

    /// Insert or replace a value. The adapter applies the TTL for `kind`.
    fn set(&self, context: &AppContext, key: &str, kind: CacheKind, value: CacheValue);

    /// Remove an exact key. Missing keys are ignored.
    fn remove(&self, context: &AppContext, key: &str);
}
