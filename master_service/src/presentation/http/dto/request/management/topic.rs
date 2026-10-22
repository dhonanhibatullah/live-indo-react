use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTopicRequest {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub polling_method_id: i64,
    #[serde(default)]
    pub selection_options: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTopicRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub polling_method_id: Option<i64>,
    pub selection_options: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

impl From<CreateTopicRequest> for crate::domain::usecases::management::topic::CreateTopic {
    fn from(value: CreateTopicRequest) -> Self {
        Self {
            slug: value.slug,
            name: value.name,
            description: value.description,
            polling_method_id: value.polling_method_id,
            selection_options: value.selection_options,
            is_active: value.is_active,
        }
    }
}

impl From<UpdateTopicRequest> for crate::domain::usecases::management::topic::UpdateTopic {
    fn from(value: UpdateTopicRequest) -> Self {
        Self {
            id: 0,
            name: value.name,
            description: value.description,
            polling_method_id: value.polling_method_id,
            selection_options: value.selection_options,
            is_active: value.is_active,
        }
    }
}
