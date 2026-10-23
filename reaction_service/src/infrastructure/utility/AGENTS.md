# Reaction service: utility infrastructure

Implement domain utility contracts with concrete libraries. Read the [service
guide](../../../AGENTS.md). Keep business orchestration and transport handlers out
of these adapters.

## Layout and behavior

Use one module per capability, explicit constructor dependencies, and private
implementation details. Keep provider payload conversion, connection management,
and error conversion next to the relevant adapter. Do not build a general helper
package for a single adapter. Read settings through injected typed configuration;
do not parse environment variables here.

The broadcast integration converts committed reaction data to the agreed external
event contract and reports delivery failures accurately. Do not perform reaction
writes here or publish tentative data. Set timeouts and bound retries when the
integration is implemented; retries must respect the documented duplicate/delivery
semantics rather than promise exactly-once behavior.

Preserve useful native causes behind domain errors without exposing provider
payloads or connection strings. Close connections and join owned background tasks
on shutdown. Test conversion, timeout/cancellation, unavailable dependencies,
resource cleanup, and the adapter's stated delivery behavior with isolated fixtures.

## Logger adapter

`logger/writer.rs` implements the domain logger through `WriterLogger<W>` with
injected `Write + Send`, format, and maximum verbosity. `logger/normalize.rs`
maps domain metadata into JSON values without mutating the caller. Nonfinite
floats normalize to null. Plain output quotes/escapes text; JSON output preserves
structured metadata. Both formats use one physical line per record.

Logger calls take `&AppContext` first. Emit `actor`, `execution_id` and `trace_id`
as top-level fields alongside existing fields in both formats. Missing values are
empty strings. Escape the actor in plain output to prevent injected log lines;
keep metadata nested so it cannot override context fields. Never log transaction
contents or mutate the caller's context. No automatic ID generation is installed.

Filtering precedes serialization and writer access, including `NONE`. Serialize
before locking; hold the writer mutex across the complete write and flush to
avoid interleaving concurrent calls. Silently discard serialization, write, flush,
and poisoned-lock failures; do not retry potentially partial writes or attempt
recursive error logging. The writer is synchronous, so callers must
account for blocking output in async paths. No global subscriber or trace
extraction is installed. Behavior tests live in `tests/logger.rs`.
