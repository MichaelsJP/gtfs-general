FROM rust:1.78.0-slim AS chef

RUN cargo install cargo-chef

WORKDIR /usr/src/gtfs-general

FROM chef as utilities-planner
COPY utilities ./utilities
RUN cd utilities && cargo chef prepare --recipe-path recipe.json

FROM chef AS gtfs-general-planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS utilities-builder
COPY --from=utilities-planner /usr/src/gtfs-general/utilities/recipe.json utilities/recipe.json

RUN cd utilities && cargo chef cook --release --recipe-path recipe.json
COPY utilities utilities
RUN cd utilities && cargo build --release

FROM chef AS builder
COPY --from=planner /usr/src/gtfs-general/recipe.json recipe.json
COPY --from=utilities-builder /usr/src/gtfs-general/utilities/target utilities/target

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY src Cargo.toml Cargo.lock ./
RUN cargo build --release --bin gtfs-general

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /usr/src/gtfs-general/target/release/gtfs-general /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gtfs-general"]