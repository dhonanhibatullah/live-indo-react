/// Package identity baked into the executable from Cargo.toml at build time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct App {
    pub name: &'static str,
    pub version: &'static str,
}
