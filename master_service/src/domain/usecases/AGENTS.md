# Master service: domain usecases

Define application-facing behavior and explicit request/result values. Read the
[service guide](../../../AGENTS.md). Application implements these interfaces;
presentation depends on them rather than concrete implementations.

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
