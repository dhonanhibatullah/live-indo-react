# reaction_database

This standalone Rust 2024 crate prepares the PostgreSQL/PostGIS database used by
live-indo-react's reaction service. It owns ordered schema migrations for one
`reactions` table and its location-source enum. It deliberately inserts no
production data: reactions come from users through `reaction_service`.

Follow the [root guide](../AGENTS.md). See [README.md](README.md) for setup and
execution, and [docs/erd.dbml](docs/erd.dbml) for schema semantics. Follow the
configuration, migration, error, logging, testing, and packaging patterns of
`master_database` without importing its internal code or adding a shared crate.
There is no root Cargo workspace.

## Layout and responsibilities

| Path | Responsibility |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | Independent crate dependencies and reproducible resolution. |
| `src/main.rs` | Async lifecycle: app metadata, environment loading, connection, migrations, schema checks, pool shutdown, and exit status. |
| `src/lib.rs` | Public config/database/model modules for the executable and tests. |
| `src/models/` | App identity and typed, password-redacted environment settings. |
| `src/config/` | Compile-time Cargo metadata and optional working-directory `.env` loading with process-variable overrides. |
| `src/database/connection.rs` | Typed PostgreSQL options, bounded pool, migration-capable and schema-verified connections. |
| `src/database/migrations.rs` | SQLx migration loading, locking, checksum/history verification, execution, and safe diagnostics. |
| `src/database/mod.rs` | Database error type and simple console helpers. |
| `migrations/` | Ordered paired `.up.sql`/`.down.sql` files: PostGIS first, then reactions and the location-source enum. |
| `docs/erd.dbml` | Human-maintained schema design; executable SQL lives in migrations. |
| `tests/` | All test implementations, including private-module tests under `tests/unit/`. |
| `Dockerfile`, `.dockerignore` | Multi-stage build and single nonroot runtime executable; exclude secrets and local artifacts. |

Do not add unused seed modules, `seed_history`, a build script, compression
machinery, or synthetic production reactions. There is no reference catalog here.
Keep test rows inside disposable integration-test databases. Reaction ingestion,
repositories, HTTP endpoints, geolocation, GIF providers, and broadcasts belong to
services, not this preparation tool. Add models and dependencies when a feature
actually needs them, rather than mirroring every master-database file.

## Runtime and schema contracts

- Preserve `config -> connect for migrations -> migrate -> verify schema -> close`.
  The target database must already exist. The executable enables the installed
  PostGIS extension but does not install server packages or create databases.
- Keep all orchestration in `main.rs`. Close the pool after migration/verification
  success or failure; return a failing process status for any failed stage.
- Preserve SQLx migration checksums, locking, per-migration transactions, and
  `_sqlx_migrations` history. Do not silently rewrite applied history, ignore
  checksum conflicts, or infer a baseline for manually created schemas.
- Load SQL from the working directory's `migrations/` folder. Connections use
  typed fields rather than interpolated DSNs; disable driver statement logging.
- Keep `polling_method_code` flexible indexed text, not an enum or hard-coded
  method-to-answer CHECK. Exactly one current answer column must be populated;
  disagreement (`false`), rating zero, and selection zero are valid values.
- Selections are zero-based integer indexes. Preserve paired optional GIF provider
  and content-ID references; never replace them with cached media URLs.
- Preserve the unquoted SQL type `geometry(Point,4326)`, nonempty finite-coordinate
  checks, and GiST index. Longitude is X; latitude is Y. Raw IP and coordinates are
  internal metadata, never public feed fields or routine log content.
- Preserve both partial feed indexes with `WHERE deleted_at IS NULL`, the method
  B-tree index, and `GENERATED ALWAYS AS IDENTITY`. Do not blindly export DBML
  into migrations: it cannot express every SQL detail correctly.
- Topic IDs and regional codes are external master references; add no
  cross-database foreign keys or local reference-table copies. The service owns
  topic/method/option validation, geographic admission, master-record existence,
  original-payload immutability, URL/provider rules, and payload limits.
- Add no business triggers, accounts, aggregates, deduplication policy, outbox,
  partitioning, or delivery guarantees without a feature requirement.
- Keep reverse migrations paired and dependency ordered. PostGIS removal uses
  `RESTRICT` so unrelated dependent objects are not silently dropped.

## Code style and errors

Use idiomatic Rust 2024: snake_case functions/modules/fields, PascalCase types,
SCREAMING_SNAKE_CASE constants, explicit imports, focused functions, early returns,
and narrow visibility. Keep parsing in config, data vocabulary in models, driver
operations in database, and composition in main. Do not impose the services'
domain/application/infrastructure layers on this database tool.

Return `Result` for recoverable failures. Avoid production `unwrap`, `expect`, and
panics. Preserve native causes and operation/SQLSTATE/constraint metadata while
keeping error `Display`/`Debug` safe for console output. Never print raw driver
errors, credentials, rejected environment values, IPs, geometry, or reaction text.
Use static SQL and bound parameters. Do not add a logger framework: INFO goes to
stdout, ERROR to stderr, and console write failures stay silent.

## Verification

Run from this directory for Rust changes:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All test implementations belong under `tests/`; private production modules may
include files in `tests/unit/` using `#[cfg(test)]` and `#[path]`. Cover behavior,
constraints, safe diagnostics, rollback/recovery, and lifecycle outcomes.

Live integration tests require `REACTION_DATABASE_TEST_DATABASE_URL` for a
separate disposable PostgreSQL server with PostGIS available. The role must be
able to create/drop databases and create PostGIS extensions. Each case creates
and drops its own database. Never target a shared development or production
server; report skipped live cases when the variable is absent.

Docker changes require a build and a disposable-PostGIS smoke test: fresh setup,
rerun, empty reactions, and failure status. Documentation-only changes require
content, path/link, and whitespace checks; Rust tests do not establish prose
accuracy.
