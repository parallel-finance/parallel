# =============
# Computes the recipe file
FROM rust as planner
WORKDIR /app
RUN rustup default nightly-2021-03-04
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# =============
# Caches dependencies
FROM rust as cacher
WORKDIR /app
RUN rustup default nightly-2021-03-04 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-04
RUN cargo install cargo-chef
RUN apt update -y && \
    apt upgrade -y && \
	apt install -y cmake pkg-config libssl-dev git clang libclang-dev
COPY . .
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json "--$PROFILE"

# =============
# Builds binaries
FROM rust as builder
WORKDIR /app
RUN rustup default nightly-2021-03-04 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-04

ARG PROFILE=release

RUN apt update -y && \
    apt upgrade -y && \
	apt install -y cmake pkg-config libssl-dev git clang libclang-dev

COPY . .

COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME

RUN cargo build "--$PROFILE" --manifest-path node/parallel-dev/Cargo.toml

# =============

FROM debian:buster-slim
ARG PROFILE=release

RUN useradd -m -u 1000 -U -s /bin/sh -d /parallel parallel

COPY --from=builder /app/target/$PROFILE/parallel-dev /usr/local/bin
# COPY --from=builder /app/target/$PROFILE/parallel /usr/local/bin

USER parallel
EXPOSE 30333 9933 9944

RUN mkdir /parallel/data

VOLUME ["/parallel/data"]
