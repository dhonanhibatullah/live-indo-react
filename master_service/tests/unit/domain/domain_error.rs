use std::{error::Error as StdError, io};

use master_service::domain::models::error::{Error, ErrorKind};

#[test]
fn generic_classifications_keep_lockmate_codes() {
    for (kind, code) in [
        (ErrorKind::NotFound, "NOT_FOUND"),
        (ErrorKind::BadArgs, "BAD_ARGS"),
        (ErrorKind::Conflict, "CONFLICT"),
        (ErrorKind::BadState, "BAD_STATE"),
        (ErrorKind::Forbidden, "FORBIDDEN"),
        (ErrorKind::Unauthorized, "UNAUTHORIZED"),
        (ErrorKind::Timeout, "TIMEOUT"),
        (ErrorKind::Unimplemented, "UNIMPLEMENTED"),
        (ErrorKind::Failure, "FAILURE"),
        (ErrorKind::Unknown, "UNKNOWN"),
    ] {
        let error = Error::new("operation failed", kind);
        assert_eq!(kind.as_str(), code);
        assert_eq!(kind.to_string(), code);
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), "operation failed");
        assert_eq!(error.to_string(), format!("[{code}] operation failed"));
        assert!(error.source().is_none());
    }
}

#[test]
fn native_cause_is_preserved_for_downcasting_and_diagnostics() {
    let error = Error::new(String::from("delivery failed"), ErrorKind::Failure).with_source(
        io::Error::new(io::ErrorKind::BrokenPipe, "connection closed"),
    );

    assert_eq!(error.kind(), ErrorKind::Failure);
    assert_eq!(error.message(), "delivery failed");
    assert_eq!(
        error.to_string(),
        "[FAILURE] delivery failed: connection closed"
    );
    let cause = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(cause.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(cause.to_string(), "connection closed");
}

#[test]
fn nested_domain_errors_keep_each_classification_and_native_source() {
    let inner = Error::new("dependency timed out", ErrorKind::Timeout)
        .with_source(io::Error::new(io::ErrorKind::TimedOut, "deadline reached"));
    let outer = Error::new("submission failed", ErrorKind::Failure).with_source(inner);
    let boxed: Box<dyn StdError + Send + Sync> = Box::new(outer);

    let outer = boxed.downcast_ref::<Error>().unwrap();
    assert_eq!(outer.kind(), ErrorKind::Failure);
    let inner = outer.source().unwrap().downcast_ref::<Error>().unwrap();
    assert_eq!(inner.kind(), ErrorKind::Timeout);
    assert_eq!(inner.message(), "dependency timed out");
    let native = inner.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(native.kind(), io::ErrorKind::TimedOut);
    assert_eq!(
        outer.to_string(),
        "[FAILURE] submission failed: [TIMEOUT] dependency timed out: deadline reached"
    );
}
