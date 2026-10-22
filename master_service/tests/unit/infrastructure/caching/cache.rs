use std::time::Duration;

use master_service::{
    domain::{
        contracts::caching::{Cache, CacheKind, CacheValue},
        models::AppContext,
    },
    infrastructure::caching::MokaCache,
};

#[test]
fn moka_cache_stores_overwrites_and_removes_values() {
    let cache = MokaCache::new(16, Duration::from_secs(60), Duration::from_secs(5));
    let context = AppContext::default();

    assert!(cache.get(&context, "key").is_none());

    cache.set(
        &context,
        "key",
        CacheKind::Individual,
        CacheValue::new("first".to_owned()),
    );
    assert_eq!(
        cache
            .get(&context, "key")
            .expect("cached value")
            .downcast::<String>()
            .expect("string value")
            .as_str(),
        "first"
    );

    cache.set(
        &context,
        "key",
        CacheKind::List,
        CacheValue::new(vec!["second".to_owned()]),
    );
    assert_eq!(
        cache
            .get(&context, "key")
            .expect("cached list")
            .downcast::<Vec<String>>()
            .expect("list value")
            .as_slice(),
        &["second".to_owned()]
    );

    cache.remove(&context, "key");
    assert!(cache.get(&context, "key").is_none());
}

#[test]
fn cached_values_are_type_checked() {
    let cache = MokaCache::new(16, Duration::from_secs(60), Duration::from_secs(5));
    let context = AppContext::default();

    cache.set(
        &context,
        "key",
        CacheKind::Individual,
        CacheValue::new(7_u64),
    );

    let value = cache.get(&context, "key").expect("cached value");
    assert_eq!(*value.downcast::<u64>().expect("u64 value"), 7);
    assert!(value.downcast::<String>().is_none());
}
