# Master service: composition

Construct concrete drivers, infrastructure, application, and presentation together.
Read the [service guide](../../AGENTS.md). Keep business rules and request handling
in their respective layers; composition is explicit wiring, not a service locator.

## Layout and startup

Use focused modules such as `driver.rs`, `infrastructure.rs`, `application.rs`,
`presentation.rs`, and `launcher.rs` when their responsibilities exist. Keep
`src/main.rs` thin and delegate launch to composition. Load and validate typed
settings through `src/config.rs`, then inject settings into constructors. Do not
copy Lockmate's Go global configuration pattern into Rust.

Initialize resources before constructing dependent adapters and usecases. Start
listeners/consumers only after required initialization succeeds. Retain handles
for owned tasks and resources. Do not add a database, event broker, async runtime,
or HTTP framework just to match a reference project's dependency list.

## Lifecycle and failures

Wire master-data presentation to domain usecase implementations and their
explicit dependencies. On shutdown, stop new requests, drain in-flight operations
within a bounded deadline, then close owned resources in dependency order. Do not
wire reaction submission or broadcast fan-out into this service by analogy.

Surface startup failures with an unsuccessful exit; normal signal-driven shutdown
should exit successfully. Apply a bounded shutdown deadline and clean up partially
initialized resources on failure. Do not dump settings or connection strings.
Master-database migrations and seeds belong in `master_database`; this scaffold
does not run them. NGINX setup remains deferred.

Test dependency construction, initialization failure cleanup, cancellation, and
shutdown ordering in `tests/integration/composition/` when implemented. Prefer
injected lifecycle controls over real process signals and timing-dependent sleeps
in tests.
