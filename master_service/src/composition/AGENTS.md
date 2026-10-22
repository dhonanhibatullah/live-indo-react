# Master service: composition

Construct concrete drivers, infrastructure, application, and presentation together.
Read the [service guide](../../AGENTS.md). Keep business rules and request handling
in their respective layers; composition is explicit wiring, not a service locator.

## Layout and startup

Composition roots live beside each other under `src/composition/`; the current
server root is `main/`. A future composition can add another root without mixing
its wiring into `main/`.

Focused `main/` modules own one part of the runtime:

- `main/driver.rs`: structured stdout logger and PostgreSQL/SQLx pool construction.
- `main/infrastructure.rs`: Moka cache, validator, context factory, and cached
  repository adapters over one shared `Pgdt`.
- `main/application.rs`: usecase implementations and their explicit dependencies.
- `main/presentation.rs`: HTTP router and gRPC `Routes` built from the same usecases.
- `main/launcher.rs`: settings loading, startup ordering, listeners, signals, and
  bounded cleanup.

Keep `src/main.rs` thin and delegate to `composition::launch`. Typed settings come
from `src/config/app.rs` and `src/config/env.rs`; composition injects them into
constructors. Do not parse environment variables outside the config module or copy
Lockmate's Go global configuration pattern.

Initialize resources before constructing dependent adapters and usecases. Build
the database pool before infrastructure, and build usecases before presentation.
Bind both listeners before logging readiness. Do not run migrations or seeds here;
`master_database` prepares the schema. Do not add a broker or unrelated service
wiring by analogy.

## Lifecycle and failures

HTTP and gRPC listen on separate configured addresses and share usecase instances,
repositories, and cache state. Both drain on SIGINT/SIGTERM within the configured
deadline, then the PostgreSQL pool closes. If a listener or server fails, stop the
other transport and close the pool before returning failure.

Surface configuration, database, bind, serve, and shutdown failures with a
non-zero exit; normal signal-driven shutdown exits zero. Force-close remaining
tasks when the drain deadline expires. Do not dump settings, passwords,
connection strings, or request data.

`launch_with` accepts an injected shutdown future for tests. Prefer local
listeners, fake usecases, and channels over real process signals and sleeps.
