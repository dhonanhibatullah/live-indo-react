# Master service: composition

Construct concrete drivers, infrastructure, application, and presentation together.
Read the [service guide](../../AGENTS.md). Keep business rules and request handling
in their respective layers; composition is explicit wiring, not a service locator.

## Layout and startup

Composition roots live beside each other under `src/composition/`; the current
server root is `main/`. A future composition can add another root without mixing
its wiring into `main/`.

Focused `main/` modules own one part of the runtime:

- `main/driver.rs`: structured stdout logger, PostgreSQL/SQLx pool and Pgdt construction.
- `main/infrastructure.rs`: async builder for Moka cache, validator, context
  factory, cached repositories and GeoIP configuration/local-reader initialization.
- `main/application.rs`: usecase implementations and their explicit dependencies.
- `main/presentation.rs`: HTTP router, gRPC Routes and cron registration over injected usecases.
- `main/launcher.rs`: thin driver -> infrastructure -> application -> presentation
  wiring and delegation to the lifecycle runner.
- `main/startup.rs`: one awaited refresh through the already-wired maintenance
  usecase, followed by GeoIP readiness verification.
- `main/runner.rs`: entrypoint/settings delegation, lifecycle reporting, listener
  binding, HTTP/gRPC/cron supervision, signals and bounded pool cleanup.

Keep `src/main.rs` thin and delegate to `composition::launch`. Typed settings come
from `src/config/app.rs` and `src/config/env.rs`; composition injects them into
constructors. Do not parse environment variables outside the config module or copy
Lockmate's Go global configuration pattern.

Wire driver, infrastructure, application and presentation in that order. The
async infrastructure builder initializes GeoIP internally without downloading.
After all presentation objects exist, startup calls the shared application's
RefreshGeoIp instance, then checks readiness against the same GeoIP service.
Use the wired CreateAppContext usecase for startup correlation metadata. Do not
construct another maintenance implementation or run refresh inside a layer builder.
A refresh error, failed readiness check or ten-minute deadline fails startup.
Only after startup succeeds may runner bind listeners or start cron workers.
Bind both listeners before logging readiness. Do not run migrations or seeds here;
`master_database` prepares the schema. Do not add a broker or unrelated service
wiring by analogy.

## Lifecycle and failures

HTTP and gRPC listen on separate configured addresses and share usecase instances,
repositories, and cache state. Cron starts after both listeners bind and shares
one GeoIP service between maintenance and IP region resolution. Its daily job
runs at 00:00 UTC after the one awaited startup refresh. The scheduler does not
repeat that initial refresh when it starts. GeoIP file paths come from
AppEnv, and real credentials remain in the ignored GeoIP.conf.

HTTP, gRPC and cron drain on SIGINT/SIGTERM within the configured deadline, then
the PostgreSQL pool closes. On scheduler/server failure, stop all presentations
and close the pool before returning failure. The runner also closes the pool
after infrastructure construction or startup failure, because drivers are already
constructed. Pool close has the configured shutdown deadline; preserve the initial
failure if cleanup also times out. Cron emits no job-level logs.

Surface configuration, database, bind, serve, and shutdown failures with a
non-zero exit; normal signal-driven shutdown exits zero. Force-close remaining
tasks when the drain deadline expires. Do not dump settings, passwords,
connection strings, or request data.

`launch_with` accepts an injected shutdown future for tests. Prefer local
listeners, fake usecases, and channels over real process signals and sleeps.
