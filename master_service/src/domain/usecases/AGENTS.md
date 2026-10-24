# Master service: domain usecases

Define application-facing behavior and explicit request/result values. Read the
[service guide](../../../AGENTS.md). Application implements these interfaces;
presentation depends on them rather than concrete implementations.

## Capability groups

- `maintenance/` exposes operational upkeep independently of its trigger.
  `RefreshGeoIp` ensures the local GeoIP database is ready and current.
- `management/` contains strict write operations. Topics and regions expose
  explicit create/update/delete requests; polling methods are seed-created and
  support update/delete; the singleton national boundary supports update only.
- `geographic/` retrieves provinces and cities, resolves a coordinate to a
  province plus optional city, resolves estimated IP locations, and checks the
  national boundary.
- `topic/` exposes the available polling-method catalog and active topic reads.
- `misc/` exposes presentation-facing request context creation. `CreateAppContext`
  receives actor and trace values; the application implementation supplies the
  service-local execution ID.

Use `shared/future.rs` for the object-safe `UsecaseFuture` and `shared/page.rs`
for offset pagination. Request types are explicit per capability; do not expose
repository arguments or generic CRUD. Management requests omit server-owned IDs
on creation and immutable slugs/codes/parent links on update.

## Layout and boundaries

Group interfaces by capability, with the trait followed by its request/result
types. Keep methods focused and master-data scope explicit. Use domain types,
`Option`, and `Result`; exclude HTTP status codes, wire DTOs, sockets, database
transactions, and provider SDK types. Declare only behavior the feature needs.

Requests express master-data operations through explicit domain values. They
cannot overwrite server-owned identifiers or audit fields. Results contain only
values suitable for presentation mapping. Distinguish invalid input, missing
master records, conflicts, and operational failures as features are introduced.
Do not introduce generic CRUD interfaces or endpoints without a requirement.

## Verification

Keep business validation in application and document the intended outcomes on
interfaces. Exercise implementations through these interfaces with test doubles
for external capabilities, covering invalid scope and dependency failures. Changing
an interface requires updating its callers, implementation, and tests together.

## Region lookup

`geographic/region.rs` exposes the object-safe `GetRegion` trait with
`get_region_by_coord` and `get_region_by_ip`. Coordinate lookup returns
`Option<Region>`; a region contains a required province and optional city.
IP lookup takes `IpAddr` and returns `RegionByIp` with an optional GeoIP estimate
and optional matched region. Missing provider records yield both absent; missing
coordinates or boundary matches preserve the estimate with no region. Do not map
provider names/codes directly to Kemendagri codes, substitute zero coordinates,
or enforce country admission here. GeoIP failures propagate through UsecaseError.

## GeoIP maintenance

`maintenance/geoip.rs` defines context-first, object-safe `RefreshGeoIp`, returning
`UsecaseFuture<()>`. An unready service updates immediately; a ready service first
checks for a new release and updates only when needed. Both an already-current
database and a successful refresh return unit; this does not assert a download
occurred because the adapter rechecks under its update lock. Propagate GeoIP errors
through UsecaseError, retaining codes and causes. No scheduling, retries or cron
presentation is part of this interface.
