# Master service: repository infrastructure

Implement domain repository contracts when persistence is required. Read the
[service guide](../../../AGENTS.md). The master schema uses PostgreSQL/PostGIS; a runtime driver is not selected.
This guide does not authorize migrations, seeders, or a new database dependency.

## Layout and style

Group adapters by entity/capability. When SQL is introduced, separate query
construction (for example `query.rs`) from execution, row mapping, and error
conversion (for example `adapter.rs`). Keep constructors explicit and fields
private. Bind values rather than interpolating input; adapter-owned identifiers
and ordering must not come from unchecked user strings.
Map persistence rows into domain values explicitly. Preserve master-data filters,
pagination, and deletion predicates throughout scoped operations. Translate native
failures into `RepositoryError`: use specific repository codes for recognized
not-found, uniqueness, and relation failures; use `Timeout`/`Failure` for
operational failures while retaining the native source; use generic buckets only
when no specific classification exists. Keep those native causes behind
`std::error::Error::source` and do not expose driver text in Display. Do not log
SQL values, raw payloads, or connection strings, and do not add business
validation or service orchestration.

## Transactions and resources

Honor the commit boundaries specified by the selected domain contracts.
Use the same transaction for related writes, propagate failures, and release driver
resources on every path. A successful query is not evidence of a committed write.
Repositories must not trigger broadcasts or unrelated external side effects.

## Tests

Test scope predicates, row mapping, not-found behavior where applicable, and error
conversion. Use isolated integration resources for real database checks. Report
unavailable database tests as skipped; never modify a developer's live database
for verification. Keep migrations and seeders outside this work's scope; executable schema changes belong in `master_database`.
