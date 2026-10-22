use std::io;

use master_service::domain::models::logger::{Format, Level, MetaValue};

#[test]
fn configuration_values_round_trip_and_reject_invalid_input() {
    for level in [
        Level::None,
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
    ] {
        assert_eq!(level.to_string().parse::<Level>(), Ok(level));
    }
    for format in [Format::Plain, Format::Json] {
        assert_eq!(format.to_string().parse::<Format>(), Ok(format));
    }
    for invalid in ["", "info", "TRACE", " INFO", "INFO "] {
        assert!(invalid.parse::<Level>().is_err(), "{invalid:?}");
    }
    for invalid in ["", "JSON", "text", " json", "plain "] {
        assert!(invalid.parse::<Format>().is_err(), "{invalid:?}");
    }
}

#[test]
fn level_thresholds_allow_only_emitted_severities_up_to_the_limit() {
    assert!(!Level::None.allows(Level::None));
    assert!(!Level::None.allows(Level::Error));
    assert!(Level::Error.allows(Level::Error));
    assert!(!Level::Error.allows(Level::Warn));
    assert!(Level::Info.allows(Level::Error));
    assert!(Level::Info.allows(Level::Warn));
    assert!(Level::Info.allows(Level::Info));
    assert!(!Level::Info.allows(Level::Debug));
}

#[test]
fn meta_value_from_error_captures_display_text() {
    let error = io::Error::new(io::ErrorKind::BrokenPipe, "connection closed");

    assert_eq!(
        MetaValue::from_error(&error),
        MetaValue::String("connection closed".into())
    );
}
