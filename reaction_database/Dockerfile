# Build with: docker build -t live-indo-reaction-database ./reaction_database
FROM rust:1.94-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=builder /build/target/release/reaction_database /app/reaction_database
COPY migrations/ /app/migrations/

USER 10001:10001
ENTRYPOINT ["/app/reaction_database"]
