use std::{error::Error, fmt, io};

/// Seed loading failures retain the original cause and identify the failed file.
#[derive(Debug)]
pub enum LoadError {
    Decompress {
        filename: &'static str,
        source: io::Error,
    },
    Deserialize {
        filename: &'static str,
        source: serde_json::Error,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decompress { filename, .. } => {
                write!(formatter, "failed to decompress seed {filename}")
            }
            Self::Deserialize { filename, .. } => {
                write!(formatter, "failed to deserialize seed {filename}")
            }
        }
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decompress { source, .. } => Some(source),
            Self::Deserialize { source, .. } => Some(source),
        }
    }
}
