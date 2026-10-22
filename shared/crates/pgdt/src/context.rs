use crate::Transaction;

/// A service-owned context that can carry a transaction without losing its metadata.
/// `with_transaction` must return a copy with exactly the supplied handle attached.
pub trait TransactionContext: Clone + Send + Sync {
    fn transaction(&self) -> Option<&Transaction>;
    fn with_transaction(&self, transaction: Transaction) -> Self;
}
