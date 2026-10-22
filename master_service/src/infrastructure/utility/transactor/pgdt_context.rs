use pgdt::{Transaction, TransactionContext};

use crate::domain::models::{AppContext, AppTransaction};

/// Bridges the opaque domain transaction handle to the selected PostgreSQL driver.
impl TransactionContext for AppContext {
    fn transaction(&self) -> Option<&Transaction> {
        self.transaction.as_ref().map(|transaction| {
            transaction
                .downcast_ref::<Transaction>()
                .expect("AppContext transaction must contain a pgdt transaction")
        })
    }

    fn with_transaction(&self, transaction: Transaction) -> Self {
        Self {
            transaction: Some(AppTransaction::new(transaction)),
            ..self.clone()
        }
    }
}
