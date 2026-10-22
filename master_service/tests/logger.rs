use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread,
};

use master_service::{
    domain::{
        contracts::utility::logger::Logger,
        models::{
            AppContext,
            logger::{Format, Level, Meta, MetaValue},
        },
    },
    infrastructure::utility::logger::WriterLogger,
};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Buffer {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    fn records(&self) -> Vec<Value> {
        self.text()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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
fn every_threshold_filters_all_severity_methods_in_both_formats() {
    for format in [Format::Plain, Format::Json] {
        for (threshold, expected) in [
            (Level::None, 0),
            (Level::Error, 1),
            (Level::Warn, 2),
            (Level::Info, 3),
            (Level::Debug, 4),
        ] {
            let output = Buffer::default();
            let adapter = WriterLogger::new(output.clone(), format, threshold);
            let logger: &dyn Logger = &adapter;
            let context = AppContext::default();

            logger.error(&context, "test/error", "failed", None);
            logger.warn(&context, "test/warn", "warning", None);
            logger.info(&context, "test/info", "accepted", None);
            logger.debug(&context, "test/debug", "diagnostic", None);
            logger.log(&context, Level::None, "test/none", "never emitted", None);

            let text = output.text();
            assert_eq!(text.lines().count(), expected, "{format} {threshold}");
            let levels = ["ERROR", "WARN", "INFO", "DEBUG"];
            if format == Format::Json {
                let records = output.records();
                for (record, expected_level) in records.iter().zip(levels) {
                    assert_eq!(record["level"], expected_level);
                    assert_eq!(record["actor"], "");
                    assert_eq!(record["execution_id"], "");
                    assert_eq!(record["trace_id"], "");
                    assert_eq!(record["meta"], json!({}));
                    assert!(record["time_unix_ms"].as_u64().unwrap() > 0);
                }
            } else {
                for (line, expected_level) in text.lines().zip(levels) {
                    assert!(line.contains(&format!("level={expected_level} ")));
                    assert!(line.contains("actor=\"\" execution_id=\"\" trace_id=\"\""));
                    assert!(line.ends_with("meta={}"));
                }
            }
        }
    }
}

#[test]
fn context_identifiers_are_emitted_in_both_formats() {
    let execution_id = Uuid::from_u128(3);
    let trace_id = Uuid::from_u128(4);
    let context = AppContext {
        actor: Some("anonymous".into()),
        execution_id: Some(execution_id),
        trace_id: Some(trace_id),
        ..AppContext::default()
    };

    for format in [Format::Plain, Format::Json] {
        let output = Buffer::default();
        let logger = WriterLogger::new(output.clone(), format, Level::Info);

        logger.info(&context, "test/context", "identified", None);

        let text = output.text();
        if format == Format::Json {
            let record = &output.records()[0];
            assert_eq!(record["actor"], "anonymous");
            assert_eq!(record["execution_id"], execution_id.to_string());
            assert_eq!(record["trace_id"], trace_id.to_string());
        } else {
            assert!(text.contains(&format!(
                "actor=\"anonymous\" execution_id=\"{execution_id}\" trace_id=\"{trace_id}\""
            )));
        }
    }
}

#[test]
fn json_preserves_metadata_types_and_normalizes_errors_without_mutating_input() {
    let error = io::Error::other("delivery unavailable");
    let meta = Meta::from([
        ("err".into(), MetaValue::from_error(&error)),
        ("attempt".into(), 2_u64.into()),
        ("offset".into(), (-3_i64).into()),
        ("largest".into(), u64::MAX.into()),
        ("ready".into(), true.into()),
        ("ratio".into(), 0.5.into()),
        ("missing".into(), MetaValue::Null),
        (
            "scope".into(),
            MetaValue::Object(Meta::from([("province".into(), "test-province".into())])),
        ),
        (
            "topics".into(),
            MetaValue::Array(vec!["one".into(), "two".into()]),
        ),
        ("tag".into(), "cannot replace record tag".into()),
    ]);
    let original = meta.clone();
    let output = Buffer::default();
    let logger = WriterLogger::new(output.clone(), Format::Json, Level::Debug);

    logger.error(&context(), "delivery/send", "send failed", Some(&meta));

    assert_eq!(meta, original);
    let record = &output.records()[0];
    assert_eq!(record["tag"], "delivery/send");
    assert_eq!(record["msg"], "send failed");
    assert_eq!(
        record["meta"],
        json!({
            "err": "delivery unavailable",
            "attempt": 2,
            "offset": -3,
            "largest": u64::MAX,
            "ready": true,
            "ratio": 0.5,
            "missing": null,
            "scope": {"province": "test-province"},
            "topics": ["one", "two"],
            "tag": "cannot replace record tag",
        })
    );
    assert!(record.get("trace").is_none());
}

#[test]
fn nonfinite_floats_are_null_and_input_remains_unchanged() {
    let meta = Meta::from([
        ("nan".into(), f64::NAN.into()),
        ("infinity".into(), f64::INFINITY.into()),
        ("negative_infinity".into(), f64::NEG_INFINITY.into()),
    ]);
    let output = Buffer::default();
    let logger = WriterLogger::new(output.clone(), Format::Json, Level::Info);

    logger.info(&context(), "test/float", "normalized", Some(&meta));

    assert_eq!(
        output.records()[0]["meta"],
        json!({
            "nan": null, "infinity": null, "negative_infinity": null,
        })
    );
    assert!(matches!(meta["nan"], MetaValue::Float(value) if value.is_nan()));
}

#[test]
fn both_formats_escape_text_to_prevent_extra_log_lines() {
    for format in [Format::Plain, Format::Json] {
        let output = Buffer::default();
        let logger = WriterLogger::new(output.clone(), format, Level::Info);
        let meta = Meta::from([("detail\nkey".into(), "line\nbreak\r\t\"".into())]);
        let tag = "test\nforged\r\"";
        let message = "hello\nworld\r\t\"\\";
        let context = AppContext {
            actor: Some("actor\nforged\r\"".into()),
            ..AppContext::default()
        };

        logger.info(&context, tag, message, Some(&meta));

        let text = output.text();
        assert_eq!(text.lines().count(), 1);
        assert!(text.ends_with('\n'));
        assert!(!text[..text.len() - 1].contains(['\n', '\r', '\t']));
        if format == Format::Json {
            let record = &output.records()[0];
            assert_eq!(record["tag"], tag);
            assert_eq!(record["msg"], message);
            assert_eq!(record["actor"], "actor\nforged\r\"");
            assert_eq!(record["meta"]["detail\nkey"], "line\nbreak\r\t\"");
        } else {
            assert!(text.contains("msg=\"hello\\nworld\\r\\t\\\"\\\\\""));
            assert!(text.contains("tag=\"test\\nforged\\r\\\"\""));
            assert!(text.contains("actor=\"actor\\nforged\\r\\\"\""));
        }
    }
}

struct FailingWriter {
    fail_on_flush: bool,
    calls: Arc<Mutex<(usize, usize)>>,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.calls.lock().unwrap().0 += 1;
        if self.fail_on_flush {
            Ok(bytes.len())
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "writer closed"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.calls.lock().unwrap().1 += 1;
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "flush failed"))
    }
}

