FROM docker.io/library/rust:1.97.1@sha256:1bcff4befb740599103a2c7cb51058e14479b2e35e3a34a3f0dc4ede09927488 AS rust-toolchain

FROM docker.io/oven/bun:1.3.14@sha256:e10577f0db68676a7024391c6e5cb4b879ebd17188ab750cf10024a6d700e5c4 AS bun-toolchain

FROM mcr.microsoft.com/playwright@sha256:02bbb2155cd7109e3e9c741941097ed1608cf8b6fa44ee2595896da2bdc1f471

COPY --from=rust-toolchain /usr/local/cargo /usr/local/cargo
COPY --from=bun-toolchain /usr/local/bin/bun /usr/local/bin/bun
COPY --from=rust-toolchain /usr/local/rustup /usr/local/rustup

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update \
    && apt-get install --no-install-recommends --yes build-essential libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
