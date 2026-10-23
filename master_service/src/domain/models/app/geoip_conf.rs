/// MaxMind credentials and database editions read from GeoIP.conf.
///
/// Intentionally has no Debug implementation because it contains a license key.
/// Parsing and filesystem access belong to configuration, not this model.
pub struct GeoIpConf {
    pub account_id: u64,
    pub license_key: String,
    pub edition_ids: Vec<String>,
}
