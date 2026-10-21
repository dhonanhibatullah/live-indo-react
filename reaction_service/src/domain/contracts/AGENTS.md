# Reaction service: domain contracts

Define the external capabilities application needs. Read the [service
guide](../../../AGENTS.md); implementations belong in infrastructure.

## Layout and declarations

Group persistence traits under `repository/` and external technical capabilities
under `utility/` as they are introduced. Use one focused module per capability.
Declare the trait before related argument and result structs. Use meaningful Rust
names, explicit province/topic scope, and `Result` for failures. Expose domain
values rather than drivers, sockets, SDK errors, or framework contexts.

Introduce persistence and accepted-reaction publication capabilities only as needed.
Keep transaction ownership in application while adapters encapsulate driver state.
A transaction abstraction must preserve one commit boundary without leaking native
transaction types. Do not imply that publication shares the database transaction.

## Compatibility and tests

Choose generics or object-safe traits deliberately. For async I/O, make future
bounds and dispatch compatibility explicit when implementation requires them;
use the smallest dependency surface that works. Document cancellation, failure,
and retry semantics on operations that need them. Align traits, implementations,
callers, and test doubles together. Contract tests should check observable behavior
rather than the chosen adapter library.

## Logger contract

`utility/logger.rs` defines the object-safe `Logger: Send + Sync` trait. The core
`log` method accepts a domain level, tag, message, and optional borrowed metadata;
error/warn/info/debug methods delegate to it. All calls return `()`; emission
failures are silently ignored. Filtered calls produce no output. Metadata must not be mutated; absent metadata is an empty object. Unlike
Lockmate's Go contract, no context/tracer parameter is introduced before a tracing
capability exists. Infrastructure supplies the implementation; application can
inject `&dyn Logger` or `Arc<dyn Logger>` as its ownership needs dictate.
