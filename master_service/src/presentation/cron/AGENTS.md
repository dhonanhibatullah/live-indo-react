# Master service: cron presentation

Read the [presentation guide](../AGENTS.md), [service guide](../../../AGENTS.md),
and [usecase guide](../../domain/usecases/AGENTS.md). Cron invokes domain usecase
interfaces only. It must not call repositories, service/utility contracts,
concrete application implementations or adapters directly.

## Layout

- `job.rs`: stable job ID, daily UTC time, execution deadline and private handler boundary.
- `runner.rs`: serial workers, daily scheduling, context creation and shutdown draining.
- `handler/<capability>/<job>.rs`: thin typed adapters delegating to a usecase.
- `register/<capability>.rs`: job definitions and handler dependency registration.
- `mod.rs`: public runner/registration boundaries. Composition owns start/stop and task handles.

The first job is `maintenance.geoip.refresh`, registered daily at 00:00 UTC with a
ten-minute execution limit. It invokes RefreshGeoIp, which owns readiness/update
policy. Composition wires every layer before startup awaits the existing
refresh usecase and verifies readiness; only then does the runtime runner start
listeners and scheduling. The cron runner waits for the next 00:00 UTC to avoid
duplicating that startup invocation. There is one worker per registered job; a slow job does not overlap itself or enqueue missed daily runs. Failed and
timed-out executions wait for the next daily trigger. There are no immediate
retries or catch-up runs. Future groups are added only with their first job.

Create a fresh context through CreateAppContext per execution, with actor
`cron/<job-id>`, no supplied trace ID and no transaction. The runner has no Logger
dependency and emits no job-level logs, including timeout logs. Operation logs
remain in application/infrastructure. Never introduce duplicate cron logging.

Recheck the UTC wall clock at least once per minute while waiting. Clock injection
supports deterministic tests; Tokio supplies cancellable timers. Stop admitting
executions on shutdown or channel closure; let an in-flight execution drain.
Composition cancels remaining tasks when its shared shutdown deadline expires.
Dropping the runner cancels its workers; an already-started blocking GeoIP install
may complete according to the adapter's documented cancellation semantics.

Tests cover registration, daily timing, context freshness, overlap, failure,
timeout and draining with paused Tokio time. Keep doubles in tests/unit and test
private calendar arithmetic inline. No real sleeps, credentials, downloads or
GeoIP files are needed for scheduler tests.
