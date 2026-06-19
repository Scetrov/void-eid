FROM docker.io/library/debian:trixie-slim@sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
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

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:5038/ping || exit 1

ENTRYPOINT ["/usr/local/bin/entrypoint"]
CMD ["./void-eid-backend"]
