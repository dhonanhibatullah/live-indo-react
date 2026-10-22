use std::{collections::BTreeSet, sync::Arc};

use regex::Regex;

use crate::domain::{
    contracts::utility::{logger::Logger, validator::Validator},
    models::{AppContext, ValidatorError, logger::Meta},
};

/// Synchronous validator backed by compiled regular expressions.
///
/// Logs classified failures once through the injected logger. Valid input does not
/// touch the logger. Patterns mirror the master-data format rules formerly enforced
/// by database CHECK constraints.
pub struct RegexValidator {
    logger: Arc<dyn Logger>,
    polling_method_code_pattern: Regex,
    topic_slug_pattern: Regex,
    province_code_pattern: Regex,
    city_code_pattern: Regex,
}

impl RegexValidator {
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self {
            logger,
            polling_method_code_pattern: Regex::new(r"\A[a-z]+(?:_[a-z]+)*\z")
                .expect("polling method code pattern is valid"),
            topic_slug_pattern: Regex::new(r"\A[a-z0-9]+(?:-[a-z0-9]+)*\z")
                .expect("topic slug pattern is valid"),
            province_code_pattern: Regex::new(r"\A[0-9]{2}\z")
                .expect("province code pattern is valid"),
            city_code_pattern: Regex::new(r"\A[0-9]{2}\.[0-9]{2}\z")
                .expect("city code pattern is valid"),
        }
    }

    fn check_pattern(
        &self,
        context: &AppContext,
        tag: &str,
        value: &str,
        pattern: &Regex,
        error: ValidatorError,
    ) -> Result<(), ValidatorError> {
        if pattern.is_match(value) {
            Ok(())
        } else {
            self.reject(context, tag, error)
        }
    }

    fn check_non_blank(
        &self,
        context: &AppContext,
        tag: &str,
        value: &str,
        error: ValidatorError,
    ) -> Result<(), ValidatorError> {
        if value.trim().is_empty() {
            self.reject(context, tag, error)
        } else {
            Ok(())
        }
    }

    fn reject(
        &self,
        context: &AppContext,
        tag: &str,
        error: ValidatorError,
    ) -> Result<(), ValidatorError> {
        let message = error.to_string();
        let meta = Meta::from([
            ("error_code".into(), error.code().into()),
            ("reason".into(), message.clone().into()),
        ]);
        self.logger.error(context, tag, &message, Some(&meta));
        Err(error)
    }
}

impl Validator for RegexValidator {
    fn polling_method_code(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_pattern(
            context,
            "validator/polling_method_code",
            value,
            &self.polling_method_code_pattern,
            ValidatorError::PollingMethodCodeInvalid,
        )
    }

    fn polling_method_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/polling_method_name",
            value,
            ValidatorError::PollingMethodNameInvalid,
        )
    }

    fn topic_slug(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_pattern(
            context,
            "validator/topic_slug",
            value,
            &self.topic_slug_pattern,
            ValidatorError::TopicSlugInvalid,
        )
    }

    fn topic_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/topic_name",
            value,
            ValidatorError::TopicNameInvalid,
        )
    }

    fn topic_selection_options(
        &self,
        context: &AppContext,
        polling_method_code: &str,
        options: &[String],
    ) -> Result<(), ValidatorError> {
        match polling_method_code {
            "agree_disagree" | "rating" => {
                if options.is_empty() {
                    Ok(())
                } else {
                    self.reject(
                        context,
                        "validator/topic_selection_options",
                        ValidatorError::TopicSelectionOptionsNotAllowed,
                    )
                }
            }
            "selection" => {
                if options.len() < 2 {
                    return self.reject(
                        context,
                        "validator/topic_selection_options",
                        ValidatorError::TopicSelectionOptionsTooFew,
                    );
                }

                let mut seen = BTreeSet::new();
                for option in options {
                    let trimmed = option.trim();
                    if trimmed.is_empty() || trimmed != option || !seen.insert(option) {
                        return self.reject(
                            context,
                            "validator/topic_selection_options",
                            ValidatorError::TopicSelectionOptionsInvalid,
                        );
                    }
                }

                Ok(())
            }
            _ => self.reject(
                context,
                "validator/topic_selection_options",
                ValidatorError::PollingMethodCodeInvalid,
            ),
        }
    }

    fn province_kemendagri_code(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError> {
        self.check_pattern(
            context,
            "validator/province_kemendagri_code",
            value,
            &self.province_code_pattern,
            ValidatorError::ProvinceKemendagriCodeInvalid,
        )
    }

    fn province_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/province_name",
            value,
            ValidatorError::ProvinceNameInvalid,
        )
    }

    fn province_boundary_source(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/province_boundary_source",
            value,
            ValidatorError::ProvinceBoundarySourceInvalid,
        )
    }

    fn city_kemendagri_code(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError> {
        self.check_pattern(
            context,
            "validator/city_kemendagri_code",
            value,
            &self.city_code_pattern,
            ValidatorError::CityKemendagriCodeInvalid,
        )
    }

    fn city_name(&self, context: &AppContext, value: &str) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/city_name",
            value,
            ValidatorError::CityNameInvalid,
        )
    }

    fn city_boundary_source(
        &self,
        context: &AppContext,
        value: &str,
    ) -> Result<(), ValidatorError> {
        self.check_non_blank(
            context,
            "validator/city_boundary_source",
            value,
            ValidatorError::CityBoundarySourceInvalid,
        )
    }

    fn city_province_code(
        &self,
        context: &AppContext,
        city_code: &str,
        province_code: &str,
    ) -> Result<(), ValidatorError> {
        if !self.city_code_pattern.is_match(city_code) {
            return self.reject(
                context,
                "validator/city_province_code",
                ValidatorError::CityKemendagriCodeInvalid,
            );
        }

        if !self.province_code_pattern.is_match(province_code) {
            return self.reject(
                context,
                "validator/city_province_code",
                ValidatorError::ProvinceKemendagriCodeInvalid,
            );
        }

        if &city_code[..2] != province_code {
            return self.reject(
                context,
                "validator/city_province_code",
                ValidatorError::CityProvinceCodeMismatch,
            );
        }

        Ok(())
    }
}
