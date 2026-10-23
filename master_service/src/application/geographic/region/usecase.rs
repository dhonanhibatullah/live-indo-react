use std::{net::IpAddr, sync::Arc};

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{CityRepository, ProvinceRepository},
            service::GeoIpService,
            utility::Logger,
        },
        models::{AppContext, GeoIpError, RepositoryError, logger::Meta},
        usecases::{
            UsecaseFuture,
            geographic::{GetRegion, Region, RegionByIp},
        },
    },
};

pub struct RegionQueries {
    cities: Arc<dyn CityRepository>,
    provinces: Arc<dyn ProvinceRepository>,
    logger: Arc<dyn Logger>,
    geoip: Option<Arc<dyn GeoIpService>>,
}

impl RegionQueries {
    pub fn new(
        cities: Arc<dyn CityRepository>,
        provinces: Arc<dyn ProvinceRepository>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            cities,
            provinces,
            logger,
            geoip: None,
        }
    }

    /// Enable IP resolution with an explicit service dependency. Without this,
    /// coordinate lookup remains available and IP lookup returns GEOIP_NOT_READY.
    pub fn with_geoip(mut self, geoip: Arc<dyn GeoIpService>) -> Self {
        self.geoip = Some(geoip);
        self
    }
}

impl GetRegion for RegionQueries {
    fn get_region_by_ip<'a>(
        &'a self,
        context: &'a AppContext,
        ip: IpAddr,
    ) -> UsecaseFuture<'a, RegionByIp> {
        Box::pin(async move {
            let Some(service) = &self.geoip else {
                let error = GeoIpError::NotReady;
                let meta = Meta::from([("error_code".into(), error.code().into())]);
                self.logger.error(
                    context,
                    "geographic/region/get_by_ip",
                    "GeoIP service is not configured",
                    Some(&meta),
                );
                return Err(error.into());
            };
            let geoip = service.get_location(context, ip).await?;
            let region = match geoip.as_ref().and_then(|location| location.coordinates) {
                Some(point) => {
                    self.get_region_by_coord(context, point.longitude, point.latitude)
                        .await?
                }
                None => None,
            };
            Ok(RegionByIp { geoip, region })
        })
    }

    fn get_region_by_coord<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> UsecaseFuture<'a, Option<Region>> {
        Box::pin(async move {
            if let Some(city) = self
                .cities
                .find_city_by_coord(context, longitude, latitude)
                .await?
            {
                let province = self.provinces.find_by_id(context, city.province_id).await?;
                let Some(province) = province else {
                    let error = RepositoryError::ProvinceNotFound;
                    log_repository_error(
                        self.logger.as_ref(),
                        context,
                        "geographic/region/get_by_coord",
                        &error,
                    );
                    return Err(error.into());
                };
                return Ok(Some(Region {
                    province,
                    city: Some(city),
                }));
            }

            let province = self
                .provinces
                .find_province_by_coord(context, longitude, latitude)
                .await?;
            Ok(province.map(|province| Region {
                province,
                city: None,
            }))
        })
    }
}
