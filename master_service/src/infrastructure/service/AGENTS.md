# Master service: external service infrastructure

Read the [service guide](../../../AGENTS.md) and
[contract guide](../../domain/contracts/AGENTS.md). Implement domain service
contracts here, separate from repositories and utility adapters. Keep provider
payloads, HTTP clients, archive readers and file operations inside infrastructure.
Use explicit constructor dependencies, focused files, safe category errors and
context-first logging. Application owns admission policy and scheduling.

## GeoIP adapter

`geoip/https.rs` exposes `HttpsGeoIp` implementing `GeoIpService`. `new` takes
`GeoIpConf`, a database path and logger; it builds an HTTPS-only client with a
10-second connect timeout, 120-second request timeout and at most five redirects.
`with_client` also accepts an injected client and trusted HTTPS download URL for
custom trust/proxy configuration and isolated tests. Custom clients must retain
bounded HTTPS and credential-stripping behavior. Endpoints receive Basic-auth
credentials; never accept an endpoint from request input.

Require GeoLite2-City among configured editions; download only that edition.
Use the MaxMind download endpoint with tar.gz suffix. Follow HTTPS redirects and
let the client strip authorization across origins. Never retain signed URLs in
request errors or log credentials, paths containing secrets, headers or IPs.
Log operation success/error codes through the injected context-aware logger.
After a completed installation, update logs INFO "GeoIP database updated". When
the serialized recheck skips replacement, log INFO "GeoIP update skipped: database
is current". Both use `geoip/update`; failed updates emit ERROR only. Keep the
installation outcome private to infrastructure without changing the unit-returning
service contract. Application logs its own skip only when it never calls update.

`is_ready` reports whether a validated in-memory City reader is loaded, without
network access. Missing/corrupt local files start unready so update can repair
them; other file I/O errors fail construction. `get_location` returns owned domain
values, None for an unmatched IP, and NotReady when unloaded. It uses the located
country, not the ISP's registered country. Preserve absent fields and distinguish
provider subdivision codes from our Kemendagri codes. Coordinate estimates do not
establish physical presence. No country-admission policy lives here.

`check_for_update` returns true when unloaded; otherwise compare remote HEAD
Last-Modified against the local release timestamp. Missing headers, HTTP failures
and timeouts are errors. Installation preserves Last-Modified as the file mtime,
so a restarted instance can check the same release without downloading again.
Do not touch the mtime externally; a manually supplied file with another timestamp
may be downloaded once to establish matching release metadata.

`archive.rs` bounds compressed downloads to 256 MiB and expanded data to 512 MiB,
accepts one regular GeoLite2-City.mmdb entry and never unpacks archive paths.
`reader.rs` verifies the MMDB structure and City edition and maps provider values.
Blocking file I/O, decompression, verification and lookups run off the async
executor. Limit concurrent lookup workers to 64. An instance serializes updates,
writes/syncs a same-directory temporary file, atomically replaces the target and
swaps readers while allowing existing lookups to finish against their snapshots.
The temporary file is removed on failure. Cancellation before installation leaves
the current database intact; a started blocking installation may finish after
caller cancellation and retains its update permit until it does.

One instance owns one database path. Multiple processes/instances writing a shared
path require external coordination. Replacement is atomic during operation, not
a promise of durable directory metadata across a machine crash. No background
scheduler, retries, automatic startup downloads, routes or RPCs belong in this
adapter. Composition now shares it with region lookup and maintenance; cron
presentation triggers maintenance daily at 00:00 UTC.
Tests use local HTTPS and dummy MaxMind databases under
[tests](../../../tests/AGENTS.md), never production credentials or downloads.
