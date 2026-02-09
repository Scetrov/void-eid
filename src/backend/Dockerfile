# Build stage
FROM docker.io/library/rust:latest AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime stage
FROM docker.io/library/debian:trixie-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary
COPY --from=builder /usr/src/app/target/release/void-eid-backend .
RUN chmod +x ./void-eid-backend

# Expose the API port
EXPOSE 5038

CMD ["./void-eid-backend"]
