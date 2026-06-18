# Builder stage
FROM docker.io/oven/bun:1.3.14@sha256:e10577f0db68676a7024391c6e5cb4b879ebd17188ab750cf10024a6d700e5c4 AS builder
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
FROM docker.io/library/debian:trixie-slim@sha256:1275c5673a6135ff07b289ddafe4e2270dceb08eda14c0c69bb1b93ee25a9416

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
