use std::io;

use super::*;

#[test]
fn console_logs_are_verbose_and_write_failures_are_silent() {
    let mut output = Vec::new();
    write_log(
        &mut output,
        "INFO",
        format_args!("schema migrations complete"),
    );
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "[INFO] schema migrations complete\n"
    );
    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("closed"))
        }
    }
    write_log(&mut Broken, "ERROR", format_args!("failed operation"));
}

#[test]
fn database_errors_keep_causes_without_exposing_them_in_console_output() {
    let error = DatabaseError::sql(
        "test operation",
        sqlx::Error::Protocol("sensitive-diagnostic-value".into()),
    );
    assert!(!format!("{error} {error:?}").contains("sensitive-diagnostic-value"));
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("sensitive-diagnostic-value")
    );
    assert_eq!(error.operation, "test operation");
}
