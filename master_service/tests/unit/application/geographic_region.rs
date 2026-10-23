use std::{
    error::Error,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use master_service::{
    application::geographic::{IndonesiaQueries, RegionQueries},
    domain::{
        contracts::{
            repository::ProvinceRepository,
            service::{GeoIpFuture, GeoIpService},
        },
        models::{
            AppContext, GeoIpError,
            entity::{GeoIp, GeoIpCoordinates},
        },
        usecases::geographic::{GetRegion, IsInsideIndonesia},
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

struct FakeGeoIp {
    outcome: Mutex<Option<Result<Option<GeoIp>, GeoIpError>>>,
}
impl GeoIpService for FakeGeoIp {
    fn is_ready<'a>(&'a self, _: &'a AppContext) -> GeoIpFuture<'a, bool> {
        panic!("region resolution must not perform maintenance")
    }
    fn check_for_update<'a>(&'a self, _: &'a AppContext) -> GeoIpFuture<'a, bool> {
        panic!("region resolution must not perform maintenance")
    }
    fn update<'a>(&'a self, _: &'a AppContext) -> GeoIpFuture<'a, ()> {
        panic!("region resolution must not perform maintenance")
    }
    fn get_location<'a>(&'a self, _: &'a AppContext, ip: IpAddr) -> GeoIpFuture<'a, Option<GeoIp>> {
        assert_eq!(ip, "203.0.113.1".parse::<IpAddr>().unwrap());
        let outcome = self.outcome.lock().unwrap().take().unwrap();
        Box::pin(async move { outcome })
    }
}
fn location(coordinates: Option<GeoIpCoordinates>) -> GeoIp {
    GeoIp {
        continent_code: None,
        country_code: Some("ID".into()),
        country_name: Some("Indonesia".into()),
        subdivisions: Vec::new(),
        city_name: Some("unmapped-provider-name".into()),
        postal_code: None,
        coordinates,
        accuracy_radius_km: Some(100),
        time_zone: None,
    }
}
fn geoip(outcome: Result<Option<GeoIp>, GeoIpError>) -> Arc<dyn GeoIpService> {
    Arc::new(FakeGeoIp {
        outcome: Mutex::new(Some(outcome)),
    })
}

#[tokio::test]
async fn ip_lookup_reuses_coordinate_resolution_and_preserves_estimate_and_context() {
    let point = GeoIpCoordinates {
        longitude: 106.5,
        latitude: -6.5,
    };
    let estimate = location(Some(point));
    let cities = Arc::new(FakeCityRepository::new(Some(city())));
    let service = RegionQueries::new(
        cities.clone(),
        Arc::new(FakeProvinceRepository::new(province())),
        Arc::new(CapturingLogger::default()),
    )
    .with_geoip(geoip(Ok(Some(estimate.clone()))));
    let context = AppContext {
        actor: Some("test-caller".into()),
        ..AppContext::default()
    };
    let result = service
        .get_region_by_ip(&context, "203.0.113.1".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(result.geoip, Some(estimate));
    let region = result.region.unwrap();
    assert_eq!(region.province.kemendagri_code, "11");
    assert_eq!(region.city.unwrap().kemendagri_code, "11.01");
    assert_eq!(
        *cities.last_coord.lock().unwrap(),
        Some((context, point.longitude, point.latitude))
    );
}

#[tokio::test]
async fn missing_coordinates_and_unknown_ips_do_not_guess_regions_or_query_boundaries() {
    for estimate in [Some(location(None)), None] {
        let cities = Arc::new(FakeCityRepository::new(Some(city())));
        let service = RegionQueries::new(
            cities.clone(),
            Arc::new(FakeProvinceRepository::new(province())),
            Arc::new(CapturingLogger::default()),
        )
        .with_geoip(geoip(Ok(estimate.clone())));
        let result = service
            .get_region_by_ip(&AppContext::default(), "203.0.113.1".parse().unwrap())
            .await
            .unwrap();
        assert_eq!(result.geoip, estimate);
        assert!(result.region.is_none());
        assert!(cities.last_coord.lock().unwrap().is_none());
    }
}

#[tokio::test]
async fn ip_lookup_supports_province_only_and_unassigned_points() {
    for matched in [true, false] {
        let provinces = Arc::new(FakeProvinceRepository::new(province()));
        *provinces.by_coord.lock().unwrap() = matched.then(province);
        let estimate = location(Some(GeoIpCoordinates {
            longitude: 106.5,
            latitude: -6.5,
        }));
        let service = RegionQueries::new(
            Arc::new(FakeCityRepository::new(None)),
            provinces,
            Arc::new(CapturingLogger::default()),
        )
        .with_geoip(geoip(Ok(Some(estimate.clone()))));
        let result = service
            .get_region_by_ip(&AppContext::default(), "203.0.113.1".parse().unwrap())
            .await
            .unwrap();
        assert_eq!(result.geoip, Some(estimate));
        assert_eq!(result.region.is_some(), matched);
        if let Some(region) = result.region {
            assert!(region.city.is_none());
        }
    }
}

#[tokio::test]
async fn geoip_failures_preserve_codes_and_sources_without_duplicate_logs() {
    let logger = Arc::new(CapturingLogger::default());
    let service = RegionQueries::new(
        Arc::new(FakeCityRepository::new(Some(city()))),
        Arc::new(FakeProvinceRepository::new(province())),
        logger.clone(),
    )
    .with_geoip(geoip(Err(GeoIpError::Failure {
        source: Box::new(std::io::Error::other("native-cause")),
    })));
    let error = service
        .get_region_by_ip(&AppContext::default(), "203.0.113.1".parse().unwrap())
        .await
        .unwrap_err();
    assert_eq!(error.code(), "GEOIP_FAILURE");
    let category = error
        .source()
        .unwrap()
        .downcast_ref::<GeoIpError>()
        .unwrap();
    assert_eq!(category.source().unwrap().to_string(), "native-cause");
    assert!(!format!("{error:?}").contains("native-cause"));
    assert!(logger.records().is_empty());
}

#[tokio::test]
async fn unconfigured_geoip_is_not_ready_while_coordinate_lookup_still_works() {
    let logger = Arc::new(CapturingLogger::default());
    let service = RegionQueries::new(
        Arc::new(FakeCityRepository::new(Some(city()))),
        Arc::new(FakeProvinceRepository::new(province())),
        logger.clone(),
    );
    let context = AppContext::default();
    let error = service
        .get_region_by_ip(&context, "203.0.113.1".parse().unwrap())
        .await
        .unwrap_err();
    assert_eq!(error.code(), "GEOIP_NOT_READY");
    assert_eq!(logger.records().len(), 1);
    assert!(
        service
            .get_region_by_coord(&context, 106.5, -6.5)
            .await
            .unwrap()
            .is_some()
    );
}
