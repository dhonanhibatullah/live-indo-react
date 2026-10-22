use uuid::Uuid;

use crate::domain::models::AppContext;

/// Presentation-facing context creation for one request.
///
/// The actor and trace ID come from transport metadata; the application
/// implementation supplies the service-local execution ID.
pub trait CreateAppContext: Send + Sync {
    fn create_app_context(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext;
}
