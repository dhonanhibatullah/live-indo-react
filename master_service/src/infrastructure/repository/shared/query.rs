use pgdt::sqlx::{Postgres, QueryBuilder};

use crate::domain::contracts::repository::Pagination;

pub(crate) fn push_deleted_filter(query: &mut QueryBuilder<Postgres>, include_deleted: bool) {
    if !include_deleted {
        query.push(" AND deleted_at IS NULL");
    }
}

pub(crate) fn push_name_contains(
    query: &mut QueryBuilder<Postgres>,
    column: &'static str,
    value: &str,
) {
    query
        .push(" AND ")
        .push(column)
        .push(" ILIKE ")
        .push_bind(contains_pattern(value))
        .push(" ESCAPE '\\'");
}

pub(crate) fn push_pagination(query: &mut QueryBuilder<Postgres>, pagination: Pagination) {
    query
        .push(" LIMIT ")
        .push_bind(i64::from(pagination.limit))
        .push(" OFFSET ")
        .push_bind(i64::from(pagination.offset));
}

fn contains_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for character in value.chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_like_metacharacters() {
        assert_eq!(contains_pattern("plain"), "%plain%");
        assert_eq!(contains_pattern("100%_done\\"), "%100\\%\\_done\\\\%");
    }
}
