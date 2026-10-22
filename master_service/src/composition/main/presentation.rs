use axum::Router;
use tonic::service::Routes;

use crate::composition::main::application::Application;

use crate::presentation::{
    grpc::register::{
        self, Dependencies as GrpcDependencies, GeographicDependencies, ManagementDependencies,
        TopicDependencies,
    },
    http::{Dependencies as HttpDependencies, routes::router},
};

pub fn http(app: &Application) -> Router {
    router(HttpDependencies {
        create_app_context: app.create_app_context.clone(),
        manage_topic: app.manage_topic.clone(),
        manage_polling_method: app.manage_polling_method.clone(),
        manage_province: app.manage_province.clone(),
        manage_city: app.manage_city.clone(),
        manage_indonesia_boundary: app.manage_indonesia_boundary.clone(),
        get_province: app.get_province.clone(),
        get_city: app.get_city.clone(),
        get_region: app.get_region.clone(),
        is_inside_indonesia: app.is_inside_indonesia.clone(),
        polling_method_catalog: app.polling_method_catalog.clone(),
        topic_catalog: app.topic_catalog.clone(),
    })
}

pub fn grpc(app: &Application) -> Routes {
    register::routes(GrpcDependencies {
        management: ManagementDependencies {
            topics: app.manage_topic.clone(),
            polling_methods: app.manage_polling_method.clone(),
            provinces: app.manage_province.clone(),
            cities: app.manage_city.clone(),
            indonesia_boundary: app.manage_indonesia_boundary.clone(),
        },
        geographic: GeographicDependencies {
            provinces: app.get_province.clone(),
            cities: app.get_city.clone(),
            regions: app.get_region.clone(),
            indonesia: app.is_inside_indonesia.clone(),
        },
        topic: TopicDependencies {
            polling_methods: app.polling_method_catalog.clone(),
            topics: app.topic_catalog.clone(),
        },
    })
}
