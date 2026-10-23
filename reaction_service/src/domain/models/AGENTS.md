# Reaction service: domain models

Define the vocabulary shared by contracts and usecases. Follow the [service
guide](../../../AGENTS.md) and root Rust conventions.

## Layout and style

Use focused modules such as `reaction.rs`, `province.rs`, `topic.rs`, and `error/`
when their types are needed. Keep each value's invariants close to its definition;
prefer explicit domain values over unstructured maps. Do not invent identifier
formats, emotion catalogs, message limits, or aggregation rules from examples.

Models contain domain data, not database rows, HTTP requests, WebSocket messages,
or broker SDK objects. Keep transport serialization and persistence mapping in
adapters. Keep model dependencies minimal and independent of concrete I/O libraries.
Use typed error variants with useful causes; preserve the distinction between
invalid values, missing records where applicable, and operational failure.

## App models

`app/` follows the master-service layout: `context.rs`, `transaction.rs`, `info.rs`,
and `env.rs`, re-exported through `models`. `AppContext` carries optional opaque
actor, UUID trace/execution IDs and an optional `AppTransaction`. Actor is caller
metadata, never proof of authentication or permission. Models do not generate IDs.

`AppTransaction` wraps an opaque `Arc<dyn Any + Send + Sync>` owned by a future
adapter. Clones share the handle; equality compares handle identity, not contents.
Its Debug output omits contents. This is vocabulary only, with no transaction
runner or database implementation introduced here.

`AppInfo` contains static name/version strings; Cargo metadata loading is separate.
`AppEnv` contains typed logger and PostgreSQL settings, including pool size and
connection timeout. Do not derive Debug on credential-bearing settings. Loading,
defaults, validation and additional runtime settings remain deferred. Context and
settings types must not introduce transport or driver dependencies.

## Service responsibility

Represent anonymous reaction/comment data and its province/topic scope. Do not add
account, credential, or permission models. Server-assigned fields must not become
caller-controlled merely because they exist on an entity.

Test value invariants and error classifications as they are implemented. Models
must stay usable without a running server, database, or event source.

## Logger models

`logger/` contains `Format` (`plain`/`json`), `Level`
(`NONE`/`ERROR`/`WARN`/`INFO`/`DEBUG`), and `Meta`/`MetaValue`.
Use `FromStr` for strict configuration parsing; enum values are valid by
construction. Level ordering represents maximum verbosity; `None` is never an
emitted severity. Metadata uses domain-owned values and ordered string keys,
without a serialization-library dependency. Convert errors explicitly with
`MetaValue::from_error`; diagnostic strings must be safe to log. Logging is
best effort and has no dedicated error model.

## Generic errors

`error/` exposes `Error` and `ErrorKind`. Keep generic classifications in
`error/generic.rs`, retaining Lockmate's codes: NOT_FOUND, BAD_ARGS, CONFLICT,
BAD_STATE, FORBIDDEN, UNAUTHORIZED, TIMEOUT, UNIMPLEMENTED, FAILURE, and UNKNOWN.
These are classifications, not authentication features or transport status codes.

Construct with `Error::new(message, kind)` and attach native or domain causes with
`.with_source(cause)`. Classify the current error through `.kind()` and inspect
causes through `std::error::Error::source` and downcasting. Wrapping retains each
error's own classification; it does not automatically reclassify an outer error.
Display includes `[CODE] message` and optional source diagnostics. Presentation
must select safe output rather than serialize this diagnostic chain. Keep sources
`Send + Sync + 'static` for use across service tasks. Logger emission failures
are silently ignored rather than returned as domain errors.
