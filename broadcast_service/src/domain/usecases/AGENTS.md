# Broadcast service: domain usecases

Define application-facing behavior and explicit request/result values. Read the
[service guide](../../../AGENTS.md). Application implements these interfaces;
presentation depends on them rather than concrete implementations.

## Layout and boundaries

Group interfaces by capability, with the trait followed by its request/result
types. Keep methods focused and province/topic scope explicit. Use domain types,
`Option`, and `Result`; exclude HTTP status codes, wire DTOs, sockets, database
transactions, and provider SDK types. Declare only behavior the feature needs.

Usecases cover receiving accepted events and subscribing/unsubscribing to explicit
province/topic scope as needed. Define delivery/subscription results without exposing
a socket or concrete channel. Treat disconnect and cancellation as lifecycle inputs;
never interpret a subscription as a request to persist a new reaction.

## Verification

Keep business validation in application and document the intended outcomes on
interfaces. Exercise implementations through these interfaces with test doubles
for external capabilities, covering invalid scope and dependency failures. Changing
an interface requires updating its callers, implementation, and tests together.
