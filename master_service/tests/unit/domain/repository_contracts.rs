use master_service::domain::contracts::repository::{
    CityRepository, PollingMethodRepository, ProvinceRepository, RepositoryFuture, TopicRepository,
};

#[test]
fn repository_contracts_are_object_safe_and_futures_are_send() {
    fn accept(
        _: &dyn TopicRepository,
        _: &dyn PollingMethodRepository,
        _: &dyn ProvinceRepository,
        _: &dyn CityRepository,
    ) {
    }
    fn future_is_send(_: RepositoryFuture<'static, ()>) {}

    let _ = accept;
    let _ = future_is_send;
}
