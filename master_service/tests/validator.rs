use std::sync::{Arc, Mutex};

use master_service::{
    domain::{
        contracts::utility::{Logger, Validator},
        models::{
            AppContext, ValidatorError,
            logger::{Level, Meta, MetaValue},
        },
    },
    infrastructure::utility::validator::RegexValidator,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Record {
    level: Level,
    tag: String,
    message: String,
    meta: Option<Meta>,
}

#[derive(Default)]
struct CapturingLogger {
    records: Mutex<Vec<Record>>,
}

impl CapturingLogger {
    fn records(&self) -> Vec<Record> {
        self.records.lock().unwrap().clone()
    }
}

impl Logger for CapturingLogger {
    fn log(
        &self,
        _context: &AppContext,
        level: Level,
        tag: &str,
        message: &str,
        meta: Option<&Meta>,
    ) {
        self.records.lock().unwrap().push(Record {
            level,
            tag: tag.to_owned(),
            message: message.to_owned(),
            meta: meta.cloned(),
        });
    }
}

fn context() -> AppContext {
    AppContext {
        actor: Some("anonymous".into()),
        execution_id: Some(Uuid::from_u128(1)),
        trace_id: Some(Uuid::from_u128(2)),
        ..AppContext::default()
    }
}

fn validator() -> (Arc<CapturingLogger>, RegexValidator) {
    let logger = Arc::new(CapturingLogger::default());
    let validator = RegexValidator::new(Arc::clone(&logger) as Arc<dyn Logger>);
    (logger, validator)
}

#[test]
fn valid_master_data_is_accepted_without_logging() {
    let (logger, validator) = validator();
    let context = context();

    assert_eq!(
        validator.polling_method_code(&context, "agree_disagree"),
        Ok(())
    );
    assert_eq!(validator.polling_method_code(&context, "rating"), Ok(()));
    assert_eq!(validator.polling_method_name(&context, "Rating"), Ok(()));
    assert_eq!(validator.topic_slug(&context, "covid-19"), Ok(()));
    assert_eq!(validator.topic_name(&context, "Topic"), Ok(()));
    assert_eq!(
        validator.topic_selection_options(
            &context,
            "selection",
            &["Setuju".into(), "Tidak setuju".into()],
        ),
        Ok(())
    );
    assert_eq!(validator.province_kemendagri_code(&context, "01"), Ok(()));
    assert_eq!(validator.province_name(&context, "Aceh"), Ok(()));
    assert_eq!(
        validator.province_boundary_source(&context, "Kemendagri 2025"),
        Ok(())
    );
    assert_eq!(validator.city_kemendagri_code(&context, "11.01"), Ok(()));
    assert_eq!(validator.city_name(&context, "Simeulue"), Ok(()));
    assert_eq!(
        validator.city_boundary_source(&context, "Kemendagri 2025"),
        Ok(())
    );
    assert_eq!(
        validator.city_province_code(&context, "11.01", "11"),
        Ok(())
    );
    assert!(logger.records().is_empty());
}

#[test]
fn formats_and_nonblank_rules_are_enforced() {
    let (_, validator) = validator();
    let context = context();

    assert_eq!(
        validator.polling_method_code(&context, "AgreeDisagree"),
        Err(ValidatorError::PollingMethodCodeInvalid)
    );
    assert_eq!(
        validator.polling_method_code(&context, "agree__disagree"),
        Err(ValidatorError::PollingMethodCodeInvalid)
    );
    assert_eq!(
        validator.polling_method_name(&context, " \t "),
        Err(ValidatorError::PollingMethodNameInvalid)
    );
    assert_eq!(
        validator.topic_slug(&context, "COVID-19"),
        Err(ValidatorError::TopicSlugInvalid)
    );
    assert_eq!(
        validator.topic_slug(&context, "covid--19"),
        Err(ValidatorError::TopicSlugInvalid)
    );
    assert_eq!(
        validator.topic_name(&context, ""),
        Err(ValidatorError::TopicNameInvalid)
    );
    assert_eq!(
        validator.province_kemendagri_code(&context, "1"),
        Err(ValidatorError::ProvinceKemendagriCodeInvalid)
    );
    assert_eq!(
        validator.province_name(&context, ""),
        Err(ValidatorError::ProvinceNameInvalid)
    );
    assert_eq!(
        validator.province_boundary_source(&context, "\t"),
        Err(ValidatorError::ProvinceBoundarySourceInvalid)
    );
    assert_eq!(
        validator.city_kemendagri_code(&context, "11-01"),
        Err(ValidatorError::CityKemendagriCodeInvalid)
    );
    assert_eq!(
        validator.city_name(&context, " "),
        Err(ValidatorError::CityNameInvalid)
    );
    assert_eq!(
        validator.city_boundary_source(&context, " "),
        Err(ValidatorError::CityBoundarySourceInvalid)
    );
}

#[test]
fn selection_options_follow_the_polling_method() {
    let (_, validator) = validator();
    let context = context();

    assert_eq!(
        validator.topic_selection_options(&context, "agree_disagree", &[]),
        Ok(())
    );
    assert_eq!(
        validator.topic_selection_options(&context, "rating", &[]),
        Ok(())
    );
    assert_eq!(
        validator.topic_selection_options(&context, "rating", &["Yes".into()]),
        Err(ValidatorError::TopicSelectionOptionsNotAllowed)
    );
    assert_eq!(
        validator.topic_selection_options(&context, "selection", &["One".into()]),
        Err(ValidatorError::TopicSelectionOptionsTooFew)
    );
    assert_eq!(
        validator.topic_selection_options(&context, "selection", &["One".into(), "One".into()]),
        Err(ValidatorError::TopicSelectionOptionsInvalid)
    );
    assert_eq!(
        validator.topic_selection_options(&context, "selection", &[" ".into(), "Two".into()]),
        Err(ValidatorError::TopicSelectionOptionsInvalid)
    );
    assert_eq!(
        validator.topic_selection_options(&context, "selection", &[" One".into(), "Two".into()]),
        Err(ValidatorError::TopicSelectionOptionsInvalid)
    );
    assert_eq!(
        validator.topic_selection_options(&context, "unknown", &[]),
        Err(ValidatorError::PollingMethodCodeInvalid)
    );

    let options = vec!["Yes".into(), "yes".into()];
    assert_eq!(
        validator.topic_selection_options(&context, "selection", &options),
        Ok(())
    );
    assert_eq!(options, vec!["Yes".to_owned(), "yes".to_owned()]);
}

#[test]
fn city_and_province_codes_must_agree() {
    let (_, validator) = validator();
    let context = context();

    assert_eq!(
        validator.city_province_code(&context, "11.01", "12"),
        Err(ValidatorError::CityProvinceCodeMismatch)
    );
    assert_eq!(
        validator.city_province_code(&context, "11-01", "11"),
        Err(ValidatorError::CityKemendagriCodeInvalid)
    );
    assert_eq!(
        validator.city_province_code(&context, "11.01", "1"),
        Err(ValidatorError::ProvinceKemendagriCodeInvalid)
    );
}

#[test]
fn rejection_logs_once_with_classified_metadata() {
    let (logger, validator) = validator();
    let context = context();

    assert_eq!(
        validator.topic_slug(&context, "Not_Valid"),
        Err(ValidatorError::TopicSlugInvalid)
    );

    let records = logger.records();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.level, Level::Error);
    assert_eq!(record.tag, "validator/topic_slug");
    assert_eq!(record.message, "topic slug is invalid");
    let meta = record.meta.as_ref().unwrap();
    assert_eq!(
        meta["error_code"],
        MetaValue::String("TOPIC_SLUG_INVALID".into())
    );
    assert_eq!(
        meta["reason"],
        MetaValue::String("topic slug is invalid".into())
    );
}