#[test]
fn write_and_flush_failures_are_silent_and_filtered_calls_do_not_write() {
    for fail_on_flush in [false, true] {
        let calls = Arc::new(Mutex::new((0, 0)));
        let logger = WriterLogger::new(
            FailingWriter {
                fail_on_flush,
                calls: Arc::clone(&calls),
            },
            Format::Json,
            Level::Error,
        );
        let context = AppContext::default();
        logger.debug(&context, "test", "filtered", None);
        assert_eq!(*calls.lock().unwrap(), (0, 0));

        logger.error(&context, "test", "emitted", None);
        assert_eq!(*calls.lock().unwrap(), (1, usize::from(fail_on_flush)));
    }
}

#[test]
fn concurrent_calls_produce_complete_independent_records() {
    let output = Buffer::default();
    let logger: Arc<dyn Logger> =
        Arc::new(WriterLogger::new(output.clone(), Format::Json, Level::Info));
    let handles: Vec<_> = (0..8)
        .map(|worker| {
            let logger = Arc::clone(&logger);
            thread::spawn(move || {
                let context = context();
                for item in 0..25 {
                    let meta = Meta::from([
                        ("worker".into(), MetaValue::Integer(worker)),
                        ("item".into(), MetaValue::Integer(item)),
                    ]);
                    logger.info(&context, "test/concurrent", "record", Some(&meta));
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }

    let records = output.records();
    assert_eq!(records.len(), 200);
    let pairs: std::collections::BTreeSet<_> = records
        .iter()
        .map(|record| {
            assert_eq!(record["tag"], "test/concurrent");
            (
                record["meta"]["worker"].as_i64().unwrap(),
                record["meta"]["item"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(pairs.len(), 200);
}
