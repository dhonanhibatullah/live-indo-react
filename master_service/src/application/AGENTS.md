# Master service: application

Implement domain usecases using injected domain contracts and models. Read the
[service guide](../../AGENTS.md). Do not import infrastructure, presentation,
composition, driver types, or transport frameworks.

## Layout and style

Group modules by capability. Keep dependencies and constructor near the
implementation struct; split orchestration methods into focused files only when
needed. Prefer private fields, explicit dependencies, early returns, and distinct
validation, execution, and result-mapping steps. Shared helpers belong here only
when multiple usecases actually need them; avoid universal dependency bundles.

## Orchestration

Validate master-data values and relationships before writing. Future topic
operations must enforce polling-method/option compatibility, stable option
ordering, and restrictions after responses exist. Resolve identifiers/codes through
contracts rather than assumed generated IDs. Define consistency checks across
services explicitly when implementing such features; no adapter exists yet.

Keep multi-step write boundaries explicit and propagate failures for rollback.
Do not introduce broadcasts, outbox behavior, or retries without a feature-level
contract. Keep success logs after the actual committed operation succeeds.

Return classified errors with useful causes. Log meaningful operation failures
and committed state changes through the chosen logging boundary, without raw
payloads or duplicate logs at every layer. Success logs must not precede the
operation's actual success. Preserve cancellation and do not spawn untracked work.

## Tests

Cover invalid master-data values/relationships, not-found/conflict cases,
dependency failures, and rollback when implemented in `tests/unit/application/`.
Use fakes for external capabilities; use an isolated database only for adapter
integration checks.
