use std::future::Future;

use crate::{Pgdt, Transaction, TransactionContext, lifecycle::Scope};

#[derive(Clone, Debug)]
pub struct Transactor {
    database: Pgdt,
}

impl Transactor {
    pub fn new(database: Pgdt) -> Self {
        Self { database }
    }

    /// Commits successful callbacks and rolls back failed callbacks. Nested calls
    /// reuse the outer scope: propagate an inner error if the owner should roll back.
    /// Panics and cancellation invalidate the handle and release the transaction.
    pub async fn with_tx<C, F, Fut, T, E>(&self, context: &C, operation: F) -> Result<T, E>
    where
        C: TransactionContext,
        F: FnOnce(C) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sqlx::Error> + Send,
    {
        if self
            .database
            .transaction(context)
            .map_err(E::from)?
            .is_some()
        {
            return operation(context.clone()).await;
        }
        let tx = self.database.pool().begin().await.map_err(E::from)?;
        let transaction = Transaction::new(self.database.owner(), tx);
        // Arm cleanup before asking the consumer to construct its derived context.
        let mut scope = Scope::new(transaction.clone());
        let context = context.with_transaction(transaction);
        // Guard against incorrect context implementations losing/replacing the handle.
        if context.transaction() != Some(&scope.transaction) {
            return Err(E::from(sqlx::Error::Protocol(
                "context did not attach the supplied transaction".into(),
            )));
        }
        match operation(context).await {
            Ok(value) => {
                scope.finish(true).await.map_err(E::from)?;
                Ok(value)
            }
            Err(error) => {
                let _ = scope.finish(false).await;
                Err(error)
            }
        }
    }
}
