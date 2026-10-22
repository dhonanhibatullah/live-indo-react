use crate::domain::models::{AppContext, entity::Topic};

use super::super::{Page, UsecaseFuture};

/// Public read access to available nationwide topics.
pub trait TopicCatalog: Send + Sync {
    fn list_active_topics<'a>(
        &'a self,
        context: &'a AppContext,
        request: ListTopics,
    ) -> UsecaseFuture<'a, Vec<Topic>>;

    fn get_topic_by_slug<'a>(
        &'a self,
        context: &'a AppContext,
        slug: &'a str,
    ) -> UsecaseFuture<'a, Topic>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListTopics {
    pub polling_method_id: Option<i64>,
    pub name_contains: Option<String>,
    pub page: Page,
}
