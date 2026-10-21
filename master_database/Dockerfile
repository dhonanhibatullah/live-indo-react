# Build with: docker build -t live-indo-master-database ./master_database
FROM rust:1.94-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ ./src/
COPY seeds/ ./seeds/
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=builder /build/target/release/master_database /app/master_database
COPY migrations/ /app/migrations/
COPY seeds/LICENSE.laravel-nusa /app/licenses/LICENSE.laravel-nusa

USER 10001:10001
ENTRYPOINT ["/app/master_database"]
