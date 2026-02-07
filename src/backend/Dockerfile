# Builder stage
FROM docker.io/library/rust:alpine AS builder

WORKDIR /usr/src/app

# Install build dependencies for musl
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig curl

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build the application
# We need to touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release

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
