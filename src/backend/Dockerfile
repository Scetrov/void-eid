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
    mkdir -p /data && chown appuser:appuser /data && chmod 0775 /data

# Copy the pre-built binary
COPY void-eid-backend .
RUN chmod +x ./void-eid-backend

# Create a wrapper script that ensures /data permissions are correct at runtime
RUN echo '#!/bin/sh' > /usr/local/bin/entrypoint && \
    echo 'chmod 0775 /data 2>/dev/null || true' >> /usr/local/bin/entrypoint && \
    echo 'exec "$@"' >> /usr/local/bin/entrypoint && \
    chmod +x /usr/local/bin/entrypoint

# Switch to non-root user
USER appuser

# Expose the API port
EXPOSE 5038

ENTRYPOINT ["/usr/local/bin/entrypoint"]
CMD ["./void-eid-backend"]
