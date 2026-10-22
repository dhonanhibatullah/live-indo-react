# Build from the repository root because this crate consumes the shared workspace:
# docker build -f master_service/Dockerfile -t live-indo-master-service .
FROM rust:1.94-bookworm AS builder

WORKDIR /build
COPY master_service/Cargo.toml master_service/Cargo.lock ./master_service/
COPY master_service/src/ ./master_service/src/
COPY shared/Cargo.toml ./shared/
COPY shared/crates/pgdt/ ./shared/crates/pgdt/
COPY shared/proto/gen/rust/ ./shared/proto/gen/rust/

WORKDIR /build/master_service
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=builder --chown=10001:10001 /build/master_service/target/release/master_service /app/master_service

USER 10001:10001
EXPOSE 8080 8081
STOPSIGNAL SIGTERM
ENTRYPOINT ["/app/master_service"]
