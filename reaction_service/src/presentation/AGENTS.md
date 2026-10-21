# Reaction service: presentation

Expose application usecases through transport adapters. Read the [service
guide](../../AGENTS.md). Use domain usecase interfaces; do not call repository or
utility contracts, concrete application implementations, or infrastructure.

## Layout and mapping

Organize `http/` into focused handlers, request/response DTOs or message
DTOs, explicit mappers, routes/registration, and transport helpers as needed.
Keep dependencies in constructors and handlers small. Parse transport shape here;
application validates business values. Map declared input fields explicitly so
server-owned fields cannot be overwritten through generic deserialization.

HTTP handlers accept anonymous submissions and translate explicit DTOs into
reaction usecase requests. Map results and domain error classifications to the
chosen HTTP contract. Do not invent routes, status policy, or input limits before
the API feature defines them. Enforce documented body/transport limits and preserve
request cancellation. Never report persistence failure as an accepted submission.

Only serialize safe wire DTOs, never persistence entities or diagnostic error
chains. Return sanitized operational failures. Preserve anonymous access without
copying Lockmate credential middleware. Keep transport protocol and event contract
changes coordinated with their consumers.

## Tests

Cover DTO mapping, malformed input, safe error responses, cancellation, and the
distinction between failed persistence and publication failure after commit.
