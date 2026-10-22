use master_service::domain::{
    contracts::repository::{
        CityRepository, IndonesiaBoundaryRepository, PollingMethodRepository, ProvinceRepository,
        TopicRepository,
    },
    usecases::{
        UsecaseFuture,
        geographic::{GetCity, GetProvince, GetRegionByCoord, IsInsideIndonesia},
        management::{
            ManageCity, ManageIndonesiaBoundary, ManagePollingMethod, ManageProvince, ManageTopic,
        },
        topic::{PollingMethodCatalog, TopicCatalog},
    },
};

#[test]
fn usecase_contracts_are_object_safe_and_futures_are_send() {
    fn accept_management(
        _: &dyn ManageTopic,
        _: &dyn ManagePollingMethod,
        _: &dyn ManageProvince,
        _: &dyn ManageCity,
        _: &dyn ManageIndonesiaBoundary,
    ) {
    }
    fn accept_geographic(
        _: &dyn GetProvince,
        _: &dyn GetCity,
        _: &dyn GetRegionByCoord,
        _: &dyn IsInsideIndonesia,
    ) {
    }
    fn accept_topic(_: &dyn PollingMethodCatalog, _: &dyn TopicCatalog) {}
    fn accept_repositories(
        _: &dyn TopicRepository,
        _: &dyn PollingMethodRepository,
        _: &dyn ProvinceRepository,
        _: &dyn CityRepository,
        _: &dyn IndonesiaBoundaryRepository,
    ) {
    }
    fn future_is_send(_: UsecaseFuture<'static, ()>) {}

    let _ = accept_management;
    let _ = accept_geographic;
    let _ = accept_topic;
    let _ = accept_repositories;
    let _ = future_is_send;
}
