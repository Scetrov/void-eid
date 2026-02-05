# Builder stage
FROM docker.io/library/rust:latest AS builder

WORKDIR /usr/src/app

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
FROM docker.io/library/debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies if needed (e.g. openssl, ca-certificates, sqlite3 lib)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/void-eid-backend .

# Copy any necessary runtime files (e.g. static assets, config)
# COPY .env .env # We'll use env vars passed to container for production/dev

# Expose the API port
EXPOSE 5038

# Run the binary
CMD ["./void-eid-backend"]
