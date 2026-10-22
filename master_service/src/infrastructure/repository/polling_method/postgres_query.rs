use pgdt::sqlx::{Postgres, QueryBuilder};

use crate::domain::contracts::repository::polling_method::{ListArgs, UpdateArgs};

use super::super::shared::query::{push_deleted_filter, push_pagination};

const COLUMNS: &str = "id, code, name, description, is_active, created_at, updated_at, deleted_at";

fn select() -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    query.push(COLUMNS).push(" FROM polling_methods");
    query
}

pub(super) fn find_by_id(id: i64) -> QueryBuilder<Postgres> {
    let mut query = select();
    query.push(" WHERE id = ").push_bind(id).push(" LIMIT 1");
    query
}

pub(super) fn find_by_code(code: &str) -> QueryBuilder<Postgres> {
    let mut query = select();
    query
        .push(" WHERE code = ")
        .push_bind(code.to_owned())
        .push(" LIMIT 1");
    query
}

pub(super) fn list(filter: &ListArgs) -> QueryBuilder<Postgres> {
    let mut query = select();
    query.push(" WHERE TRUE");
    push_deleted_filter(&mut query, filter.include_deleted);
    if let Some(value) = filter.is_active {
        query.push(" AND is_active = ").push_bind(value);
    }
    query.push(" ORDER BY name ASC, id ASC");
    push_pagination(&mut query, filter.pagination);
    query
}

pub(super) fn update(id: i64, input: &UpdateArgs) -> QueryBuilder<Postgres> {
    if input.name.is_none() && input.description.is_none() && input.is_active.is_none() {
        return find_active_by_id(id);
    }

    let mut query = QueryBuilder::new("UPDATE polling_methods SET ");
    {
        let mut sets = query.separated(", ");
        if let Some(value) = &input.name {
            sets.push("name = ").push_bind_unseparated(value.clone());
        }
        if let Some(value) = &input.description {
            sets.push("description = ")
                .push_bind_unseparated(value.clone());
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
    query
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
    let mut query = QueryBuilder::new("UPDATE polling_methods SET deleted_at = NOW() WHERE id = ");
    query.push_bind(id).push(" AND deleted_at IS NULL");
    query
}
