use crate::domain::models::AppInfo;

/// Load compile-time service metadata without reading runtime files.
pub const fn load() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    }
}
