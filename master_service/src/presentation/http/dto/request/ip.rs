use std::net::IpAddr;

use serde::Deserialize;

/// Explicit lookup input. Avoid Debug output that would expose the raw address.
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpQuery {
    pub ip: IpAddr,
}
