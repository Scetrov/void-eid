# Builder stage
FROM docker.io/library/rust:alpine AS builder

WORKDIR /usr/src/app

# Install build dependencies for musl
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig curl

# Install sccache
RUN curl -Lso /usr/local/bin/sccache https://github.com/mozilla/sccache/releases/download/v0.8.0/sccache-v0.8.0-x86_64-unknown-linux-musl.tar.gz \
    && tar -xvzf /usr/local/bin/sccache -C /usr/local/bin/ --strip-components=1 \
    && chmod +x /usr/local/bin/sccache

# Configure sccache
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ENV SCCACHE_GHA_ENABLED=on
ARG ACTIONS_CACHE_URL
ARG ACTIONS_RUNTIME_TOKEN

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build the application
# We need to touch the main file to force a rebuild
RUN touch src/main.rs
    --mount=type=cache,target=/root/.cache/sccache \
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
