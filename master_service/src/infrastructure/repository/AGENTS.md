# Master service: repository infrastructure

Implement domain repository contracts through the selected PostgreSQL/PostGIS
driver. Read the [service guide](../../../AGENTS.md). Migrations and seeders remain
in `master_database`; do not add schema changes, business validation, or broadcasts
here.

## Layout and style

Adapters are grouped by entity with `mod.rs`, `postgres.rs`, and
`postgres_query.rs`. Shared query predicates, scan functions, GeoJSON conversion,
and error classification live under `shared/`. Concrete adapters use
`Postgres<Entity>` names and are re-exported from `repository/mod.rs`.

Repository traits return the object-safe `RepositoryFuture<'a, T>` from
`domain/contracts/repository/future.rs`. Implementations return
`Box::pin(async move { ... })`; do not add an async-trait macro. Every method takes
`&AppContext` first and executes through `pgdt`, including its transaction handle
when present. Constructors inject `Pgdt` plus `Arc<dyn Logger>`.

Separate query construction from execution, row mapping, and error conversion.
Bind values rather than interpolating input; adapter-owned identifiers and ordering
must not come from unchecked user strings. Map rows explicitly into domain values.
GeoJSON boundaries are read with `ST_AsGeoJSON` and structurally validated with
`Boundary::new`; writes use `ST_GeomFromGeoJSON` with SRID 4326 and an explicit
MultiPolygon cast. Do not add topology checks or repair invalid geometry.

## Query and deletion semantics

`find_by_*` includes soft-deleted rows. `list` excludes them unless
`include_deleted` is set. `update` and `delete` target only non-deleted rows.
`delete` sets `deleted_at` without releasing unique slugs/codes. Empty update
arguments return the current row without changing `updated_at`; nonempty updates
set it with `NOW()`. Lists use deterministic `name ASC, id ASC` ordering and bind
the contract pagination directly. Name filters use escaped, literal `ILIKE`
contains matching.

Preserve parent relationships through the existing foreign keys and map their
constraint names to entity-specific relation errors. A soft-deleted parent remains
physically referenceable.

## Transactions and resources

Honor the commit boundaries specified by the domain contracts. Use the same
transaction for related writes, propagate failures, and release driver resources
on every path. A successful query is not evidence of a committed write.
Repositories must not trigger broadcasts or unrelated external side effects.

## Errors and logging

Translate native failures into `RepositoryError`: recognized SQLSTATE/constraint
pairs become specific not-found/conflict/relation codes; argument-shape SQLSTATEs
become `BadArgs`; pool and I/O timeouts become `Timeout`; other failures become
`Failure`. Preserve native causes only behind `std::error::Error::source`, and do
not expose driver text in `Display`.

Log each failed operation once at ERROR level with the operation tag, stable error
code, SQLSTATE, and constraint name. Tags use exactly
`repository/<entity>/<function>`, without a driver segment. Never log SQL values,
raw payloads, boundary data, or connection strings.

## Tests

Test scope predicates, pagination, row mapping, soft-delete behavior,
not-found/conflict/relation conversion, GeoJSON round-trips, transaction commit and
rollback, and error logging that excludes input values. Repository integration
tests live in `tests/integration/infrastructure/repository/` and create and drop
isolated PostgreSQL/PostGIS databases. They use
`MASTER_DATABASE_TEST_DATABASE_URL` and report a skip when it is unavailable; never
modify a developer's live database for verification. Private helper tests stay
inline beside the implementation. Executable schema changes belong in
`master_database`.
