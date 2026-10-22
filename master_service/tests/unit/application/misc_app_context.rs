use std::sync::Arc;

use master_service::{
    application::misc::AppContextService,
    domain::{
        contracts::utility::AppContextFactory, models::AppContext,
        usecases::misc::app_context::CreateAppContext,
    },
};
use uuid::Uuid;

struct FakeFactory;

impl AppContextFactory for FakeFactory {
    fn create(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext {
        AppContext {
            actor,
            trace_id,
            execution_id: Some(Uuid::from_u128(7)),
            transaction: None,
        }
    }
}

#[test]
fn app_context_usecase_delegates_without_losing_context_values() {
    let service = AppContextService::new(Arc::new(FakeFactory));
    let trace_id = Some(Uuid::from_u128(9));

    let context = service.create_app_context(Some("actor".into()), trace_id);

    assert_eq!(context.actor.as_deref(), Some("actor"));
    assert_eq!(context.trace_id, trace_id);
    assert_eq!(context.execution_id, Some(Uuid::from_u128(7)));
    assert!(context.transaction.is_none());
}
