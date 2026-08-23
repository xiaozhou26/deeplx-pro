FROM rust:1.88-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends clang cmake libclang-dev ninja-build perl pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home deeplx

COPY --from=builder /app/target/release/deeplx-pro /usr/local/bin/deeplx-pro

ENV HOST=0.0.0.0 \
    PORT=9000

EXPOSE 9000
USER 10001
ENTRYPOINT ["/usr/local/bin/deeplx-pro"]
