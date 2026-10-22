use super::AppTransaction;
use uuid::Uuid;

/// Request-scoped metadata propagated through contracts and usecases.
///
/// The actor is an opaque caller value, not an authenticated identity. It may be
/// absent and must not be treated as proof of authorization. Trace and execution
/// identifiers are optional correlation values supplied by the transport or
/// application boundary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppContext {
    pub actor: Option<String>,
    pub trace_id: Option<Uuid>,
    pub execution_id: Option<Uuid>,
    pub transaction: Option<AppTransaction>,
}
