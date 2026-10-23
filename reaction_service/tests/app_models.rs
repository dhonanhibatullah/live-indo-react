use reaction_service::domain::models::{AppContext, AppInfo, AppTransaction};
use uuid::Uuid;

#[test]
fn transaction_clones_share_handle_identity() {
    let transaction = AppTransaction::new(7_u64);
    let clone = transaction.clone();
    let other = AppTransaction::new(7_u64);

    assert_eq!(transaction, clone);
    assert_ne!(transaction, other);
    assert_eq!(transaction.downcast_ref::<u64>(), Some(&7));
    assert_eq!(transaction.downcast_ref::<String>(), None);
}

#[test]
fn app_context_equality_uses_transaction_identity() {
    let transaction = AppTransaction::new("transaction");
    let context = AppContext {
        actor: Some("anonymous".into()),
        trace_id: Some(Uuid::nil()),
        execution_id: Some(Uuid::from_u128(1)),
        transaction: Some(transaction.clone()),
    };
    let same_transaction = AppContext {
        transaction: Some(transaction),
        ..context.clone()
    };
    let different_transaction = AppContext {
        transaction: Some(AppTransaction::new("transaction")),
        ..context.clone()
    };

    assert_eq!(context, same_transaction);
    assert_ne!(context, different_transaction);
}

#[test]
fn app_info_exposes_static_metadata() {
    let info = AppInfo {
        name: "reaction_service",
        version: "0.1.0-dev.1",
    };

    assert_eq!(info.name, "reaction_service");
    assert_eq!(info.version, "0.1.0-dev.1");
}
