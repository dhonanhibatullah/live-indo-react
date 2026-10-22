mod geographic;
mod management;
mod topic;

pub use geographic::{GeographicDependencies, geographic};
pub use management::{ManagementDependencies, management};
pub use topic::{TopicDependencies, topic};

use tonic::service::Routes;

pub struct Dependencies {
    pub management: ManagementDependencies,
    pub geographic: GeographicDependencies,
    pub topic: TopicDependencies,
}

pub fn routes(dependencies: Dependencies) -> Routes {
    let routes = management(Routes::default(), dependencies.management);
    let routes = geographic(routes, dependencies.geographic);
    topic(routes, dependencies.topic)
}
