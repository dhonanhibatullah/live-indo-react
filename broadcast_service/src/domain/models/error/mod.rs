mod generic;

use std::{error::Error as StdError, fmt};

pub use generic::ErrorKind;

/// A classified failure with an operation-specific message and optional cause.
///
/// Display includes diagnostic source text. Presentation must map the kind and
/// select a safe message rather than expose Display or the source chain directly.
#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl Error {
    pub fn new(message: impl Into<String>, kind: ErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
            source: None,
        }
    }

    /// Attach a native or domain cause without changing this error's classification.
    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {}", self.kind, self.message)?;

        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}
