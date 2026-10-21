use std::{error::Error, fmt, str::FromStr};

/// Maximum verbosity, ordered from disabled to most verbose.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    None,
    Error,
    Warn,
    Info,
    Debug,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
        }
    }

    /// NONE is a configuration value and never an emitted severity.
    pub fn allows(self, severity: Self) -> bool {
        severity != Self::None && severity <= self
    }
}

impl fmt::Display for Level {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "NONE" => Ok(Self::None),
            "ERROR" => Ok(Self::Error),
            "WARN" => Ok(Self::Warn),
            "INFO" => Ok(Self::Info),
            "DEBUG" => Ok(Self::Debug),
            _ => Err(ParseLevelError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseLevelError;

impl fmt::Display for ParseLevelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("logger level must be NONE, ERROR, WARN, INFO, or DEBUG")
    }
}

impl Error for ParseLevelError {}
