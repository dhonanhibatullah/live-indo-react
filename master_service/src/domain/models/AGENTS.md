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
