ARG RUST_VERSION="1.83"
ARG RUST_IMAGE="rust:${RUST_VERSION}-bullseye"
ARG BASE_IMAGE="debian:bullseye"

# -- Install --
# We only pay the installation cost once,
# it will be cached from second build onwards

FROM ${RUST_IMAGE} as install

RUN apt-get -y update && apt-get -y upgrade
RUN apt-get install -y make git zlib1g-dev libssl-dev gperf php-cli cmake g++

# Install cargo chef
RUN cargo install cargo-chef --locked

WORKDIR /app

# -- Cache for builder --
FROM install as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# -- Builder with cache layer --
FROM install as builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --workspace --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --workspace

# -- Base Image --
# Base image for running tests and production image
FROM ${BASE_IMAGE} as base
RUN apt-get update && apt-get install -y ca-certificates

# -- Production Image --
FROM base as prod

WORKDIR /app
COPY --from=builder /app/target/release/server ./server
RUN ldconfig
COPY ./scripts ./scripts
COPY ./migrations ./migrations
COPY ./config.yaml ./config.yaml
COPY ./config.prod.yaml ./config.prod.yaml

ENTRYPOINT ["/app/scripts/entry.sh", "indexer"]