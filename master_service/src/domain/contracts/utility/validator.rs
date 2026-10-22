use crate::domain::models::{AppContext, ValidatorError};

/// Synchronous validation for master-data fields and cross-field rules.
///
/// Implementations must not mutate their inputs, perform I/O, or retain caller
/// values. Failures are classified with stable [`ValidatorError`] codes; logging
/// is best effort and never changes the returned result.
pub trait Validator: Send + Sync {
    fn polling_method_code(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn polling_method_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn topic_slug(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn topic_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn topic_selection_options(
        &self,
        context: &AppContext,
        polling_method_code: &str,
        options: &[String],
    ) -> Result<(), ValidatorError>;

    fn province_kemendagri_code(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError>;

    fn province_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn province_boundary_source(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError>;

    fn city_kemendagri_code(&self, context: &AppContext, value: &str)
    -> Result<(), ValidatorError>;

    fn city_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError>;

    fn city_boundary_source(&self, context: &AppContext, value: &str)
    -> Result<(), ValidatorError>;

    fn city_province_code(
        &self,
        context: &AppContext,
        city_code: &str,
        province_code: &str,
    ) -> Result<(), ValidatorError>;
}
