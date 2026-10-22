use serde::Deserialize;

use super::super::page::PageQuery;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListTopicsQuery {
    pub polling_method_id: Option<i64>,
    pub name_contains: Option<String>,
    #[serde(flatten)]
    pub page: PageQuery,
}
