# Master service: caching infrastructure

Implement the domain cache contract with concrete in-memory or remote backends.
Read the [service guide](../../../AGENTS.md). Keep business policy and transport
handling out of these adapters.

## Layout and behavior

Use `cache/` for the cache backend group, with one module per backend such as
`cache/moka.rs` for the process-local implementation and a future `cache/redis.rs`
for a shared implementation. `cache/mod.rs` declares the private backend modules
and re-exports their concrete adapter types under explicit backend names;
`caching/mod.rs` declares the group and re-exports the public adapter names.
Constructors receive typed configuration; do not parse environment variables here.

Cache values remain type-erased `CacheValue` domain values. Do not serialize,
inspect, or log cached data. Reads must preserve the remaining entry lifetime;
writes select the configured TTL through `CacheKind`.

## Moka adapter

`cache/moka.rs` implements `Cache` through `MokaCache::new(max_capacity, individual_ttl,
list_ttl)` using `moka::sync::Cache`. One bounded capacity covers both entry kinds.
A private entry stores `CacheKind` plus `CacheValue`; a private `Expiry`
implementation returns the configured individual or list TTL on creation and
update. Reads keep the remaining TTL.

Moka performs TinyLFU eviction and per-entry expiration internally. Cache
operations are synchronous in-memory work and must not be wrapped in surrounding
synchronous locks across repository I/O. TTL expiration is owned by Moka; private
tests assert the expiry callbacks directly rather than sleeping. Behavior tests
live in `tests/unit/infrastructure/caching/cache.rs`.
