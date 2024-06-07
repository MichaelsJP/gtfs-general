FROM rust:1.78.0-slim AS chef

RUN cargo install cargo-chef

WORKDIR /usr/src/gtfs-general

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/gtfs-general/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin gtfs-general

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /usr/src/gtfs-general/target/release/gtfs-general /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gtfs-general"]