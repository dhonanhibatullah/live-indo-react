use crate::domain::models::{AppContext, entity::Topic};

use super::super::UsecaseFuture;

/// Strict management operations for nationwide reaction topics.
pub trait ManageTopic: Send + Sync {
    fn create_topic<'a>(
        &'a self,
        context: &'a AppContext,
        input: CreateTopic,
    ) -> UsecaseFuture<'a, Topic>;

    fn update_topic<'a>(
        &'a self,
        context: &'a AppContext,
        input: UpdateTopic,
    ) -> UsecaseFuture<'a, Topic>;

    fn delete_topic<'a>(&'a self, context: &'a AppContext, id: i64) -> UsecaseFuture<'a, ()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateTopic {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub polling_method_id: i64,
    pub selection_options: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateTopic {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub polling_method_id: Option<i64>,
    pub selection_options: Option<Vec<String>>,
    pub is_active: Option<bool>,
}
