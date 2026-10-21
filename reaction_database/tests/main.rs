#[test]
fn invalid_configuration_exits_with_safe_diagnostics() {
    let directory = std::env::temp_dir().join(format!(
        "live_indo_reaction_main_config_{}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_reaction_database"))
        .current_dir(&directory)
        .env_clear()
        .env("REACTION_DATABASE_POSTGRES_PORT", "invalid-secret-value")
        .output()
        .unwrap();
    std::fs::remove_dir(directory).unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[ERROR] load configuration: REACTION_DATABASE_POSTGRES_PORT"));
    assert!(!stderr.contains("invalid-secret-value"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("database connection starting"));
}
