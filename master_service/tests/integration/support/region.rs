use std::{net::IpAddr, sync::Mutex};

use chrono::{DateTime, Utc};
use master_service::domain::{
    models::{
        AppContext, GeoIpError,
        entity::{City, CityKind, GeoIp, GeoIpCoordinates, GeoIpSubdivision, Province},
    },
    usecases::{
        UsecaseFuture,
        geographic::{GetRegion, Region, RegionByIp},
    },
};

#[derive(Clone, Copy)]
pub(crate) enum Failure {
    Configuration,
    NotReady,
    Database,
    Download,
    Unavailable,
    Timeout,
    Operation,
}
impl Failure {
    pub(crate) fn error(self) -> GeoIpError {
        let source = || Box::new(std::io::Error::other("private-provider-secret"));
        match self {
            Self::Configuration => GeoIpError::InvalidConfiguration,
            Self::NotReady => GeoIpError::NotReady,
            Self::Database => GeoIpError::InvalidDatabase { source: source() },
            Self::Download => GeoIpError::InvalidDownload { source: source() },
            Self::Unavailable => GeoIpError::Unavailable,
            Self::Timeout => GeoIpError::Timeout { source: source() },
            Self::Operation => GeoIpError::Failure { source: source() },
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Full,
    EstimateOnly,
    NoCoordinates,
    Unknown,
    Error(Failure),
}

pub(crate) struct RegionUsecase {
    mode: Mutex<Mode>,
    pub calls: Mutex<Vec<(AppContext, IpAddr)>>,
}
impl RegionUsecase {
    pub fn new() -> Self {
        Self {
            mode: Mutex::new(Mode::Full),
            calls: Mutex::new(Vec::new()),
        }
    }
    pub fn set(&self, mode: Mode) {
        *self.mode.lock().unwrap() = mode;
    }
}
impl GetRegion for RegionUsecase {
    fn get_region_by_ip<'a>(
        &'a self,
        context: &'a AppContext,
        ip: IpAddr,
    ) -> UsecaseFuture<'a, RegionByIp> {
        self.calls.lock().unwrap().push((context.clone(), ip));
        let mode = *self.mode.lock().unwrap();
        Box::pin(async move {
            if let Mode::Error(error) = mode {
                return Err(error.error().into());
            }
            let geoip = if matches!(mode, Mode::Unknown) {
                None
            } else {
                Some(GeoIp {
                    continent_code: Some("AS".into()),
                    country_code: Some("ID".into()),
                    country_name: Some("Indonesia".into()),
                    subdivisions: vec![GeoIpSubdivision {
                        code: Some("JK".into()),
                        name: Some("Jakarta".into()),
                    }],
                    city_name: Some("Jakarta".into()),
                    postal_code: None,
                    coordinates: (!matches!(mode, Mode::NoCoordinates)).then_some(
                        GeoIpCoordinates {
                            longitude: 106.8,
                            latitude: -6.2,
                        },
                    ),
                    accuracy_radius_km: Some(10),
                    time_zone: Some("Asia/Jakarta".into()),
                })
            };
            Ok(RegionByIp {
                geoip,
                region: matches!(mode, Mode::Full).then(region),
            })
        })
    }
    fn get_region_by_coord<'a>(
        &'a self,
        _: &'a AppContext,
        _: f64,
        _: f64,
    ) -> UsecaseFuture<'a, Option<Region>> {
        Box::pin(async { Ok(Some(region())) })
    }
}

fn region() -> Region {
    let time = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    Region {
        province: Province {
            id: 31,
            kemendagri_code: "31".into(),
            name: "Jakarta".into(),
            boundary: super::fixtures::boundary(),
            boundary_source: "fixture".into(),
            boundary_reference_date: None,
            is_active: true,
            created_at: time,
            updated_at: None,
            deleted_at: None,
        },
        city: Some(City {
            id: 3101,
            province_id: 31,
            kemendagri_code: "31.01".into(),
            kind: CityKind::City,
            name: "Jakarta fixture city".into(),
            boundary: super::fixtures::boundary(),
            boundary_source: "fixture".into(),
            boundary_reference_date: None,
            is_active: true,
            created_at: time,
            updated_at: None,
            deleted_at: None,
        }),
    }
}
