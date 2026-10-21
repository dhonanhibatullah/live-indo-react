use reaction_database::models::Env;

#[test]
fn environment_debug_output_redacts_the_password() {
    let settings = Env {
        postgres_host: "localhost".into(),
        postgres_port: 5432,
        postgres_username: "test-user".into(),
        postgres_password: "private-password".into(),
        postgres_database: "reaction_database".into(),
        postgres_ssl_mode: "prefer".into(),
    };
    for debug in [format!("{settings:?}"), format!("{settings:#?}")] {
        assert!(!debug.contains(&settings.postgres_password));
        assert!(debug.contains("[REDACTED]"));
    }
}
