use std::sync::Arc;

use tonic::service::Routes;

use crate::domain::usecases::geographic::{
    GetCity, GetProvince, GetRegionByCoord, IsInsideIndonesia,
};
use crate::presentation::grpc::server::geographic::{city, indonesia, province, region};

pub struct GeographicDependencies {
    pub provinces: Arc<dyn GetProvince>,
    pub cities: Arc<dyn GetCity>,
    pub regions: Arc<dyn GetRegionByCoord>,
    pub indonesia: Arc<dyn IsInsideIndonesia>,
}

pub fn geographic(routes: Routes, dependencies: GeographicDependencies) -> Routes {
    routes
        .add_service(province::server(dependencies.provinces))
        .add_service(city::server(dependencies.cities))
        .add_service(region::server(dependencies.regions))
        .add_service(indonesia::server(dependencies.indonesia))
}
