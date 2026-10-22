use pgdt::sqlx::{Postgres, QueryBuilder};

use crate::domain::{
    contracts::repository::city::{CreateArgs, ListArgs, UpdateArgs},
    models::RepositoryError,
};

use super::super::shared::{
    geo::encode_boundary,
    query::{push_deleted_filter, push_name_contains, push_pagination},
};

const COLUMNS: &str = "id, province_id, kemendagri_code, kind::text AS kind, name, ST_AsGeoJSON(boundary, 15) AS boundary_geojson, boundary_source, boundary_reference_date, is_active, created_at, updated_at, deleted_at";

fn select() -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    query.push(COLUMNS).push(" FROM cities");
    query
}

pub(super) fn create(input: &CreateArgs) -> Result<QueryBuilder<Postgres>, RepositoryError> {
    let boundary = encode_boundary(&input.boundary)?;
    let mut query = QueryBuilder::new(
        "INSERT INTO cities (province_id, kemendagri_code, kind, name, boundary, boundary_source, boundary_reference_date, is_active) VALUES (",
    );
    {
        let mut values = query.separated(", ");
        values.push_bind(input.province_id);
        values.push_bind(input.kemendagri_code.clone());
        values.push_bind(input.kind.as_str());
        values.push_unseparated("::city_kind");
        values.push_bind(input.name.clone());
        values.push("ST_SetSRID(ST_GeomFromGeoJSON(");
        values.push_bind_unseparated(boundary);
        values.push_unseparated("), 4326)::geometry(MultiPolygon, 4326)");
        values.push_bind(input.boundary_source.clone());
        values.push_bind(input.boundary_reference_date);
        values.push_bind(input.is_active);
    }
    query.push(") RETURNING ").push(COLUMNS);
    Ok(query)
}

pub(super) fn find_by_id(id: i64) -> QueryBuilder<Postgres> {
    let mut query = select();
    query.push(" WHERE id = ").push_bind(id).push(" LIMIT 1");
    query
}

pub(super) fn find_by_kemendagri_code(code: &str) -> QueryBuilder<Postgres> {
    let mut query = select();
    query
        .push(" WHERE kemendagri_code = ")
        .push_bind(code.to_owned())
        .push(" LIMIT 1");
    query
}

pub(super) fn find_by_coord(longitude: f64, latitude: f64) -> QueryBuilder<Postgres> {
    let mut query = select();
    query
        .push(" WHERE deleted_at IS NULL AND is_active AND ST_Covers(boundary, ST_SetSRID(ST_MakePoint(")
        .push_bind(longitude)
        .push(", ")
        .push_bind(latitude)
        .push("), 4326)) ORDER BY id ASC LIMIT 1");
    query
}

pub(super) fn list(filter: &ListArgs) -> QueryBuilder<Postgres> {
    let mut query = select();
    query.push(" WHERE TRUE");
    push_deleted_filter(&mut query, filter.include_deleted);
    if let Some(value) = filter.province_id {
        query.push(" AND province_id = ").push_bind(value);
    }
    if let Some(value) = filter.kind {
        query
            .push(" AND kind = ")
            .push_bind(value.as_str())
            .push("::city_kind");
    }
    if let Some(value) = filter.is_active {
        query.push(" AND is_active = ").push_bind(value);
    }
    if let Some(value) = &filter.name_contains {
        push_name_contains(&mut query, "name", value);
    }
    query.push(" ORDER BY name ASC, id ASC");
    push_pagination(&mut query, filter.pagination);
    query
}

pub(super) fn update(
    id: i64,
    input: &UpdateArgs,
) -> Result<QueryBuilder<Postgres>, RepositoryError> {
    if input.kind.is_none()
        && input.name.is_none()
        && input.boundary.is_none()
        && input.boundary_source.is_none()
        && input.boundary_reference_date.is_none()
        && input.is_active.is_none()
    {
        return Ok(find_active_by_id(id));
    }

    let boundary = input.boundary.as_ref().map(encode_boundary).transpose()?;
    let mut query = QueryBuilder::new("UPDATE cities SET ");
    {
        let mut sets = query.separated(", ");
        if let Some(value) = input.kind {
            sets.push("kind = ")
                .push_bind_unseparated(value.as_str())
                .push_unseparated("::city_kind");
        }
        if let Some(value) = &input.name {
            sets.push("name = ").push_bind_unseparated(value.clone());
        }
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
        if let Some(value) = input.is_active {
            sets.push("is_active = ").push_bind_unseparated(value);
        }
        sets.push("updated_at = NOW()");
    }
    query
        .push(" WHERE id = ")
        .push_bind(id)
        .push(" AND deleted_at IS NULL RETURNING ")
        .push(COLUMNS);
    Ok(query)
}

fn find_active_by_id(id: i64) -> QueryBuilder<Postgres> {
    let mut query = select();
    query
        .push(" WHERE id = ")
        .push_bind(id)
        .push(" AND deleted_at IS NULL LIMIT 1");
    query
}

pub(super) fn delete(id: i64) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("UPDATE cities SET deleted_at = NOW() WHERE id = ");
    query.push_bind(id).push(" AND deleted_at IS NULL");
    query
}
