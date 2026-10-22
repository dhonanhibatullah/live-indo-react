use std::sync::Arc;

use tonic::service::Routes;

use crate::domain::usecases::topic::{PollingMethodCatalog, TopicCatalog};
use crate::presentation::grpc::server::topic::{catalog, polling_method};

pub struct TopicDependencies {
    pub polling_methods: Arc<dyn PollingMethodCatalog>,
    pub topics: Arc<dyn TopicCatalog>,
}

pub fn topic(routes: Routes, dependencies: TopicDependencies) -> Routes {
    routes
        .add_service(polling_method::server(dependencies.polling_methods))
        .add_service(catalog::server(dependencies.topics))
}
