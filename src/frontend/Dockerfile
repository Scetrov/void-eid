# Builder stage
FROM docker.io/oven/bun:latest AS builder
WORKDIR /app

# Copy package.json and lockfile (if exists)
COPY package.json bun.lock* ./

# Install dependencies
RUN bun install

# Copy source code
COPY . .

# Build the application
ARG VITE_API_URL
ENV VITE_API_URL=$VITE_API_URL
RUN bun run build

# Runtime stage
FROM docker.io/library/debian:trixie-slim

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    nginx && \
    rm -rf /var/lib/apt/lists/* && \
    rm /etc/nginx/sites-enabled/default

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create required nginx directories with proper permissions
RUN mkdir -p /var/cache/nginx /var/log/nginx /var/lib/nginx && \
    chown -R appuser:appuser /var/cache/nginx /var/log/nginx /var/lib/nginx /usr/share/nginx/html && \
    chmod -R 0755 /var/cache/nginx /var/log/nginx /var/lib/nginx

# Forward request and error logs to docker log collector
RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

# Copy built assets from builder stage
COPY --from=builder --chown=appuser:appuser /app/dist /usr/share/nginx/html

# Copy nginx config - replace the default site
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Add entrypoint for runtime config
COPY --chown=appuser:appuser entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER appuser

# Expose port 8080 (non-privileged port)
EXPOSE 8080

CMD ["/entrypoint.sh"]
