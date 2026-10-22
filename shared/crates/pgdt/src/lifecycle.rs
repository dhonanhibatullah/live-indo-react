use crate::{Transaction, transaction::inactive};

pub(crate) struct Scope {
    pub(crate) transaction: Transaction,
    armed: bool,
}

impl Scope {
    pub(crate) fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            armed: true,
        }
    }

    pub(crate) async fn finish(&mut self, commit: bool) -> Result<(), sqlx::Error> {
        self.transaction.close();
        let tx = self
            .transaction
            .state
            .transaction
            .lock()
            .await
            .take()
            .ok_or_else(inactive)?;
        let result = if commit {
            tx.commit().await
        } else {
            tx.rollback().await
        };
        // SQLx also queues rollback on drop when completion fails.
        self.armed = false;
        result
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        self.transaction.close();
        let state = &self.transaction.state;
        if let Ok(mut guard) = state.transaction.try_lock() {
            drop(guard.take());
            return;
        }
        let state = state.clone();
        state.runtime.clone().spawn(async move {
            if let Some(tx) = state.transaction.lock().await.take() {
                let _ = tx.rollback().await;
            }
        });
    }
}
