# syntax=docker/dockerfile:1

# Builder stage
FROM docker.io/library/rust:alpine AS builder

WORKDIR /usr/src/app

# Install build dependencies for musl
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build the application
# Touch main.rs to invalidate the dummy build and force a recompile of our code
# while reusing cached dependencies from the layer above
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    touch src/main.rs && cargo build --release

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
