use std::{any::Any, fmt, sync::Arc};

/// An opaque transaction handle whose concrete type belongs to infrastructure.
/// Clones share the same handle; equality compares handle identity.
#[derive(Clone)]
pub struct AppTransaction {
    inner: Arc<dyn Any + Send + Sync>,
}

impl AppTransaction {
    pub fn new<T: Any + Send + Sync>(transaction: T) -> Self {
        Self {
            inner: Arc::new(transaction),
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }
}

impl fmt::Debug for AppTransaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppTransaction")
            .finish_non_exhaustive()
    }
}

impl PartialEq for AppTransaction {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for AppTransaction {}
