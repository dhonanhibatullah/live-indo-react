use std::sync::Arc;

use master_service::{
    application::geographic::{IndonesiaQueries, RegionQueries},
    domain::{
        contracts::repository::ProvinceRepository,
        models::AppContext,
        usecases::geographic::{GetRegionByCoord, IsInsideIndonesia},
    },
};

use super::support::{
    CapturingLogger, FakeCityRepository, FakeIndonesiaBoundaryRepository, FakeProvinceRepository,
    city, province,
};

#[tokio::test]
async fn region_returns_city_and_parent_when_city_contains_coordinate() {
    let cities = Arc::new(FakeCityRepository::new(Some(city())));
    let provinces = Arc::new(FakeProvinceRepository::new(province()));
    let service = RegionQueries::new(cities, provinces, Arc::new(CapturingLogger::default()));

    let region = service
        .get_region_by_coord(&AppContext::default(), 106.5, -6.5)
        .await
        .expect("resolve region")
        .expect("region exists");
    assert_eq!(region.province.id, 1);
    assert_eq!(region.city.expect("city").id, 1);
}

#[tokio::test]
async fn region_falls_back_to_province_and_can_be_absent() {
    let cities = Arc::new(FakeCityRepository::new(None));
    let provinces = Arc::new(FakeProvinceRepository::new(province()));
    *provinces.by_coord.lock().expect("coord lock") = Some(province());
    let province_repository: Arc<dyn ProvinceRepository> = provinces.clone();
    let service = RegionQueries::new(
        cities,
        province_repository,
        Arc::new(CapturingLogger::default()),
    );

    let region = service
        .get_region_by_coord(&AppContext::default(), 106.5, -6.5)
        .await
        .expect("resolve region")
        .expect("province exists");
    assert!(region.city.is_none());

    *provinces.by_coord.lock().expect("coord lock") = None;
    assert!(
        service
            .get_region_by_coord(&AppContext::default(), 0.0, 0.0)
            .await
            .expect("resolve missing region")
            .is_none()
    );
}

#[tokio::test]
async fn indonesia_query_delegates_to_the_national_boundary() {
    let boundary = Arc::new(FakeIndonesiaBoundaryRepository::new(true));
    let service = IndonesiaQueries::new(boundary);

    assert!(
        service
            .is_inside_indonesia(&AppContext::default(), 110.0, 0.0)
            .await
            .expect("check inside")
    );
}
