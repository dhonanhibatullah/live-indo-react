# Master service: repository infrastructure

Implement domain repository contracts through the selected PostgreSQL/PostGIS
driver. Read the [service guide](../../../AGENTS.md). Migrations and seeders remain
in `master_database`; do not add schema changes, business validation, or broadcasts
here.

## Layout and style

Adapters are grouped by entity with `mod.rs`, `postgres.rs`,
`postgres_query.rs`, and `cached.rs`. Shared query predicates, scan functions,
GeoJSON conversion, cache namespaces, and error classification live under
`shared/`. Concrete PostgreSQL adapters use `Postgres<Entity>` names; cache-aside
decorators use `Cached<Entity>` names. Both are re-exported from
`repository/mod.rs`.

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

Coordinate lookups use the existing GiST boundary indexes and
`ST_Covers(boundary, ST_SetSRID(ST_MakePoint(longitude, latitude), 4326))`. Filter
`deleted_at IS NULL AND is_active`, order overlaps by `id ASC` with `LIMIT 1`, and
validate finite longitude/latitude ranges as `BadArgs` before querying. Boundary
points are included; no extra spatial index is required for these queries.

Preserve parent relationships through the existing foreign keys and map their
constraint names to entity-specific relation errors. A soft-deleted parent remains
physically referenceable.

## Cached decorators

`Cached<Entity>` wraps `Arc<dyn EntityRepository>` and `Arc<dyn Cache>`. All
`find_by_*` methods use `CacheKind::Individual`; `list` uses `CacheKind::List`.
Reads are cache-aside: return a hit, otherwise call the inner repository and cache
only successful values. Do not cache `None`; empty lists are successful list
results. Recheck the namespace generation before inserting so a concurrent write
cannot repopulate an invalidated generation. Reads carrying any `AppContext`
transaction bypass the cache entirely.

Writes always call the inner repository and are never cached. A write-scoped
`InvalidationGuard` advances the entity generation on drop, covering successful,
failed, and canceled writes. Generation bumps make old keys unreachable; Moka TTL
and capacity eventually evict them. Do not hold locks across `.await`, and do not
add distributed or commit-aware invalidation here. Successful transactional writes
invalidate before commit; TTLs bound the resulting local race. Keys use
`master:v1:<entity>:g<generation>:<kind>:<discriminator>`; list keys hash every
filter and pagination field. Composition must construct one decorated instance per
entity; separate decorator instances have independent generations.

Test cache hits, misses, filter-key separation, write invalidation, transaction
bypass, and cancellation using in-memory fakes in
`tests/unit/infrastructure/repository/`. Exercise coordinate containment and
active/nondeleted filtering through PostgreSQL integration tests in
`tests/integration/infrastructure/repository/`.

## Indonesia boundary

`indonesia_boundary/` implements the singleton `IndonesiaBoundaryRepository`
through `PostgresIndonesiaBoundary` and `CachedIndonesiaBoundary`. The row is
created by `master_database`; this layer does not create it. `get` returns
`Option`, `update` handles partial boundary/source/reference-date changes, and
`is_inside` validates WGS84 coordinates and queries with `ST_Covers`. A missing
row makes `is_inside` false and makes `update` return
`INDONESIA_BOUNDARY_NOT_FOUND`; invalid coordinates return `BAD_ARGS`.

The cached decorator stores `get` under the singleton ID key and caches both true
and false `is_inside` results by coordinate bits. `update` uses the standard
write-scoped invalidation guard. Transaction reads bypass the cache. Use tag
`repository/indonesia_boundary/<operation>` and preserve the same error/logging
rules as other repositories.

Test get/update/containment in
`tests/integration/infrastructure/repository/indonesia_boundary.rs` and cache
behavior in `tests/unit/infrastructure/repository/indonesia_boundary.rs`.

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
