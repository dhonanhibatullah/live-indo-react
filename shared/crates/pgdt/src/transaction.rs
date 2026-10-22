use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use sqlx::Postgres;
use tokio::{runtime::Handle, sync::Mutex};

/// An opaque shared handle. Clones refer to the same transaction, not a new scope.
#[derive(Clone)]
pub struct Transaction {
    pub(crate) state: Arc<State>,
}

pub(crate) struct State {
    pub(crate) owner: Arc<()>,
    pub(crate) closed: AtomicBool,
    pub(crate) transaction: Mutex<Option<sqlx::Transaction<'static, Postgres>>>,
    pub(crate) runtime: Handle,
}

impl Transaction {
    pub(crate) fn new(owner: Arc<()>, transaction: sqlx::Transaction<'static, Postgres>) -> Self {
        Self {
            state: Arc::new(State {
                owner,
                closed: AtomicBool::new(false),
                transaction: Mutex::new(Some(transaction)),
                runtime: Handle::current(),
            }),
        }
    }

    /// False as soon as completion or cancellation starts.
    pub fn is_active(&self) -> bool {
        !self.state.closed.load(Ordering::Acquire)
    }

    pub(crate) fn close(&self) {
        self.state.closed.store(true, Ordering::Release);
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("active", &self.is_active())
            .finish_non_exhaustive()
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}
impl Eq for Transaction {}

pub(crate) fn inactive() -> sqlx::Error {
    sqlx::Error::Protocol("transaction is no longer active".into())
}
