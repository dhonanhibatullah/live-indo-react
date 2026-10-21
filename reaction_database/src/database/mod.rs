//! PostgreSQL connection and tracked schema migration operations.
//! The caller owns pool shutdown; no production reaction data is inserted.

pub mod connection;
pub mod migrations;

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

/// Display/Debug are safe for console output; source() retains native diagnostics.
pub struct DatabaseError {
    pub operation: &'static str,
    details: Box<ErrorDetails>,
}

struct ErrorDetails {
    sqlstate: Option<String>,
    constraint: Option<String>,
    message: String,
    source: Box<dyn Error + Send + Sync>,
}

impl DatabaseError {
    pub fn sqlstate(&self) -> Option<&str> {
        self.details.sqlstate.as_deref()
    }

    pub fn constraint(&self) -> Option<&str> {
        self.details.constraint.as_deref()
    }

    fn new(operation: &'static str, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            operation,
            details: Box::new(ErrorDetails {
                sqlstate: None,
                constraint: None,
                message: "operation failed".into(),
                source: Box::new(source),
            }),
        }
    }
    fn sql(operation: &'static str, source: sqlx::Error) -> Self {
        let sqlstate = source
            .as_database_error()
            .and_then(|error| error.code())
            .map(|code| code.into_owned());
        let constraint = source
            .as_database_error()
            .and_then(|error| error.constraint())
            .map(str::to_owned);
        let mut error = Self::new(operation, source);
        error.details.sqlstate = sqlstate;
        error.details.constraint = constraint;
        error
    }
    fn invalid(operation: &'static str, message: impl Into<String>) -> Self {
        let message = message.into();
        let mut error = Self::new(operation, io::Error::other(message.clone()));
        error.details.message = message;
        error
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.details.message)?;
        if let Some(state) = &self.details.sqlstate {
            write!(formatter, " sqlstate={state}")?;
        }
        if let Some(constraint) = &self.details.constraint {
            write!(formatter, " constraint={constraint}")?;
        }
        Ok(())
    }
}
impl fmt::Debug for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.details.source.as_ref())
    }
}

fn write_log(writer: &mut impl Write, level: &str, message: fmt::Arguments<'_>) {
    let _ = writeln!(writer, "[{level}] {message}");
}
fn info(message: fmt::Arguments<'_>) {
    write_log(&mut io::stdout().lock(), "INFO", message);
}
fn log_error(error: &DatabaseError) {
    write_log(&mut io::stderr().lock(), "ERROR", format_args!("{error}"));
}

#[cfg(test)]
#[path = "../../tests/unit/database.rs"]
mod tests;
