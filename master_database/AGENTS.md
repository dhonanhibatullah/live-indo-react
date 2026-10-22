# master_database

This standalone Rust 2024 crate owns the PostgreSQL/PostGIS reference schema and
geographic seed importer for live-indo-react. The schema contains reaction topics,
Indonesian provinces, cities/regencies, a singleton national boundary, polling
methods, and seed history. The current embedded
catalog contains 38 provinces and 514 cities/regencies across 45 JSON files, plus
three polling methods in version 46 and a national boundary in version 47;
there is no initial topic catalog. Counts describe this dataset, not permanent
administrative rules.

Follow the [root guide](../AGENTS.md) and use [README.md](README.md) for setup and
execution. This crate is a database preparation tool with a focused module layout;
do not impose the backend services' domain/application/infrastructure layers on it.
It has no HTTP server, reaction storage, authentication, or aggregation logic.

## Layout and responsibilities

| Path | Responsibility |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | Independent crate dependencies and reproducible resolution; there is no root Cargo workspace. |
| `src/main.rs` | Async lifecycle: app/environment loading, connection, migrations, schema verification, seeding, pool shutdown, and exit status. |
| `src/lib.rs` | Public module exports for the executable and integration tests. |
| `src/models/` | App/environment types, typed seed envelopes, province/city and polling-method records, catalog envelopes, and GeoJSON boundary types. |
| `src/config/` | Cargo metadata and optional working-directory `.env` loading with process-variable overrides. |
| `src/seeds/` | Embedded file metadata, decompression, typed JSON loading, and seed loading errors. |
| `src/database/connection.rs` | Typed PostgreSQL connection options, pool settings, and schema prerequisite checks. |
| `src/database/migrations.rs` | SQLx migration loading/execution, history checks, and safe diagnostics. |
| `src/database/mod.rs` | Seed orchestration, transactions, advisory locking, history checks, summaries, errors, and console output. |
| `src/database/0001_0007_provinces.rs` | Province preparation, existence queries, and inserts. |
| `src/database/0008_0045_cities.rs` | City/regency preparation, province-code validation, parent lookup, existence queries, and inserts. |
| `src/database/0046_polling_methods.rs` | Catalog preparation, existence checks by code, and conflict-safe inserts. |
| `build.rs` | Validates seed filenames/envelopes, hashes exact JSON bytes, and generates deterministic compressed assets in Cargo's `OUT_DIR`. |
| `seeds/` | Prettified source JSON, source attribution, and the upstream license. |
| `migrations/` | Ordered paired `.up.sql`/`.down.sql` schema changes; applied by the executable before seeding. |
| `docs/erd.dbml` | Human-maintained schema design; executable SQL lives in migrations. |
| `tests/` | All test implementations, including private-module tests under `tests/unit/`. |
| `Dockerfile`, `.dockerignore` | Multi-stage image running the single migrate-then-seed executable; exclude credentials and local artifacts. |

The numeric seed modules in `src/seeds/` use the same filenames as their
database counterparts. `#[path]` exposes them as `seeds::provinces`,
`seeds::cities`, `seeds::polling_methods`, and their `database` counterparts.

## Data and execution contracts

- Preserve the flow `config -> connection -> migrations -> schema check ->
  provinces -> cities -> polling methods -> close` in `main.rs`. Use the migration-capable connection
  before schema creation and verify prerequisites on the same pool afterward.
  Preserve SQLx checksums, migration locking, and `_sqlx_migrations` tracking;
  keep it separate from JSON `seed_history`. The database must already exist.
- Close the pool after migration/schema/seed success or failure and return a
  failing process status for configuration, connection, migration, or seed errors.
  Stop before seeding when migrations or schema verification fail.
- Embed compressed JSON at build time; deserialize one pending file at runtime.
  Keep decompression, deserialization, and geometry serialization off async worker
  threads with `spawn_blocking`. Do not load all boundary files at once.
- Serialize cooperating runners with the shared transaction advisory lock. Check
  history by version or filename before decoding; skip only when version,
  filename, and exact-byte SHA-256 agree. Conflicts must fail explicitly.
