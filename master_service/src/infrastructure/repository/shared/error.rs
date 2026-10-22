use pgdt::sqlx;

use crate::domain::{
    contracts::utility::Logger,
    models::{AppContext, RepositoryError, logger::Meta},
};

pub(crate) enum OperationError {
    Driver(sqlx::Error),
    Domain(RepositoryError),
}

impl From<sqlx::Error> for OperationError {
    fn from(source: sqlx::Error) -> Self {
        Self::Driver(source)
    }
}

impl From<RepositoryError> for OperationError {
    fn from(error: RepositoryError) -> Self {
        Self::Domain(error)
    }
}

pub(crate) fn finish<T>(
    logger: &dyn Logger,
    context: &AppContext,
    tag: &str,
    result: Result<T, OperationError>,
    not_found: fn() -> RepositoryError,
) -> Result<T, RepositoryError> {
    result.map_err(|error| {
        let mut meta = Meta::new();
        let error = match error {
            OperationError::Domain(error) => error,
            OperationError::Driver(source) => {
                if let Some(database) = source.as_database_error() {
                    if let Some(code) = database.code() {
                        meta.insert("sql_state".into(), code.as_ref().into());
                    }
                    if let Some(constraint) = database.constraint() {
                        meta.insert("constraint".into(), constraint.into());
                    }
                }
                map_driver(source, not_found)
            }
        };
        meta.insert("error_code".into(), error.code().into());
        logger.error(context, tag, &error.to_string(), Some(&meta));
        error
    })
}

fn map_driver(source: sqlx::Error, not_found: fn() -> RepositoryError) -> RepositoryError {
    if matches!(source, sqlx::Error::RowNotFound) {
        return not_found();
    }
    if matches!(source, sqlx::Error::PoolTimedOut)
        || matches!(&source, sqlx::Error::Io(error) if error.kind() == std::io::ErrorKind::TimedOut)
    {
        return RepositoryError::Timeout {
            source: Box::new(source),
        };
    }
    if let Some(database) = source.as_database_error()
        && let Some(code) = database.code()
        && let Some(error) = classify_sqlstate(code.as_ref(), database.constraint())
    {
        return error;
    }
    RepositoryError::Failure {
        source: Box::new(source),
    }
}

fn classify_sqlstate(code: &str, constraint: Option<&str>) -> Option<RepositoryError> {
    match code {
        "23505" => Some(match constraint {
            Some("uq_topics_slug") => RepositoryError::TopicSlugConflict,
            Some("uq_polling_methods_code") => RepositoryError::PollingMethodCodeConflict,
            Some("uq_provinces_kemendagri_code") => RepositoryError::ProvinceKemendagriCodeConflict,
            Some("uq_cities_kemendagri_code") => RepositoryError::CityKemendagriCodeConflict,
            _ => RepositoryError::Conflict,
        }),
        "23503" => Some(match constraint {
            Some("fk_topics_polling_method") => RepositoryError::TopicPollingMethodNotFound,
            Some("fk_cities_province") => RepositoryError::CityProvinceNotFound,
            _ => RepositoryError::Conflict,
        }),
        "23502" | "23514" | "22P02" | "22001" | "22003" | "22023" => Some(RepositoryError::BadArgs),
        _ => None,
    }
}

pub(crate) fn require_affected(
    rows: u64,
    not_found: fn() -> RepositoryError,
) -> Result<(), OperationError> {
    if rows == 0 {
        Err(not_found().into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_unique_constraints() {
        assert!(matches!(
            classify_sqlstate("23505", Some("uq_topics_slug")),
            Some(RepositoryError::TopicSlugConflict)
        ));
        assert!(matches!(
            classify_sqlstate("23505", Some("uq_cities_kemendagri_code")),
            Some(RepositoryError::CityKemendagriCodeConflict)
        ));
        assert!(matches!(
            classify_sqlstate("23505", Some("unknown_index")),
            Some(RepositoryError::Conflict)
        ));
    }

    #[test]
    fn classifies_known_foreign_keys() {
        assert!(matches!(
            classify_sqlstate("23503", Some("fk_topics_polling_method")),
            Some(RepositoryError::TopicPollingMethodNotFound)
        ));
        assert!(matches!(
            classify_sqlstate("23503", Some("fk_cities_province")),
            Some(RepositoryError::CityProvinceNotFound)
        ));
        assert!(matches!(
            classify_sqlstate("23503", Some("unknown_constraint")),
            Some(RepositoryError::Conflict)
        ));
    }

    #[test]
    fn classifies_arguments_and_unknown_states() {
        assert!(matches!(
            classify_sqlstate("22P02", None),
            Some(RepositoryError::BadArgs)
        ));
        assert!(classify_sqlstate("XX000", None).is_none());
    }
}
