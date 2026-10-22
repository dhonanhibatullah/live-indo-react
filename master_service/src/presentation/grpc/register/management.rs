use std::sync::Arc;

use tonic::service::Routes;

use crate::domain::usecases::management::{
    ManageCity, ManageIndonesiaBoundary, ManagePollingMethod, ManageProvince, ManageTopic,
};
use crate::presentation::grpc::server::management::{
    city, indonesia_boundary, polling_method, province, topic,
};

pub struct ManagementDependencies {
    pub topics: Arc<dyn ManageTopic>,
    pub polling_methods: Arc<dyn ManagePollingMethod>,
    pub provinces: Arc<dyn ManageProvince>,
    pub cities: Arc<dyn ManageCity>,
    pub indonesia_boundary: Arc<dyn ManageIndonesiaBoundary>,
}

pub fn management(routes: Routes, dependencies: ManagementDependencies) -> Routes {
    routes
        .add_service(topic::server(dependencies.topics))
        .add_service(polling_method::server(dependencies.polling_methods))
        .add_service(province::server(dependencies.provinces))
        .add_service(city::server(dependencies.cities))
        .add_service(indonesia_boundary::server(dependencies.indonesia_boundary))
}
