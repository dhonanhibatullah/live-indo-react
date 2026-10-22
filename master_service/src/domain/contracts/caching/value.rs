use std::{any::Any, sync::Arc};

/// A type-erased value stored in the cache.
///
/// Values remain owned domain objects so the cache contract does not require
/// serialization. Callers recover the expected type through [`Self::downcast`].
#[derive(Clone)]
pub struct CacheValue(Arc<dyn Any + Send + Sync>);

impl CacheValue {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self(Arc::new(value))
    }

    /// Return the stored value when it has the requested type.
    ///
    /// A mismatched type is treated as a cache miss, never as a domain failure.
    pub fn downcast<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.0.clone().downcast::<T>().ok()
    }
}
