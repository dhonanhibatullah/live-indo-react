use super::*;

fn settings() -> Env {
    Env {
        postgres_host: "localhost".into(),
        postgres_port: 6543,
        postgres_username: "test-user".into(),
        postgres_password: "special:p@ss/#'".into(),
        postgres_database: "master_test".into(),
        postgres_ssl_mode: "verify-full".into(),
    }
}

#[test]
fn connection_settings_map_to_typed_options() {
    let options = options(&settings()).unwrap();
    assert_eq!(options.get_host(), "localhost");
    assert_eq!(options.get_port(), 6543);
    assert_eq!(options.get_username(), "test-user");
    assert_eq!(options.get_database(), Some("master_test"));
    assert!(matches!(options.get_ssl_mode(), PgSslMode::VerifyFull));
}

#[test]
fn connection_rejects_invalid_direct_settings_without_logging_values() {
    let mut env = settings();
    env.postgres_ssl_mode = "private-invalid-mode".into();
    let error = options(&env).unwrap_err();
    assert!(!error.to_string().contains("private-invalid-mode"));
    env.postgres_ssl_mode = "disable".into();
    env.postgres_port = 0;
    assert!(options(&env).is_err());
}
