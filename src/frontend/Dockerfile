# Builder stage
FROM docker.io/oven/bun:1.2.5@sha256:4da29167551ee56bf05864bfff60430aefd1b0bf29a4671f5ed567a07d568d1f AS builder
WORKDIR /app

# Copy package.json and lockfile (if exists)
COPY package.json bun.lock* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
ARG VITE_API_URL
ENV VITE_API_URL=$VITE_API_URL
RUN bun run build

# Runtime stage
FROM docker.io/library/debian:trixie-slim@sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    nginx && \
    rm -rf /var/lib/apt/lists/* && \
    rm /etc/nginx/sites-enabled/default

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create required nginx directories with proper permissions
RUN mkdir -p /var/cache/nginx /var/log/nginx /var/lib/nginx /run/nginx && \
    chown -R appuser:appuser /var/cache/nginx /var/log/nginx /var/lib/nginx /run/nginx /usr/share/nginx/html && \
    chmod -R 0755 /var/cache/nginx /var/log/nginx /var/lib/nginx /run/nginx

# Forward request and error logs to docker log collector
RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

# Copy built assets from builder stage
COPY --from=builder --chown=appuser:appuser /app/dist /usr/share/nginx/html

# Copy nginx configs — replace default main config for rootless compatibility
COPY nginx.main.conf /etc/nginx/nginx.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Add entrypoint for runtime config
COPY --chown=appuser:appuser entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER appuser

# Expose port 8080 (non-privileged port)
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/ || exit 1

CMD ["/entrypoint.sh"]
