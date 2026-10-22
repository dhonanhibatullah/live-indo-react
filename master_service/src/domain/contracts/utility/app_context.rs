use uuid::Uuid;

use crate::domain::models::AppContext;

/// Build request-scoped context from transport-supplied actor and trace values.
///
/// The actor is an opaque caller value, not an authenticated identity. The
/// implementation generates a service-local execution ID and leaves the
/// transaction unset.
pub trait AppContextFactory: Send + Sync {
    fn create(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext;
}
