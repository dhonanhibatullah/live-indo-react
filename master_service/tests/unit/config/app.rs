#[test]
fn app_metadata_comes_from_cargo() {
    let app = master_service::config::app::load();

    assert_eq!(app.name, "master_service");
    assert_eq!(app.version, env!("CARGO_PKG_VERSION"));
}
