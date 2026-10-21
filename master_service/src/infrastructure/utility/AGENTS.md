# Master service: utility infrastructure

Implement domain utility contracts with concrete libraries. Read the [service
guide](../../../AGENTS.md). Keep business orchestration and transport handlers out
of these adapters.

## Layout and behavior

Use one module per capability, explicit constructor dependencies, and private
implementation details. Keep provider payload conversion, connection management,
and error conversion next to the relevant adapter. Do not build a general helper
package for a single adapter. Read settings through injected typed configuration;
do not parse environment variables here.

Add master-data external integrations only for implemented capabilities. Keep
provider-specific conversion and operational behavior here, while application
owns policy and orchestration. Set timeouts and bounded retries when an operation
requires them, respecting its stated idempotency and cancellation semantics.

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

Filtering precedes serialization and writer access, including `NONE`. Serialize
before locking; hold the writer mutex across the complete write and flush to
avoid interleaving concurrent calls. Silently discard serialization, write, flush,
and poisoned-lock failures; do not retry potentially partial writes or attempt
recursive error logging. The writer is synchronous, so callers must
account for blocking output in async paths. No global subscriber or trace
extraction is installed. Behavior tests live in `tests/logger.rs`.
