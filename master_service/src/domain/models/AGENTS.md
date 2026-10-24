# Master service: domain models

Define the vocabulary shared by contracts and usecases. Follow the [service
guide](../../../AGENTS.md) and root Rust conventions.

## Layout and style

Use focused modules such as `topic.rs`, `polling_method.rs`, `province.rs`, `city.rs`, and `error/`
when their types are needed. Keep each value's invariants close to its definition;
prefer explicit domain values over unstructured maps. Do not invent identifier
formats, emotion catalogs, message limits, or aggregation rules from examples.

Models contain domain data, not database rows, HTTP requests, WebSocket messages,
or broker SDK objects. Keep transport serialization and persistence mapping in
adapters. Keep model dependencies minimal and independent of concrete I/O libraries.
Use typed error variants with useful causes; preserve the distinction between
invalid values, missing records where applicable, and operational failure.

## Service responsibility

Represent master-data vocabulary as features need it. Preserve topic slugs,
polling-method codes, Kemendagri codes, and ordered topic option meanings. Keep
server-owned identifiers and audit fields out of caller-controlled input. Do not
introduce accounts, credentials, permission models, or reaction submission models
solely because other projects have them.

Test value invariants and error classifications as they are implemented in
`tests/unit/domain/`. Models must stay usable without a running server, database,
or event source.

## Logger models

`logger/` contains `Format` (`plain`/`json`), `Level`
(`NONE`/`ERROR`/`WARN`/`INFO`/`DEBUG`), and `Meta`/`MetaValue`.
Use `FromStr` for strict configuration parsing; enum values are valid by
construction. Level ordering represents maximum verbosity; `None` is never an
emitted severity. Metadata uses domain-owned values and ordered string keys,
without a serialization-library dependency. Convert errors explicitly with
`MetaValue::from_error`; diagnostic strings must be safe to log. Logging is
best effort and has no dedicated error model.

## App models

`app/` defines request and process vocabulary shared across layers. `AppInfo`
carries compile-time service name and version. `AppContext` carries an optional
opaque actor value plus trace, execution, and optional transaction handles; the
actor is never authentication or authorization. `AppTransaction` is an opaque,
cloneable handle with identity-based equality and typed downcasting for
infrastructure adapters. `AppEnv` holds logger format/level plus PostgreSQL host,
port, credentials, database, SSL mode, pool size, connect timeout, cache maximum
capacity, and separate individual/list cache TTLs; it deliberately does not
implement `Debug` because it contains a password. It also holds the GeoIP
credentials-file and MMDB-file paths as PathBuf values; configuration supplies
working-directory-relative defaults and rejects blank values.
Environment parsing, default selection, setting validation, pool construction,
and cache construction belong to configuration/composition,
not this model layer.

`GeoIpConf` in `app/geoip_conf.rs` holds a numeric MaxMind account ID, license
key and ordered database edition identifiers. It has no Debug implementation
because it contains credentials. Parsing and file access live in
`config/geoip_conf.rs`; the service adapter consumes this model explicitly.

## GeoIP location

`entity/geoip.rs` defines `GeoIp`, optional country/continent/city/postal/time-zone
values, ordered provider subdivisions, optional estimated WGS84 coordinates and
accuracy radius. Missing coordinates/fields remain absent, not zero/empty values.
Provider region codes are not Kemendagri codes. Use the located country rather
than an ISP registration country for this location vocabulary. Location and point
Debug output omit location values. Do not add transport or MMDB types here.

`GeoIpError` owns stable GEOIP_* codes for readiness, invalid configuration,
invalid database/download, unavailable upstream, timeout and operational failure.
Operational variants preserve native causes while Display/Debug stay safe; never
log raw causes containing provider data or signed download URLs.

## Error model

`error/` exposes the generic `Error` and `ErrorKind` pair plus the
category-specific `ValidatorError` and `RepositoryError` enums. Keep generic
classifications in `error/generic.rs`, retaining Lockmate's codes: NOT_FOUND,
BAD_ARGS, CONFLICT, BAD_STATE, FORBIDDEN, UNAUTHORIZED, TIMEOUT, UNIMPLEMENTED,
FAILURE, and UNKNOWN. These are classifications, not authentication features or
transport status codes.

`Error` combines a diagnostic message, generic kind, and optional native or
domain cause. Construct it with `Error::new(message, kind)` and attach causes with
`.with_source(cause)`. Wrapping retains each error's own classification; it does
not automatically reclassify an outer error. Display includes `[CODE] message`
and optional source diagnostics. Presentation must select safe output rather than
serialize this diagnostic chain. Keep sources `Send + Sync + 'static` for use
across service tasks. Logger emission failures are silently ignored rather than
returned as domain errors.

`ValidatorError` classifies environment and master-data validation rules with
stable codes such as `TOPIC_SLUG_INVALID` and `CITY_PROVINCE_CODE_MISMATCH`.
`RepositoryError` has generic operation buckets plus entity-specific not-found,
conflict, and relation failures such as `TOPIC_SLUG_CONFLICT` and
`CITY_PROVINCE_NOT_FOUND`. `UsecaseError` aggregates validation, repository and GeoIP
failures without losing their classifications. Only `Timeout` and `Failure` retain a native source;
classification variants have none. Both enums expose `code()`; Display remains
diagnostic. Add variants only for rules or failures the feature actually defines;
do not invent lengths, formats, aggregation formulas, or permissions to fill an
enum. Test stable codes, safe messages, and source preservation.
