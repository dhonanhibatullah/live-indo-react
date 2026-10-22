use master_service::domain::contracts::caching::{Cache, CacheValue};

#[test]
fn cache_value_round_trips_and_rejects_other_types() {
    let value = CacheValue::new(vec![1_u8, 2, 3]);

    assert_eq!(
        value
            .downcast::<Vec<u8>>()
            .expect("cached vector")
            .as_slice(),
        &[1, 2, 3]
    );
    assert!(value.downcast::<String>().is_none());
}

#[test]
fn cache_contract_is_object_safe() {
    fn accept(_: &dyn Cache) {}

    let _ = accept;
}
