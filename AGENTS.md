# live-indo-react

live-indo-react is a public, unauthenticated platform for Indonesians to express
reactions and emotions about Indonesia's current situation. The intended interface
shows a province-based heat distribution on an Indonesia map, summarized information
on hover, and province detail views containing individual reaction/comment bubbles
and quickly updated feeds. All views are grouped by topic.

## Project state and scope

[INITIAL_PLAN.md](INITIAL_PLAN.md) is the project brief. The architecture below adapts
the explicit clean architecture and distributed guidance of `../mate/lockmate` to
Rust; the sibling checkout is a design reference, not a runtime dependency.

Both backend services are Rust 2024 crates with implemented generic domain errors,
logger models, logger utility contracts, and writer adapters. Their `src/main.rs`
entrypoints remain placeholders. The remaining documented source layouts are
targets, not implemented modules. Do not describe proposed behavior as already
available. Add source files and dependencies only when implementing a
feature that needs them.

- `reaction_service/`: accepts and inserts anonymous reactions/comments. Read its
  [service guide](reaction_service/AGENTS.md).
- `broadcast_service/`: broadcasts accepted reactions, primarily over WebSockets.
  Read its [service guide](broadcast_service/AGENTS.md).

Preserve anonymous participation. Do not import Lockmate's accounts, permissions,
JWTs, client credentials, or authentication flows into this project by analogy.
Province and topic scope must remain explicit across submission, event delivery,
and display. Aggregation rules for heat distribution and summaries are unspecified;
do not invent formulas or assign them to a service without a feature requirement.

## Guidance and dependency boundaries

Read this file and every applicable descendant `AGENTS.md` before editing. Read
related layer guides for changes crossing interfaces. Find guides with
`rg --files -g AGENTS.md`. Scoped guides refine this overview; explicit user
instructions take precedence. Keep guides and their links accurate as code evolves.

Each Rust service follows these boundaries:

- Domain models define vocabulary and errors; contracts define external
  capabilities; usecases define application behavior and request/result types.
- Application implements usecases using injected domain contracts.
- Infrastructure implements contracts using concrete drivers and libraries.
- Presentation parses transport input and calls domain usecase interfaces.
- Composition constructs concrete dependencies and owns process lifecycle.

Domain must not import application, infrastructure, presentation, or composition.
Application must not import concrete adapters or transport types. Presentation
must not call repository/utility contracts or concrete application implementations.
Composition is the place that connects concrete implementations across layers.
Each service builds independently; there is no root Cargo workspace. Do not import
another service's internal modules or introduce a shared crate merely for symmetry.

## Shared Rust conventions

Use idiomatic Rust 2024: snake_case modules, functions, and fields; PascalCase types
and traits; SCREAMING_SNAKE_CASE constants. Prefer explicit imports, focused files,
early returns, and blank lines between logical steps. Keep visibility narrow;
prefer private or `pub(crate)` items unless a public boundary requires otherwise.
Use constructors with explicit dependencies rather than global service locators.
Unlike Go interfaces, Rust constructors can return concrete implementations;
choose generics or trait objects according to the actual dispatch needs.

Use owned/borrowed values deliberately, `Option` for absence, and `Result` for
fallible operations. Preserve useful error causes and classify domain failures;
transport maps them to safe responses. Avoid production `unwrap`, `expect`, and
panics for recoverable failures. Never log raw comments, complete request bodies,
secrets, or configuration dumps; use operation and correlation metadata instead.

Keep blocking I/O and heavy computation off async executor threads. Do not hold
synchronous lock guards across `.await`. Propagate cancellation and bound queues,
concurrency, retries, and shutdown waits where those capabilities are introduced.
Keep async trait signatures compatible with the chosen dispatch strategy; do not
assume native async methods support `dyn Trait` or add a macro dependency by default.
Keep framework and driver types outside domain interfaces.

## Verification

For Rust implementation changes, run from each affected service directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Use inline unit tests for private behavior and `tests/` for integration behavior;
keep test doubles outside production modules. Cover behavior and failures rather
than merely duplicating implementation. External integration tests must use
isolated resources; state when they were skipped or unavailable.

Documentation-only changes need guide discovery, content, relative-link, and
whitespace checks. Do not run service tests as evidence of documentation accuracy.
