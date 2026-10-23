use std::{error::Error, fmt, fs, io, path::Path};

use crate::domain::models::app::GeoIpConf;

/// Load required MaxMind settings from GeoIP.conf in the working directory.
/// No environment overrides, downloads or process-environment mutations occur.
pub fn load() -> Result<GeoIpConf, LoadError> {
    load_from_path("GeoIP.conf")
}

/// Load the three required directives from an explicit GeoIP.conf path.
/// Blank lines, full-line comments and whitespace-separated edition IDs are supported.
/// Unsupported directives and duplicate entries fail rather than being ignored.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<GeoIpConf, LoadError> {
    let contents = fs::read_to_string(path).map_err(LoadError::Io)?;
    parse(&contents)
}

fn parse(contents: &str) -> Result<GeoIpConf, LoadError> {
    let mut account_id = None;
    let mut license_key = None;
    let mut edition_ids = None;

    for (index, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let number = index + 1;
        let invalid = |reason| LoadError::Entry { number, reason };
        let (name, value) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| invalid("directive requires a value"))?;
        let value = value.trim();
        if value.is_empty() {
            return Err(invalid("directive requires a value"));
        }

        match name {
            "AccountID" => {
                if account_id.is_some() {
                    return Err(invalid("duplicate AccountID directive"));
                }
                account_id = Some(
                    value
                        .parse::<u64>()
                        .ok()
                        .filter(|id| *id > 0)
                        .ok_or_else(|| invalid("AccountID must be a positive integer"))?,
                );
            }
            "LicenseKey" => {
                if license_key.is_some() {
                    return Err(invalid("duplicate LicenseKey directive"));
                }
                if value.chars().any(char::is_whitespace) {
                    return Err(invalid("LicenseKey must be a single nonempty token"));
                }
                license_key = Some(value.to_owned());
            }
            "EditionIDs" => {
                if edition_ids.is_some() {
                    return Err(invalid("duplicate EditionIDs directive"));
                }
                let editions: Vec<String> = value.split_whitespace().map(str::to_owned).collect();
                if editions.iter().any(|edition| {
                    !edition
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                }) {
                    return Err(invalid(
                        "EditionIDs must contain database edition identifiers",
                    ));
                }
                edition_ids = Some(editions);
            }
            _ => return Err(invalid("unsupported directive")),
        }
    }

    Ok(GeoIpConf {
        account_id: account_id.ok_or(LoadError::Missing("AccountID"))?,
        license_key: license_key.ok_or(LoadError::Missing("LicenseKey"))?,
        edition_ids: edition_ids.ok_or(LoadError::Missing("EditionIDs"))?,
    })
}

/// Errors retain I/O causes but never include configuration values or source lines.
pub enum LoadError {
    Io(io::Error),
    Missing(&'static str),
    Entry { number: usize, reason: &'static str },
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => formatter.write_str("could not read GeoIP configuration file"),
            Self::Missing(name) => write!(formatter, "GeoIP configuration requires {name}"),
            Self::Entry { number, reason } => {
                write!(formatter, "GeoIP configuration line {number}: {reason}")
            }
        }
    }
}

impl fmt::Debug for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LoadError, parse};

    fn base() -> &'static str {
        "AccountID 123\nLicenseKey test-license-key\nEditionIDs GeoLite2-City\n"
    }

    fn error(contents: &str) -> LoadError {
        match parse(contents) {
            Ok(_) => panic!("expected invalid configuration"),
            Err(error) => error,
        }
    }

    #[test]
    fn parses_exported_format_comments_tabs_crlf_and_multiple_editions() {
        let conf = parse(
            "# Exported settings\r\n\r\n AccountID\t123 \r\nLicenseKey test-license-key\r\nEditionIDs GeoLite2-ASN  GeoLite2-City\tGeoLite2-Country\r\n",
        )
        .unwrap();
        assert_eq!(conf.account_id, 123);
        assert_eq!(conf.license_key, "test-license-key");
        assert_eq!(
            conf.edition_ids,
            ["GeoLite2-ASN", "GeoLite2-City", "GeoLite2-Country"]
        );
    }

    #[test]
    fn requires_each_directive_and_rejects_duplicates() {
        for name in ["AccountID", "LicenseKey", "EditionIDs"] {
            let lines: Vec<_> = base()
                .lines()
                .filter(|line| !line.starts_with(name))
                .collect();
            assert!(matches!(error(&lines.join("\n")), LoadError::Missing(key) if key == name));
            let duplicate = base().lines().find(|line| line.starts_with(name)).unwrap();
            assert!(matches!(
                error(&format!("{}{duplicate}\n", base())),
                LoadError::Entry { .. }
            ));
        }
    }

    #[test]
    fn malformed_values_and_directives_fail_without_echoing_secrets() {
        for line in [
            "AccountID 0",
            "AccountID -1",
            "AccountID private-secret",
            "LicenseKey",
            "LicenseKey private secret",
            "EditionIDs ",
            "EditionIDs private/secret",
            "Unknown private-secret",
        ] {
            let err = error(line);
            assert!(matches!(err, LoadError::Entry { number: 1, .. }));
            assert!(!err.to_string().contains("private"));
            assert!(!format!("{err:?}").contains("secret"));
        }
        assert!(matches!(
            error("# comment only"),
            LoadError::Missing("AccountID")
        ));
    }
}
