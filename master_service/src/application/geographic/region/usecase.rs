use std::sync::Arc;

use crate::{
    application::shared::log_repository_error,
    domain::{
        contracts::{
            repository::{CityRepository, ProvinceRepository},
            utility::Logger,
        },
        models::{AppContext, RepositoryError},
        usecases::{
            UsecaseFuture,
            geographic::{GetRegionByCoord, Region},
        },
    },
};

pub struct RegionQueries {
    cities: Arc<dyn CityRepository>,
    provinces: Arc<dyn ProvinceRepository>,
    logger: Arc<dyn Logger>,
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
        }
    }
}

impl GetRegionByCoord for RegionQueries {
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
