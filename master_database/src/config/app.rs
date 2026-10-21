use crate::models::App;

/// Load Cargo package metadata without reading Cargo.toml at runtime.
pub const fn load() -> App {
    App {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    }
}
