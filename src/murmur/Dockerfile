FROM docker.io/library/debian:trixie-slim@sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e

# Install runtime dependencies
# python3-zeroc-ice is available in sid
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    mumble-server \
    python3 \
    python3-zeroc-ice \
    python3-requests \
    zeroc-ice-slice \
    curl \
    bzip2 \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# No need to pip install zeroc-ice or requests as they are installed via apt

# Copy files
COPY murmur.ini /etc/murmur.ini
COPY Murmur.ice /app/Murmur.ice
COPY authenticator.py /app/authenticator.py
ENV ICE_SLICE=/app/Murmur.ice
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh && \
    chown mumble-server:mumble-server /etc/murmur.ini /app/start.sh /app/authenticator.py /app/Murmur.ice

# Create mumble-server user/group if not exists (invoke-rc.d might complain if not present, but for docker build it's fine)
# The package usually creates the user 'mumble-server'
RUN getent group mumble-server || groupadd -r mumble-server && \
    getent passwd mumble-server || useradd -r -g mumble-server mumble-server

# Create data dir
RUN mkdir -p /data && chown -R mumble-server:mumble-server /data

USER mumble-server

EXPOSE 64738/tcp 64738/udp

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD python3 -c "import socket; s=socket.create_connection(('127.0.0.1', 64738), timeout=3); s.close()" || exit 1

CMD ["/app/start.sh"]
