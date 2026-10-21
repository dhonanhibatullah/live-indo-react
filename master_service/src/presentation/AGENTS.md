# Master service: presentation

Expose application usecases through transport adapters. Read the [service
guide](../../AGENTS.md). Use domain usecase interfaces; do not call repository or
utility contracts, concrete application implementations, or infrastructure.

## Layout and mapping

Organize `http/` into focused handlers, request/response DTOs or message
DTOs, explicit mappers, routes/registration, and transport helpers as needed.
Keep dependencies in constructors and handlers small. Parse transport shape here;
application validates business values. Map declared input fields explicitly so
server-owned fields cannot be overwritten through generic deserialization.

Handlers map explicit master-data DTOs into domain usecase requests and map
results/errors to the selected transport contract. Do not invent routes, input
limits, write permissions, or access policy before the API feature defines them.
Enforce the chosen transport limits and preserve cancellation when implemented.
Never report a failed persistence operation as success.

Only serialize safe wire DTOs, never persistence entities or diagnostic error
chains. Return sanitized operational failures. Do not import Lockmate credential middleware by analogy; feature requirements
must define access policy without changing anonymous reaction participation. Keep transport protocol and event contract
changes coordinated with their consumers.

## Tests

Cover DTO mapping, malformed input, safe error responses, cancellation, and the
mapping of invalid input, missing records, conflicts, and persistence failures.
