use serde::Deserialize;

use super::super::page::PageQuery;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListPollingMethodsQuery {
    #[serde(flatten)]
    pub page: PageQuery,
}
