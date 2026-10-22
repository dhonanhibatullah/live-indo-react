# live-indo proto

Shared protobuf and gRPC contracts for `master_service`.

## Layout

- `proto/live_indo_react/master/v1/common/`: shared models, geometry,
  pagination, and classified errors.
- `proto/live_indo_react/master/v1/management/`: strict master-data writes.
- `proto/live_indo_react/master/v1/geographic/`: province/city reads,
  coordinate resolution, and national-boundary checks.
- `proto/live_indo_react/master/v1/topic/`: public polling-method and topic reads.
- `gen/rust/`: checked-in tonic/prost bindings.
- `gen/react/`: checked-in protobuf-es and Connect ES TypeScript bindings.

The namespace is `live_indo_react.master.v1`. Services are unary and mirror the
domain usecase interfaces.

## Context metadata

Actor and trace context travel in gRPC metadata, not request messages:

- `x-trace-id`: required for every call; one non-zero UUID string.
- `x-actor`: optional opaque caller value; when present it must be one nonempty
  value. It is correlation context, never authentication or authorization.

The server maps these values into the request `AppContext`. Invalid or duplicate
values are `InvalidArgument` with application code `BAD_ARGS`. Generated types do
not contain actor or trace fields.

`execution_id` is not part of this transport contract yet.

## Generation

```sh
make lint
make generate
make test
```

Generation requires Buf on `PATH` and network access to the Buf Schema Registry.
Rust generation uses prost/tonic. React generation uses protobuf-es and Connect
ES. The React/TypeScript bindings are intended for Next server actions; there are
no browser-side gRPC calls, so no gRPC-Web proxy is required.

Commit both source contracts and generated bindings.
