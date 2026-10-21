# Reaction service: repository infrastructure

Implement domain repository contracts when persistence is required. Read the
[service guide](../../../AGENTS.md). The database and driver are not selected;
this guide does not authorize migrations, seeders, or a new database dependency.

## Layout and style

Group adapters by entity/capability. When SQL is introduced, separate query
construction (for example `query.rs`) from execution, row mapping, and error
conversion (for example `adapter.rs`). Keep constructors explicit and fields
private. Bind values rather than interpolating input; adapter-owned identifiers
and ordering must not come from unchecked user strings.

Map persistence rows into domain values explicitly. Preserve province/topic
predicates throughout scoped operations. Translate native failures into domain
classifications while retaining useful diagnostic causes. Do not log SQL values,
raw comments, or connection strings, and do not add business validation or fan-out.

## Transactions and resources

Honor application-owned commit boundaries through the selected contract design.
Use the same transaction for related writes, propagate failures, and release driver
resources on every path. A successful query is not evidence of a committed write.
Publication belongs to application orchestration, never a repository side effect.

## Tests

Test scope predicates, row mapping, not-found behavior where applicable, and error
conversion. Use isolated integration resources for real database checks. Report
unavailable database tests as skipped; never modify a developer's live database
for verification. Keep migrations and seeders outside this work's scope.
