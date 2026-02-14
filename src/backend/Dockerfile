FROM docker.io/library/debian:trixie-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user and data directory
RUN groupadd -r appuser && useradd -r -g appuser appuser && \
    mkdir -p /data && chown appuser:appuser /data

# Copy the pre-built binary
COPY void-eid-backend .
RUN chmod +x ./void-eid-backend

# Switch to non-root user
USER appuser

# Expose the API port
EXPOSE 5038

CMD ["./void-eid-backend"]
