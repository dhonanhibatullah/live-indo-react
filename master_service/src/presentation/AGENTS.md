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
```

Per-service handlers are single files under their group; they are adapters, not
their own implementation folders. Mapper, interceptor, and registration folders
stay separate because they are shared transport infrastructure.

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

Use `tests/unit/presentation/grpc/` for mapping, metadata, and handler tests.
Use `tests/integration/presentation/grpc/` for tonic transport tests with fake
usecases; no database is needed.
