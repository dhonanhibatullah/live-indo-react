use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePollingMethodRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

impl From<UpdatePollingMethodRequest>
    for crate::domain::usecases::management::polling_method::UpdatePollingMethod
{
    fn from(value: UpdatePollingMethodRequest) -> Self {
        Self {
            id: 0,
            name: value.name,
            description: value.description,
            is_active: value.is_active,
        }
    }
}
