use std::{error::Error as StdError, fmt};

use pgdt::sqlx::{Row, postgres::PgRow};

use crate::domain::models::{
    RepositoryError,
    entity::{City, CityKind},
};

use super::super::{error::OperationError, geo};

pub(crate) fn decode(row: &PgRow) -> Result<City, OperationError> {
    let boundary: String = row.try_get("boundary_geojson")?;
    let kind: String = row.try_get("kind")?;
    Ok(City {
        id: row.try_get("id")?,
        province_id: row.try_get("province_id")?,
        kemendagri_code: row.try_get("kemendagri_code")?,
        kind: decode_kind(&kind)?,
        name: row.try_get("name")?,
        boundary: geo::decode_boundary(&boundary)?,
        boundary_source: row.try_get("boundary_source")?,
        boundary_reference_date: row.try_get("boundary_reference_date")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        deleted_at: row.try_get("deleted_at")?,
    })
}

fn decode_kind(value: &str) -> Result<CityKind, RepositoryError> {
    match value {
        "city" => Ok(CityKind::City),
        "regency" => Ok(CityKind::Regency),
        _ => Err(RepositoryError::Failure {
            source: Box::new(UnknownCityKind),
        }),
    }
}

#[derive(Debug)]
struct UnknownCityKind;

impl fmt::Display for UnknownCityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("stored city kind is not recognized")
    }
}

impl StdError for UnknownCityKind {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_city_kinds() {
        assert!(matches!(decode_kind("city"), Ok(CityKind::City)));
        assert!(matches!(decode_kind("regency"), Ok(CityKind::Regency)));
        assert!(decode_kind("village").is_err());
    }
}
