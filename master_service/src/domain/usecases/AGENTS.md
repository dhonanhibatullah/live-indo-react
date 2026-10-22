# Master service: domain usecases

Define application-facing behavior and explicit request/result values. Read the
[service guide](../../../AGENTS.md). Application implements these interfaces;
presentation depends on them rather than concrete implementations.

## Capability groups

- `management/` contains strict write operations. Topics and regions expose
  explicit create/update/delete requests; polling methods are seed-created and
  support update/delete; the singleton national boundary supports update only.
- `geographic/` retrieves provinces and cities, resolves a coordinate to a
  province plus optional city, and checks the national boundary.
- `topic/` exposes the available polling-method catalog and active topic reads.

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
