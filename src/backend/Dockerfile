# Builder stage
FROM docker.io/library/rust:alpine AS builder

WORKDIR /usr/src/app

# Install build dependencies for musl
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig curl

# Install sccache
RUN curl -fsSL https://github.com/mozilla/sccache/releases/download/v0.8.0/sccache-v0.8.0-x86_64-unknown-linux-musl.tar.gz -o /tmp/sccache.tar.gz \
    && tar -xzf /tmp/sccache.tar.gz -C /tmp --strip-components=1 \
    && mv /tmp/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache \
    && rm -rf /tmp/sccache.tar.gz /tmp/LICENSE /tmp/README.md

# Configure sccache
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ARG SCCACHE_GHA_ENABLED=off
ENV SCCACHE_GHA_ENABLED=${SCCACHE_GHA_ENABLED}
ARG ACTIONS_CACHE_URL
ENV ACTIONS_CACHE_URL=${ACTIONS_CACHE_URL}
ARG SCCACHE_NO_DAEMON=0
ENV SCCACHE_NO_DAEMON=${SCCACHE_NO_DAEMON}
ARG RUST_BACKTRACE=0
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN,required=false \
    ACTIONS_RUNTIME_TOKEN=$([ -f /run/secrets/ACTIONS_RUNTIME_TOKEN ] && cat /run/secrets/ACTIONS_RUNTIME_TOKEN || echo "") cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build the application
# We need to touch the main file to force a rebuild
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN,required=false \
    touch src/main.rs && \
    ACTIONS_RUNTIME_TOKEN=$([ -f /run/secrets/ACTIONS_RUNTIME_TOKEN ] && cat /run/secrets/ACTIONS_RUNTIME_TOKEN || echo "") \
    SCCACHE_NO_DAEMON=1 RUST_BACKTRACE=1 cargo build --release && sccache --show-stats

# Runtime stage
FROM alpine:latest

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    sqlite-libs \
    libgcc

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/void-eid-backend .

# Expose the API port
EXPOSE 5038

# Run the binary
CMD ["./void-eid-backend"]
