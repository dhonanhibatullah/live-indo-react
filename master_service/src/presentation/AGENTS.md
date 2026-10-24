# Master service: presentation

Expose application usecases through transport adapters. Read the [service
guide](../../AGENTS.md). Use domain usecase interfaces; do not call repository or
utility contracts, concrete application implementations, or infrastructure.

## Layout and mapping

```text
grpc/
  server/
    shared/context.rs
    management/<service>.rs
    geographic/<service>.rs
    topic/<service>.rs
  mapper/
    entity.rs geometry.rs audit.rs request.rs error.rs
  interceptor/app_context.rs
  register/{management,geographic,topic}.rs

http/
  handler/
    management/<resource>.rs
    geographic/<resource>.rs
    topic/<resource>.rs
  dto/
    request/...
    response/{model.rs,management.rs,geographic.rs,topic.rs,error.rs}
  routes/{management,geographic,topic}.rs
  middleware/{context,recovery}.rs
```

Per-service handlers are single files under their group; they are adapters, not
their own implementation folders. Mapper, interceptor, DTO, middleware, and
registration folders stay separate because they are shared transport
infrastructure.

Handlers:
- Implement generated tonic traits with `#[tonic::async_trait]`.
- Extract `AppContext` from request extensions.
- Map proto requests to usecase requests, call the usecase, and map the result.
- Contain no business rules, persistence access, or duplicate logging.

Map generated messages through explicit mapper functions. Never serialize domain
entities directly. Map `UsecaseError` once through `mapper/error.rs` to a safe
gRPC status and typed `Error` detail; never expose source chains.

## Metadata contract

Actor and trace context travel in gRPC metadata, not request messages:

- `x-trace-id`: required, one non-zero UUID string.
- `x-actor`: optional opaque caller value, at most once.

The context interceptor calls the `misc/app_context` usecase, which generates the
service-local execution ID, then inserts `AppContext` into request extensions.
The actor is correlation context, never authentication or authorization. No
client credentials, bearer tokens, IP allowlists, or browser gRPC/WebSockets are
introduced.

## Registration

`register/` combines generated service implementations into `tonic::service::Routes`.
Composition constructs usecases and applies the context interceptor; presentation
does not start the server or own process lifecycle.

## HTTP presentation

Axum routes live under `/v1`. Management writes use explicit resource routes;
geographic and topic reads use separate lookup paths such as `by-code` and
`by-slug`. Request DTOs reject unknown fields and carry only declared,
client-controlled values. Response DTOs are safe wire models; boundaries use
GeoJSON `MultiPolygon`. List responses are `{data, page}` and do not invent totals.

The HTTP pipeline is recovery → context → handler. Context uses the same
`x-trace-id`/`x-actor` metadata contract as gRPC and inserts `AppContext` into
request extensions. `ApiError` maps `UsecaseError` to safe JSON
`{code, message}` responses with stable HTTP statuses. Body size is limited to
1 MiB. There is no authentication, CORS, Swagger, health route, or server startup
in this step.

Use `tests/unit/presentation/grpc/` and `tests/unit/presentation/http/` for
mapping, metadata, and handler tests. Use
`tests/integration/presentation/{grpc,http}/` for transport tests with fake
usecases; no database is needed.

## Cron presentation

Read the scoped [cron guide](cron/AGENTS.md). Cron mirrors handler and registration
groups with shared job/runner modules. The maintenance GeoIP handler calls
RefreshGeoIp daily at 00:00 UTC, starting at the next occurrence after startup.
Composition separately awaits one maintenance refresh before opening listeners.
Context creation uses the CreateAppContext usecase; no utility contract is called
from presentation. There are no job-level logs; application/infrastructure own
operation logs. Composition starts the runner after both listeners bind and owns
its bounded shutdown alongside HTTP/gRPC.
