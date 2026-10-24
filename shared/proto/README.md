# live-indo proto

Shared protobuf and gRPC contracts for `master_service`.

## Layout

- `proto/live_indo_react/master/v1/common/`: shared models, geometry,
  pagination, and classified errors.
- `proto/live_indo_react/master/v1/management/`: strict master-data writes.
- `proto/live_indo_react/master/v1/geographic/`: province/city reads,
  coordinate/IP region resolution, and national-boundary checks.
- `proto/live_indo_react/master/v1/topic/`: public polling-method and topic reads.
- `gen/rust/`: checked-in tonic/prost bindings.
- `gen/react/`: checked-in Protobuf-ES v2 messages and service descriptors.

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
Rust generation uses prost/tonic. TypeScript generation uses Protobuf-ES v2
(pinned in buf.gen.yaml), which includes service descriptors for Connect v2.
Import GeographicRegionService from `geographic/region_pb.ts`; legacy
`*_connect.ts` files are removed because the Connect v1 plugin is incompatible
with the existing v2 message output. See the
[Connect migration guide](https://github.com/connectrpc/connect-es/blob/main/MIGRATING.md). The React/TypeScript bindings are intended for Next server actions; there are
no browser-side gRPC calls, so no gRPC-Web proxy is required.

Commit both source contracts and generated bindings.

## Region lookup by IP

GeographicRegionService.GetRegionByIp takes `{ip: "203.0.113.1"}` and returns an
optional GeoIP estimate and optional canonical Region. The explicit lookup IP
accepts IPv4/IPv6 host addresses, not CIDRs, ports or forwarded-header contents.
Invalid input is InvalidArgument/BAD_ARGS and is not echoed in diagnostics.
Existing actor/trace metadata requirements apply to this method.

GeoIp contains optional country/continent/city/postal/time-zone values, ordered
provider subdivisions, optional estimated coordinates and accuracy radius in km.
Provider subdivision identifiers are not Kemendagri codes. Unknown IPs return
both response fields absent; estimates can exist without coordinates or region
matches. Optional scalar presence preserves zero and empty values. The response
does not include the lookup IP, credentials or native provider errors.

GeoIP errors have appended ErrorCode values 34–40; existing values/tags remain
unchanged. Not-ready/unavailable map to gRPC Unavailable, timeout maps to
DeadlineExceeded, and configuration/database/download/operation failures map to
Internal. GeographicRegionService.GetRegionByCoord remains available unchanged.
