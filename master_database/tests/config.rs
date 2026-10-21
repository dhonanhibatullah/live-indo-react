use master_database::config;

#[test]
fn app_metadata_matches_the_compiled_cargo_package() {
    let app = config::app::load();
    assert_eq!(app.name, env!("CARGO_PKG_NAME"));
    assert_eq!(app.version, env!("CARGO_PKG_VERSION"));
}
