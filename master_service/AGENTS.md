# Master service

Provide the service boundary for live-indo-react master data: topics, polling
methods, provinces, and cities/regencies. Use the master database schema as the
data reference; database preparation remains in `master_database`.

Read the [root guide](../AGENTS.md) first. This is an independent Rust 2024 crate
with a binary entrypoint and a library exposing domain and infrastructure modules.
Generic domain errors, validator and repository error models, logger models, the
logger utility contract and its writer adapter, the validator contract and
regex-backed adapter, and the master-data cache contract and Moka adapter are
implemented. Entity models, repository contracts, cached repository decorators,
and app context/config models for topics, polling methods, provinces, and cities
are implemented. Repository contracts return object-safe boxed `RepositoryFuture`
values. Province and city coordinate lookups and the national-boundary
repository contract, PostgreSQL adapter, and cached decorator are implemented.
Domain usecase contracts and application implementations are grouped under
management, geographic, topic, and misc capabilities. The misc context usecase
accepts actor/trace metadata and delegates execution-ID generation to the utility
factory.
PostgreSQL/PostGIS repository adapters for all four master entities use
`shared/crates/pgdt` and SQLx, including the `AppContext` transaction bridge.
Validator field-format, nonblank, selection-option, and city/province-code rules
replace the former master-table CHECK constraints. JSON formatting uses
`serde_json` only in infrastructure. `AppEnv` includes cache capacity plus
separate individual and list TTLs, but configuration loading and composition are
not implemented. `src/main.rs` remains a placeholder. The remaining modules below
are the target architecture, not an implemented master-data runtime.

## Target source layout

```text
src/
  lib.rs                        # domain and infrastructure library boundary
  main.rs                       # thin entrypoint delegating to composition
  config.rs                     # typed settings and environment loading
  domain/
    models/                     # domain vocabulary and classified errors
    contracts/                  # repository, caching, and utility traits
    usecases/                   # application traits and request/result values
  application/                  # usecase implementations, grouped by capability
  infrastructure/
    cache/                      # cache backend adapters
    repository/                 # concrete persistence and cached adapters
    utility/                    # external integrations and technical adapters
  presentation/
    http/                       # transport handlers and explicit wire DTOs
  composition/                  # dependency construction and lifecycle
tests/
  unit/                         # black-box unit tests by source layer
  integration/                  # external-boundary tests
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
- Infrastructure: [caching](src/infrastructure/caching/AGENTS.md),
  [repositories](src/infrastructure/repository/AGENTS.md),
  [utilities](src/infrastructure/utility/AGENTS.md).
- [Presentation](src/presentation/AGENTS.md).
- [Composition](src/composition/AGENTS.md).
- [Tests](tests/AGENTS.md).

## Service behavior and boundaries

Presentation maps transport input to domain usecase requests. Application owns
business validation and calls injected domain contracts. Infrastructure implements
those contracts against selected drivers or providers. Usecase implementations are
available, but HTTP routes and composition wiring are not implemented by this
scaffold.

The master schema lives in [master_database/docs/erd.dbml](../master_database/docs/erd.dbml).
The validator preserves stable topic slugs and regional/method code formats and
checks polling-method-dependent selection options. Future features must keep topic
polling configuration fixed after responses exist. `selection_options` is an
ordered native string array; reaction selection indexes depend on that ordering.
There are no database CHECK constraints or business triggers for these policies,
and implementing cross-service checks needs an explicit feature design.

Reaction submission belongs to `reaction_service`; WebSocket subscriber handling
and fan-out belong to `broadcast_service`. Do not import another service's internal
modules. Expose explicit domain types at boundaries and coordinate future external
contracts with consumers. Do not add authentication, routes, write permissions,
cache behavior beyond the explicit master-data cache contract, or publication
guarantees by analogy; those are feature decisions.
The PostgreSQL/PostGIS schema, SQLx/`pgdt` repository adapters, Moka cache adapter,
cached repository decorators, and national-boundary adapter exist; `AppEnv` models
logger, PostgreSQL, and cache settings, but this scaffold does not yet construct a
pool or cache in composition, load environment settings, or select an HTTP
framework, broker, or runtime lifecycle. `master_database` seeds the
`indonesia_boundary` singleton from a simplified Marine Regions EEZ/land union with
CC BY 4.0 attribution.

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

The validator contract is a synchronous, object-safe `Validator: Send + Sync`
trait. Every method takes `&AppContext` first and a borrowed field value, then
returns `Result<(), ValidatorError>`; option and city/province rules take explicit
related values. `RegexValidator::new(Arc<dyn Logger>)` compiles the format patterns
and logs each rejected value once at ERROR level with safe `error_code` and
`reason` metadata. Valid input does not log. Validation never mutates inputs or
performs I/O. Boundary topology is not validated by the service or the schema.

The cache contract is a synchronous, object-safe `Cache: Send + Sync` trait in
`domain/contracts/caching/`. `CacheValue` type-erases owned domain values without
serialization. `CacheKind::{Individual, List}` selects the adapter TTL.
`MokaCache::new(max_capacity, individual_ttl, list_ttl)` in
`infrastructure/caching/cache/moka.rs` uses Moka with one bounded capacity and
per-entry expiration. Cached repository decorators perform cache-aside reads and
invalidate the entity namespace on writes. Province and city coordinate reads use
the same cache path with coordinate-bit keys. Reads in an active `AppContext`
transaction bypass the cache. A write-scoped drop guard invalidates on success,
failure, or cancellation; successful transactional writes still invalidate before
commit, so TTLs bound that local race.

`IndonesiaBoundaryRepository` is a repository contract backed by
`PostgresIndonesiaBoundary` and `CachedIndonesiaBoundary`. It exposes singleton
get/update operations and `is_inside`; the latter validates WGS84 coordinates and
queries the `indonesia_boundary` table with `ST_Covers`, independent of province
and city coverage. Cache tests live in `tests/unit/domain/cache_contract.rs`,
`tests/unit/infrastructure/caching/cache.rs`, and
`tests/unit/infrastructure/repository/`; national-boundary integration behavior
lives in `tests/integration/infrastructure/repository/`; private expiry and guard
tests stay inline.

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

Layer guides and the [tests guide](tests/AGENTS.md) describe behavior tests.
Documentation-only changes require content, link, and whitespace checks; they do
not require introducing tests or runtime dependencies. Migration/seeder and NGINX
work remain deferred.
