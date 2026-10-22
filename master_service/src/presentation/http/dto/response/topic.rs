use serde::Serialize;

use crate::domain::usecases::Page;

pub use super::model::{PollingMethodResponse, TopicResponse};

#[derive(Clone, Debug, Serialize)]
pub struct PollingMethodListResponse {
    pub data: Vec<PollingMethodResponse>,
    pub page: PageResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct TopicListResponse {
    pub data: Vec<TopicResponse>,
    pub page: PageResponse,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PageResponse {
    pub limit: u32,
    pub offset: u32,
}

impl From<Page> for PageResponse {
    fn from(page: Page) -> Self {
        Self {
            limit: page.limit,
            offset: page.offset,
        }
    }
}
