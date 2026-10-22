use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

/// Versioned cache keys and invalidation state for one master-data entity.
///
/// A successful write advances the generation. Earlier keys become unreachable
/// without a scan, and their entries expire through the adapter's TTL and
/// capacity policy. Each decorated repository owns one namespace.
pub(crate) struct CacheNamespace {
    entity: &'static str,
    generation: AtomicU64,
}

impl CacheNamespace {
    pub(crate) fn new(entity: &'static str) -> Self {
        Self {
            entity,
            generation: AtomicU64::new(0),
        }
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub(crate) fn invalidation_guard(&self) -> InvalidationGuard<'_> {
        InvalidationGuard { namespace: self }
    }

    pub(crate) fn id_key(&self, generation: u64, id: i64) -> String {
        format!("master:v1:{}:g{}:id:{}", self.entity, generation, id)
    }

    pub(crate) fn text_key(&self, generation: u64, kind: &str, value: &str) -> String {
        format!(
            "master:v1:{}:g{}:{}:{}",
            self.entity, generation, kind, value
        )
    }

    pub(crate) fn hash_key<T: Hash + ?Sized>(
        &self,
        generation: u64,
        kind: &str,
        value: &T,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);

        format!(
            "master:v1:{}:g{}:{}:{:016x}",
            self.entity,
            generation,
            kind,
            hasher.finish()
        )
    }

    fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}

/// Invalidates an entity namespace when dropped.
///
/// Keeping this guard across the repository write future also covers
/// cancellation: a dropped write future still makes cached reads unreachable.
/// Failed writes may invalidate unnecessarily, which is safe.
pub(crate) struct InvalidationGuard<'a> {
    namespace: &'a CacheNamespace,
}

impl Drop for InvalidationGuard<'_> {
    fn drop(&mut self) {
        self.namespace.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_guard_advances_generation_on_drop() {
        let namespace = CacheNamespace::new("topic");
        let before = namespace.generation();

        {
            let _guard = namespace.invalidation_guard();
        }

        assert_eq!(namespace.generation(), before + 1);
    }

    #[test]
    fn keys_are_scoped_by_entity_and_generation() {
        let topic = CacheNamespace::new("topic");
        let city = CacheNamespace::new("city");
        let id = topic.id_key(1, 7);
        let text = topic.text_key(1, "slug", "example");
        let list = topic.hash_key(1, "list", &"filter");

        assert!(id.contains("topic"));
        assert!(id.contains("g1"));
        assert!(text.contains("slug"));
        assert!(list.contains("list"));
        assert_ne!(topic.id_key(1, 7), city.id_key(1, 7));
        assert_ne!(topic.id_key(1, 7), topic.id_key(2, 7));
    }
}
