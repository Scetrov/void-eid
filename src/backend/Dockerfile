FROM docker.io/library/debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd

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
