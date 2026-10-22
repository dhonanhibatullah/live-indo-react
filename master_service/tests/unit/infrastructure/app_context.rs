use master_service::{
    domain::contracts::utility::AppContextFactory, infrastructure::utility::UuidAppContextFactory,
};
use uuid::Uuid;

#[test]
fn uuid_factory_preserves_actor_and_trace_and_generates_unique_execution_ids() {
    let factory = UuidAppContextFactory::new();
    let actor = Some("anonymous-admin".to_owned());
    let trace_id = Some(Uuid::from_u128(42));

    let first = factory.create(actor.clone(), trace_id);
    let second = factory.create(actor, trace_id);

    assert_eq!(first.actor.as_deref(), Some("anonymous-admin"));
    assert_eq!(first.trace_id, trace_id);
    assert!(first.execution_id.is_some_and(|id| !id.is_nil()));
    assert!(first.transaction.is_none());
    assert_ne!(first.execution_id, second.execution_id);
}
