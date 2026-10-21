use std::fmt;

/// Generic failure classifications shared across application boundaries.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    NotFound,
    BadArgs,
    Conflict,
    BadState,
    Forbidden,
    Unauthorized,
    Timeout,
    Unimplemented,
    Failure,
    Unknown,
}

impl ErrorKind {
    /// Stable diagnostic code, matching the Lockmate generic error vocabulary.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "NOT_FOUND",
            Self::BadArgs => "BAD_ARGS",
            Self::Conflict => "CONFLICT",
            Self::BadState => "BAD_STATE",
            Self::Forbidden => "FORBIDDEN",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Timeout => "TIMEOUT",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::Failure => "FAILURE",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
