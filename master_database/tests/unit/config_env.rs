use super::*;

fn example() -> HashMap<String, String> {
    parse_dotenv(include_str!("../../.env.example").as_bytes()).unwrap()
}

#[test]
fn example_matches_all_connection_settings() {
    let values = example();
    let settings = from_sources(&values, |_| Ok(None)).unwrap();
    assert_eq!(settings.postgres_host, "127.0.0.1");
    assert_eq!(settings.postgres_port, 5432);
    assert_eq!(
        settings.postgres_username,
        values["MASTER_DATABASE_POSTGRES_USERNAME"]
    );
    assert_eq!(
        settings.postgres_password,
        values["MASTER_DATABASE_POSTGRES_PASSWORD"]
    );
    assert_eq!(settings.postgres_database, "master_database");
    assert_eq!(settings.postgres_ssl_mode, "prefer");
}

#[test]
fn process_values_override_dotenv_and_preserve_password_bytes() {
    let password = "  password # $ ' \"  ";
    let settings = from_sources(&example(), |name| {
        Ok(match name {
            "MASTER_DATABASE_POSTGRES_HOST" => Some("database.internal".into()),
            "MASTER_DATABASE_POSTGRES_PORT" => Some("6543".into()),
            "MASTER_DATABASE_POSTGRES_PASSWORD" => Some(password.into()),
            _ => None,
        })
    })
    .unwrap();
    assert_eq!(settings.postgres_host, "database.internal");
    assert_eq!(settings.postgres_port, 6543);
    assert_eq!(settings.postgres_password, password);
    assert_eq!(settings.postgres_database, "master_database");
}

#[test]
fn missing_file_is_optional_and_credentials_have_no_defaults() {
    let missing = std::env::temp_dir().join(format!(
        "live-indo-missing-env-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut values = read_dotenv(&missing).unwrap();
    assert!(values.is_empty());
    assert!(from_sources(&values, |_| Ok(None)).is_err());
    values.insert(
        "MASTER_DATABASE_POSTGRES_USERNAME".into(),
        "test-user".into(),
    );
    assert!(from_sources(&values, |_| Ok(None)).is_err());
    values.insert("MASTER_DATABASE_POSTGRES_PASSWORD".into(), String::new());
    let settings = from_sources(&values, |_| Ok(None)).unwrap();
    assert_eq!(settings.postgres_port, 5432);
    assert_eq!(settings.postgres_host, "127.0.0.1");
    assert_eq!(settings.postgres_database, "master_database");
    assert_eq!(settings.postgres_ssl_mode, "prefer");
    assert_eq!(settings.postgres_password, "");
}

#[test]
fn invalid_process_values_do_not_fall_back_to_dotenv() {
    for (name, value) in [
        ("MASTER_DATABASE_POSTGRES_PORT", "0"),
        ("MASTER_DATABASE_POSTGRES_PORT", "65536"),
        ("MASTER_DATABASE_POSTGRES_PORT", "not-a-port"),
        ("MASTER_DATABASE_POSTGRES_SSL_MODE", "invalid-mode"),
        ("MASTER_DATABASE_POSTGRES_HOST", "   "),
        ("MASTER_DATABASE_POSTGRES_USERNAME", ""),
        ("MASTER_DATABASE_POSTGRES_DATABASE", "  "),
    ] {
        let error = from_sources(&example(), |candidate| {
            Ok((candidate == name).then(|| value.to_owned()))
        })
        .unwrap_err();
        assert!(matches!(error, LoadError::Variable { name: failed, .. } if failed == name));
    }
}

#[test]
fn malformed_dotenv_errors_do_not_expose_source_lines() {
    let error = parse_dotenv(b"PASSWORD='secret-unclosed\n".as_slice()).unwrap_err();
    assert!(matches!(error, LoadError::DotenvSyntax));
    assert!(!format!("{error:?} {error}").contains("secret-unclosed"));
    assert!(error.source().is_none());
}
