pub mod city;
pub mod polling_method;
pub mod province;
pub mod topic;

/// Offset pagination shared by list operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}
