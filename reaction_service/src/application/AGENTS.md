# Reaction service: application

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

Validate reaction/comment values and province/topic scope before writing. Own the
logical transaction boundary for multi-step persistence and propagate failures so
the adapter can roll back. Invoke publication only for committed reactions. Keep
commit success distinct from subsequent publication failure; implement a documented
partial-failure policy rather than silently claiming full success or blindly
retrying the write. Do not introduce an outbox, broker, or retry policy solely
because this guide mentions delivery failures.

Return classified errors with useful causes. Log meaningful operation failures
and committed state changes through the chosen logging boundary, without raw
comments or duplicate logs at every layer. Success logs must not precede the
operation's actual success. Preserve cancellation and do not spawn untracked work.

## Tests

Cover invalid province/topic and reaction values, dependency failures, rollback,
no publication after failed writes, and commit followed by publication failure.
Use fakes to verify operation order without requiring a real broker or database.
