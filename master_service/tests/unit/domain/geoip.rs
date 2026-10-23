use std::{error::Error, io};

use master_service::domain::{
    contracts::service::GeoIpService,
    models::{
        AppContext,
        entity::{GeoIp, GeoIpCoordinates},
        error::GeoIpError,
    },
};

fn send<T: Send>(_: T) {}

#[test]
fn contract_is_object_safe_and_operations_return_send_futures() {
    fn accepts_service(service: &dyn GeoIpService, context: &AppContext) {
        send(service.is_ready(context));
        send(service.check_for_update(context));
        send(service.update(context));
        send(service.get_location(context, "203.0.113.1".parse().unwrap()));
    }
    let _: fn(&dyn GeoIpService, &AppContext) = accepts_service;
}

#[test]
fn error_codes_are_stable_and_operational_causes_are_preserved_without_rendering_them() {
    let cause = || Box::new(io::Error::other("private-provider-detail"));
    for (error, code, has_source) in [
        (
            GeoIpError::InvalidConfiguration,
            "GEOIP_CONFIGURATION_INVALID",
            false,
        ),
        (GeoIpError::NotReady, "GEOIP_NOT_READY", false),
        (
            GeoIpError::InvalidDatabase { source: cause() },
            "GEOIP_DATABASE_INVALID",
            true,
        ),
        (
            GeoIpError::InvalidDownload { source: cause() },
            "GEOIP_DOWNLOAD_INVALID",
            true,
        ),
        (GeoIpError::Unavailable, "GEOIP_UNAVAILABLE", false),
        (
            GeoIpError::Timeout { source: cause() },
            "GEOIP_TIMEOUT",
            true,
        ),
        (
            GeoIpError::Failure { source: cause() },
            "GEOIP_FAILURE",
            true,
        ),
    ] {
        assert_eq!(error.code(), code);
        assert!(!error.to_string().contains("private-provider-detail"));
        assert!(!format!("{error:?}").contains("private-provider-detail"));
        if has_source {
            let native = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
            assert_eq!(native.to_string(), "private-provider-detail");
        } else {
            assert!(error.source().is_none());
        }
    }
}

#[test]
fn debug_output_omits_location_values() {
    let point = GeoIpCoordinates {
        longitude: 106.812345,
        latitude: -6.234567,
    };
    let location = GeoIp {
        continent_code: None,
        country_code: Some("ID".into()),
        country_name: None,
        subdivisions: Vec::new(),
        city_name: Some("private-city-name".into()),
        postal_code: Some("private-postal-code".into()),
        coordinates: Some(point),
        accuracy_radius_km: None,
        time_zone: None,
    };
    assert!(!format!("{location:?}").contains("private-"));
    assert!(!format!("{point:?}").contains("106.812345"));
    assert!(!format!("{point:?}").contains("-6.234567"));
}
