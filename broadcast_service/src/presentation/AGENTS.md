# Broadcast service: presentation

Expose application usecases through transport adapters. Read the [service
guide](../../AGENTS.md). Use domain usecase interfaces; do not call repository or
utility contracts, concrete application implementations, or infrastructure.

## Layout and mapping

Organize `websocket/` into focused handlers, request/response DTOs or message
DTOs, explicit mappers, routes/registration, and transport helpers as needed.
Keep dependencies in constructors and handlers small. Parse transport shape here;
application validates business values. Map declared input fields explicitly so
server-owned fields cannot be overwritten through generic deserialization.

WebSocket handlers own upgrades, frame parsing/serialization, and connection I/O.
Map subscription messages into explicit province/topic usecase requests. Enforce
documented frame limits, keepalive/timeouts, and queue policy once selected. Keep
per-connection writes coordinated; disconnect, malformed input, and cancellation
must clean up subscriptions and owned tasks. Slow clients must not create
unbounded buffers or block unrelated connections. Accepted event ingestion may
use another adapter; do not expose public reaction creation through WebSockets.

Only serialize safe wire DTOs, never persistence entities or diagnostic error
chains. Return sanitized operational failures. Preserve anonymous access without
copying Lockmate credential middleware. Keep transport protocol and event contract
changes coordinated with their consumers.

## Tests

Cover subscription message mapping, province/topic isolation, malformed frames,
slow/disconnected consumers, safe error frames, cancellation, and cleanup after
connection closure. Use protocol integration tests when the server is implemented.