- Commit each file's rows and history together. Failed files roll back; earlier
  committed files remain available for a subsequent retry. Log row success only
  after commit. Preserve rollback diagnostics without replacing the original error.
- Check row existence by Kemendagri code, including inactive or soft-deleted rows.
  Preserve existing data; retain conflict-safe insertion and reselect behavior.
- Resolve city parents by province code, never assumed identity values. Validate
  the city code prefix and reject missing parents for new rows.
- Keep codes as strings and boundaries as 2D WGS84 MultiPolygons (SRID 4326,
  longitude then latitude). Preserve source attribution, license, and nullable
  reference dates. The schema does not enforce topological validity; the importer
  checks administrative codes/parents but not polygon validity or containment.
  Seed data is expected to be valid, and tests assert imported geometries are valid.
- `indonesia_boundary` is a singleton national boundary for country containment,
  including waters outside province boundaries. Seed version 47 contains one
  simplified Marine Regions EEZ/land union record with CC BY 4.0 attribution.
  Province boundaries are not a substitute for the national boundary.
- Applied seeds are immutable, including whitespace: checksums hash source bytes.
  Corrections need a new version and explicit update behavior; the current runner
  preserves existing rows. Adding versions also requires updating `build.rs`'s
  supported ranges/count and affected modules/tests, rather than just adding JSON.

## Rust and SQL style

Use snake_case functions/modules/fields, PascalCase types, and SCREAMING_SNAKE_CASE
constants. Keep functions focused, imports explicit, visibility narrow, and errors
represented by `Result`; avoid production `unwrap`, `expect`, and recoverable
panics. Keep composition in `main.rs`, configuration parsing in `config`, data
loading in `seeds`, and driver-specific work in `database`.

Use static SQL with bound parameters. Build connection options from typed fields,
not interpolated URLs. Preserve `DatabaseError` operation/file/code context and
native causes, while keeping console `Display`/`Debug` safe. Never print passwords,
connection strings, raw database error payloads, or geometry documents. Use the
existing simple console helpers (INFO to stdout, ERROR to stderr); console write
failures stay silent. No logging framework is required.

Keep SQL migrations ordered and paired with reverse operations. Preserve required
columns, stable identifiers, foreign keys, unique indexes, GiST boundary indexes,
and GIN trigram name indexes. Enable PostGIS and pg_trgm before creating their
indexes. Do not add CHECK constraints for master-data field rules. Update the ERD
alongside schema changes. Do not regenerate migrations blindly from DBML.

## Verification

Run Rust checks from this directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Keep all test bodies under `tests/`; private modules can include `tests/unit/`
files with `#[cfg(test)]` and `#[path]`. Test behavior, transaction boundaries,
safe diagnostics, and recovery rather than duplicating implementation details.

Live database tests require `MASTER_DATABASE_TEST_DATABASE_URL` pointing to a
dedicated disposable PostgreSQL server with PostGIS installed and a role able to
create/drop databases and create the extension. Tests create their own databases.
Report when these cases are skipped because the variable is absent. Never direct
them at a shared development or production server.

For documentation-only edits, check paths, commands against implementation, and
whitespace; Rust tests are not evidence of documentation accuracy.

## Polling metadata

The fresh workflow has seven schema migrations, 47 seed files, and 556 reference
rows. Keep geographic seed versions/bytes unchanged. Version 46 uses the
`CatalogSeedFile<T>` envelope (`version`, `name`, `records`) and the catalog loader,
sharing compression, checksums, advisory locking, and transaction/history behavior
with geographic seeds. It does not have SRID or upstream boundary metadata.

Topics have a required polling-method reference and ordered native `TEXT[]`
options. Migrations enforce required columns, the foreign key, and the array
column type. Do not add CHECK constraints or business validation triggers here:
the `master_service` validator owns code/slug/name/source formats and
method-dependent option checks, while application code owns method code
immutability and freezing polling configuration after responses exist. Selection
requires at least two distinct, nonblank, trimmed choices; other methods require
empty options. Rating is integer 0–10 and selection chooses one option. This crate
has no initial topic seed or response validation implementation.
