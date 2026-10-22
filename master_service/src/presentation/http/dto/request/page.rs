use serde::Deserialize;

use crate::domain::usecases::Page;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

impl PageQuery {
    pub(crate) fn into_page(self) -> Page {
        let limit = if self.limit == 0 {
            default_limit()
        } else {
            self.limit.min(100)
        };
        Page {
            limit,
            offset: self.offset,
        }
    }
}

const fn default_limit() -> u32 {
    10
}
