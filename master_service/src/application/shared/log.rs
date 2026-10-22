use crate::domain::{
    contracts::utility::Logger,
    models::{AppContext, RepositoryError, ValidatorError, logger::Meta},
};

/// Log an application-owned repository-classified failure.
pub(crate) fn log_repository_error(
    logger: &dyn Logger,
    context: &AppContext,
    tag: &str,
    error: &RepositoryError,
) {
    let meta = Meta::from([("error_code".into(), error.code().into())]);
    logger.error(context, tag, &error.to_string(), Some(&meta));
}

/// Log an application-owned validation failure.
#[allow(dead_code)]
pub(crate) fn log_validator_error(
    logger: &dyn Logger,
    context: &AppContext,
    tag: &str,
    error: &ValidatorError,
) {
    let meta = Meta::from([("error_code".into(), error.code().into())]);
    logger.error(context, tag, &error.to_string(), Some(&meta));
}
