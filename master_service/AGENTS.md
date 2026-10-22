# Master service

Provide the service boundary for live-indo-react master data: topics, polling
methods, provinces, and cities/regencies. Use the master database schema as the
data reference; database preparation remains in `master_database`.

Read the [root guide](../AGENTS.md) first. This is an independent Rust 2024 crate
with a binary entrypoint and a library exposing domain and infrastructure modules.
Generic domain errors, logger models, the logger utility contract, and its
writer adapter are implemented. Entity models, repository contracts, and app
context/config models for topics, polling methods, provinces, and cities are
implemented. `shared/crates/pgdt` is available for future infrastructure but
is not yet a dependency. JSON formatting uses `serde_json` only in infrastructure.
`src/main.rs` remains a placeholder. The remaining modules below are the target
architecture, not an implemented master-data runtime.

## Target source layout

```text
src/
  lib.rs                        # domain and infrastructure library boundary
  main.rs                       # thin entrypoint delegating to composition
  config.rs                     # typed settings and environment loading
  domain/
    models/                     # domain vocabulary and classified errors
    contracts/                  # repository and utility traits
    usecases/                   # application traits and request/result values
  application/                  # usecase implementations, grouped by capability
  infrastructure/
    repository/                 # concrete persistence adapters, if needed
    utility/                    # external integrations and technical adapters
  presentation/
    http/                       # transport handlers and explicit wire DTOs
  composition/                  # dependency construction and lifecycle
tests/                          # integration tests (at crate root)
```

Use Rust module declarations only as implementation files are added; do not create
empty modules to satisfy this tree. Keep capability-specific code together and
split files by responsibility when needed. Do not add a repository implementation
to a service without an actual persistence requirement.

## Scoped guides

- Domain: [models](src/domain/models/AGENTS.md),
  [contracts](src/domain/contracts/AGENTS.md),
  [usecases](src/domain/usecases/AGENTS.md).
- [Application](src/application/AGENTS.md).
- Infrastructure: [repositories](src/infrastructure/repository/AGENTS.md),
  [utilities](src/infrastructure/utility/AGENTS.md).
- [Presentation](src/presentation/AGENTS.md).
- [Composition](src/composition/AGENTS.md).

## Service behavior and boundaries

Presentation maps transport input to domain usecase requests. Application owns
business validation and calls injected domain contracts. Infrastructure implements
those contracts against selected drivers or providers. Master-data runtime
operations are not implemented by this scaffold.

The master schema lives in [master_database/docs/erd.dbml](../master_database/docs/erd.dbml).
Future features should preserve stable topic slugs and regional/method codes,
validate polling-method-dependent topic options, and keep topic polling
configuration fixed after responses exist. `selection_options` is an ordered
native string array; reaction selection indexes depend on that ordering. There
are no database business triggers for these policies, and implementing their
cross-service checks needs an explicit feature design.

Reaction submission belongs to `reaction_service`; WebSocket subscriber handling
and fan-out belong to `broadcast_service`. Do not import another service's internal
modules. Expose explicit domain types at boundaries and coordinate future external
contracts with consumers. Do not add authentication, routes, write permissions,
cache behavior, or publication guarantees by analogy; those are feature decisions.
The PostgreSQL/PostGIS schema exists; `AppEnv` models logger and PostgreSQL
settings, but this scaffold does not yet construct a pool or select a runtime
database driver, HTTP framework, broker, or async runtime.

## Style and verification

The logger contract exposes error/warn/info/debug calls with `&AppContext` first,
then a tag, message, and optional borrowed metadata. `WriterLogger::new(writer,
format, level)` injects a `Write + Send` output, plain/JSON format, and maximum
verbosity. `NONE` disables all records. The adapter emits and flushes one line per
call, including `time_unix_ms`, `level`, `tag`, `msg`, `actor`, `execution_id`,
`trace_id`, and `meta`; absent context values become empty strings and absent
metadata becomes `{}`. Writes are synchronous and can block. Logger methods return
`()` and silently ignore emission failures without retries or recursive logging.
No process-global logger or environment loading is installed; callers supply
correlation identifiers through `AppContext`.

Follow the root Rust conventions, constructor injection, narrow visibility, and
classified errors. Use one focused capability per module, explicit request/result
structs for complex signatures, and private implementation details. Avoid blanket
abstractions, generic CRUD dispatch, and redundant trait layers.

When implementation changes, run from this directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Layer guides describe future behavior tests. Documentation-only changes require
content, link, and whitespace checks; they do not require introducing tests or
runtime dependencies. Migration/seeder and NGINX work remain deferred.
