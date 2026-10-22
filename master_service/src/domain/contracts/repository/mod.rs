pub mod city;
pub mod future;
pub mod polling_method;
pub mod province;
pub mod topic;

pub use city::CityRepository;
pub use future::RepositoryFuture;
pub use polling_method::PollingMethodRepository;
pub use province::ProvinceRepository;
pub use topic::TopicRepository;

/// Offset pagination shared by list operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}
