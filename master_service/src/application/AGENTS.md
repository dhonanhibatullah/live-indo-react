# Master service: application

Implement domain usecases using injected domain contracts and models. Read the
[service guide](../../AGENTS.md). Do not import infrastructure, presentation,
composition, driver types, or transport frameworks.

## Layout and style

Group modules by capability. Each usecase capability gets its own folder with a
`mod.rs` and an implementation file such as `usecase.rs`, mirroring infrastructure
adapter grouping. Keep dependencies and constructors near the implementation
struct; split orchestration methods into focused files only when needed. Prefer
private fields, explicit dependencies, early returns, and distinct validation,
execution, and result-mapping steps. Shared helpers belong here only when multiple
usecases actually need them; avoid universal dependency bundles. Shared logging
helpers live in `shared/log.rs`.

## Capability groups

- `management/` validates and executes strict topic, polling-method, province,
  city, and national-boundary writes. It resolves referenced records and maps
  explicit usecase requests to repository arguments.
- `geographic/` maps repository reads to province/city results, coordinate
  resolution, and national-boundary checks.
- `topic/` maps public catalog reads and applies active/nondeleted filters.

Application failures aggregate domain validation and repository classifications
through `UsecaseError`; preserve their codes and sources rather than flattening
them into strings.

## Orchestration

Validate master-data values and relationships before writing. Future topic
operations must enforce polling-method/option compatibility, stable option
ordering, and restrictions after responses exist. Resolve identifiers/codes through
contracts rather than assumed generated IDs. Define consistency checks across
services explicitly when implementing such features; no adapter exists yet.

Keep multi-step write boundaries explicit and propagate failures for rollback.
Do not introduce broadcasts, outbox behavior, or retries without a feature-level
contract. Keep success logs after the actual committed operation succeeds.
Management success tags use `management/<entity>/<operation>` and safe identifier
metadata such as `topic_id` or `city_id`, never raw request payloads.

Return classified errors with useful causes. Log meaningful operation failures
and committed state changes through the chosen logging boundary, without raw
payloads or duplicate logs at every layer. Adapters and validator already log
their failures; application logs only errors it creates itself, such as a missing
referenced record with `error_code` metadata. Use `log_repository_error` for
application-created repository classifications and `log_validator_error` only when
the validator implementation does not already own the failure log. Success logs
must not precede the operation's actual success. Preserve cancellation and do not spawn untracked work.

## Tests

Cover invalid master-data values/relationships, not-found/conflict cases,
dependency failures, and rollback when implemented in `tests/unit/application/`.
Use fakes for external capabilities; use an isolated database only for adapter
integration checks.
