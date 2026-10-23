# Build the Actix central registry. Context is the repository root
# because `server` is a Cargo workspace member.
FROM rust:1.98-bookworm AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.toml
COPY core/Cargo.toml core/Cargo.toml
COPY cli/Cargo.toml cli/Cargo.toml
COPY desktop/Cargo.toml desktop/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY axon-android/rust/Cargo.toml axon-android/rust/Cargo.toml
COPY core/src core/src
COPY cli/src cli/src
COPY desktop/src desktop/src
COPY server/src server/src
COPY axon-android/rust/src axon-android/rust/src
RUN cargo build --release -p server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/axon-registry /usr/local/bin/axon-registry
ENV AXON_BIND=0.0.0.0:7400 \
    RUST_LOG=info \
    SECRET_KEY=Nishanth \
    DEBUG=false
EXPOSE 7400
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/axon-registry", "--healthcheck"]
ENTRYPOINT ["/usr/local/bin/axon-registry"]
