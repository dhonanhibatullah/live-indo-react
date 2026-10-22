# Master service: domain contracts

Define the external capabilities application needs. Read the [service
guide](../../../AGENTS.md); implementations belong in infrastructure.

## Layout and declarations

Group persistence traits under `repository/` and external technical capabilities
under `utility/` as they are introduced. Use one focused module per capability.
Declare the trait before related argument and result structs. Use meaningful Rust
names, explicit entity/filter scope, and `Result` for failures. Expose domain
values rather than drivers, sockets, SDK errors, or framework contexts.

Introduce master-data persistence and external lookup capabilities only when a
feature needs them. Every repository method takes `&AppContext` first, then the
explicit entity/filter scope. Repository contracts return `RepositoryError`, using
specific not-found/conflict codes where the repository owns that classification
and generic buckets otherwise. The validator contract returns `ValidatorError`.
Adapters must route execution through the context's opaque transaction handle
without exposing native transaction types to callers. Do not add a transaction
abstraction, publication contract, or distributed consistency guarantee just for
symmetry.

## Compatibility and tests

Choose generics or object-safe traits deliberately. Master repository traits are
object-safe through `RepositoryFuture<'a, T>`, a boxed `Send` future with an
explicit context/self lifetime. This supports `Arc<dyn Repository>` composition
without an async-trait macro. For async I/O, make future bounds and dispatch
compatibility explicit when implementation requires them; use the smallest
dependency surface that works. Document cancellation, failure,
and retry semantics on operations that need them. Align traits, implementations,
callers, and test doubles together. Contract tests should check observable behavior
rather than the chosen adapter library.

## Logger contract

`utility/logger.rs` defines the object-safe `Logger: Send + Sync` trait. The core
`log` method accepts `&AppContext` first, then a domain level, tag, message, and
optional borrowed metadata; error/warn/info/debug methods delegate to it. All
calls return `()`; emission failures are silently ignored. Filtered calls produce
no output. Metadata must not be mutated; absent metadata is an empty object.
Infrastructure supplies the implementation; application can inject `&dyn Logger`
or `Arc<dyn Logger>` as its ownership needs dictate.

## Validator contract

`utility/validator.rs` defines the object-safe `Validator: Send + Sync` trait.
Methods are synchronous, take `&AppContext` first, and return
`Result<(), ValidatorError>`. Field methods cover polling-method code/name, topic
slug/name, province code/name/boundary source, and city code/name/boundary source.
`topic_selection_options` takes the polling-method code plus ordered labels, and
`city_province_code` checks the city prefix against the parent province code.
Implementations must not mutate inputs, retain caller data, or perform I/O.
Failures are classified with stable domain codes; logging is best effort and
never changes the result. Boundary topology is outside this contract.
