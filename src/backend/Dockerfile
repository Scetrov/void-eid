FROM docker.io/library/debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary
COPY void-eid-backend .
RUN chmod +x ./void-eid-backend

# Expose the API port
EXPOSE 5038

CMD ["./void-eid-backend"]
