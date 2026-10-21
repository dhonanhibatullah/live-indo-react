# Broadcast service

Receive accepted reaction events and deliver them to matching province/topic
subscribers, primarily through WebSocket connections.

Read the [root guide](../AGENTS.md) first. This is an independent Rust 2024 crate
with a binary entrypoint and a library exposing domain and infrastructure modules.
Generic domain errors, logger models, the logger utility contract, and its
writer adapter are implemented. JSON formatting uses `serde_json` only in
infrastructure. `src/main.rs` remains a
placeholder. The rest of the layout below is the target architecture, not an
implemented broadcast runtime.

## Target source layout

```text
src/
  lib.rs                        # domain and infrastructure library boundary
  main.rs                       # thin entrypoint delegating to composition
  config.rs                     # typed settings and environment loading
  domain/
    models/                     # domain vocabulary and classified errors
    contracts/                  # repository and utility traits
    usecases/                   # application traits and request/result values
  application/                  # usecase implementations, grouped by capability
  infrastructure/
    repository/                 # concrete persistence adapters, if needed
    utility/                    # external integrations and technical adapters
  presentation/
    websocket/                  # transport handlers and explicit wire DTOs
  composition/                  # dependency construction and lifecycle
tests/                          # integration tests (at crate root)
```

Use Rust module declarations only as implementation files are added; do not create
empty modules to satisfy this tree. Keep capability-specific code together and
split files by responsibility when needed. Do not add a repository implementation
to a service without an actual persistence requirement.

## Scoped guides

- Domain: [models](src/domain/models/AGENTS.md),
  [contracts](src/domain/contracts/AGENTS.md),
  [usecases](src/domain/usecases/AGENTS.md).
- [Application](src/application/AGENTS.md).
- Infrastructure: [repositories](src/infrastructure/repository/AGENTS.md),
  [utilities](src/infrastructure/utility/AGENTS.md).
- [Presentation](src/presentation/AGENTS.md).
- [Composition](src/composition/AGENTS.md).

## Service behavior and boundaries

Accepted reaction events enter through an integration adapter, are checked and
routed by application usecases, and reach only subscriptions matching their province
and topic. Presentation owns WebSocket protocol handling; application owns routing
rules; infrastructure utilities own concrete event-source and fan-out mechanisms
behind contracts. Keep socket objects out of domain and application types.

This service does not create reactions or bypass the reaction service write path.
Bound outgoing queues and define an explicit slow-consumer policy when delivery is
implemented. Isolate a slow or disconnected subscriber from other subscribers, and
release its subscriptions, tasks, and buffers on disconnect or cancellation.

Use domain-only values at layer boundaries. Keep province and topic scope explicit
in request, result, event, and subscription types as applicable. Coordinate event
contract changes across both services without importing their internal code.
The database, broker, runtime, transport framework, endpoint names, wire schemas,
identifier formats, and delivery guarantees are not selected by these guides.
Record such decisions alongside the feature that introduces them.

## Style and verification

The logger contract exposes error/warn/info/debug calls with a tag, message, and
optional borrowed metadata. `WriterLogger::new(writer, format, level)` injects a
`Write + Send` output, plain/JSON format, and maximum verbosity. `NONE` disables
all records. The adapter emits and flushes one line per call, including
`time_unix_ms`, `level`, `msg`, `tag`, and `meta`; absent metadata becomes `{}`.
Writes are synchronous and can block. Logger methods return `()` and silently
ignore emission failures without retries or recursive logging.
No process-global logger, environment loading, or tracer integration is installed.

Follow the root Rust conventions, constructor injection, narrow visibility, and
classified errors. Use one focused capability per module, explicit request/result
structs for complex signatures, and private implementation details. Avoid blanket
abstractions, generic CRUD dispatch, and redundant trait layers.

When implementation changes, run from this directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Layer guides describe future behavior tests. Documentation-only changes require
content, link, and whitespace checks; they do not require introducing tests or
runtime dependencies. Migration/seeder and NGINX work remain deferred.
