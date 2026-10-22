use pgdt::sqlx::{Postgres, QueryBuilder};

use crate::domain::{
    contracts::repository::indonesia_boundary::UpdateArgs, models::RepositoryError,
};

use super::super::shared::geo::encode_boundary;

const COLUMNS: &str = "id::bigint AS id, ST_AsGeoJSON(boundary, 15) AS boundary_geojson, boundary_source, boundary_reference_date, created_at, updated_at";

fn select() -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    query.push(COLUMNS).push(" FROM indonesia_boundary");
    query
}

pub(super) fn get() -> QueryBuilder<Postgres> {
    let mut query = select();
    query.push(" WHERE id = 1 LIMIT 1");
    query
}

pub(super) fn update(input: &UpdateArgs) -> Result<QueryBuilder<Postgres>, RepositoryError> {
    if input.boundary.is_none()
        && input.boundary_source.is_none()
        && input.boundary_reference_date.is_none()
    {
        return Ok(get());
    }

    let boundary = input.boundary.as_ref().map(encode_boundary).transpose()?;
    let mut query = QueryBuilder::new("UPDATE indonesia_boundary SET ");
    {
        let mut sets = query.separated(", ");
        if let Some(value) = boundary {
            sets.push("boundary = ST_SetSRID(ST_GeomFromGeoJSON(");
            sets.push_bind_unseparated(value);
            sets.push_unseparated("), 4326)::geometry(MultiPolygon, 4326)");
        }
        if let Some(value) = &input.boundary_source {
            sets.push("boundary_source = ")
                .push_bind_unseparated(value.clone());
        }
        match input.boundary_reference_date {
            None => {}
            Some(None) => {
                sets.push("boundary_reference_date = NULL");
            }
            Some(Some(value)) => {
                sets.push("boundary_reference_date = ")
                    .push_bind_unseparated(value);
            }
        }
        sets.push("updated_at = NOW()");
    }
    query.push(" WHERE id = 1 RETURNING ").push(COLUMNS);
    Ok(query)
}

pub(super) fn is_inside(longitude: f64, latitude: f64) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(
        "SELECT EXISTS (SELECT 1 FROM indonesia_boundary WHERE ST_Covers(boundary, ST_SetSRID(ST_MakePoint(",
    );
    query
        .push_bind(longitude)
        .push(", ")
        .push_bind(latitude)
        .push("), 4326))) AS inside");
    query
}
