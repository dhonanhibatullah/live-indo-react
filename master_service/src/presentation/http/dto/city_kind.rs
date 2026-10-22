use serde::{Deserialize, Serialize};

use crate::domain::models::entity::CityKind;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CityKindDto {
    City,
    Regency,
}

impl CityKindDto {
    pub(crate) fn into_domain(self) -> CityKind {
        match self {
            Self::City => CityKind::City,
            Self::Regency => CityKind::Regency,
        }
    }
}

impl From<CityKind> for CityKindDto {
    fn from(value: CityKind) -> Self {
        match value {
            CityKind::City => Self::City,
            CityKind::Regency => Self::Regency,
        }
    }
}
