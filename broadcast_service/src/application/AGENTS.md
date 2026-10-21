# Broadcast service: application

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

Validate event and subscription scope, then route accepted events to matching
province/topic subscribers. Depend on contracts for event input and delivery; do
not manipulate concrete sockets or maintain a hidden process-global registry.
Bound concurrent delivery and avoid one subscriber holding up all others. Handle
closed subscriptions, disconnects, cancellation, and queue saturation explicitly;
implement and document the chosen slow-consumer policy with the delivery feature.

Return classified errors with useful causes. Log meaningful operation failures
and committed state changes through the chosen logging boundary, without raw
comments or duplicate logs at every layer. Success logs must not precede the
operation's actual success. Preserve cancellation and do not spawn untracked work.

## Tests

Cover province/topic routing isolation, no-subscriber events, subscription removal,
slow and failed consumers, queue saturation, and cancellation. Verify that one
subscriber failure does not prevent other matching subscribers from receiving.
Use controlled test doubles rather than timing-dependent sleeps.
