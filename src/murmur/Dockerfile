FROM docker.io/library/debian:trixie-slim

# Install runtime dependencies and build dependencies in one go to keep layer small if cleanup worked (but we do separate install for clarity and caching if needed, though combined is better for size)
# We need to install runtime deps, then build deps, compile/install pip packages, then remove build deps.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    mumble-server \
    python3 \
    python3-pip \
    python3-zeroc-ice \
    zeroc-ice-slice \
    curl \
    libbz2-1.0 \
    openssl \
    libstdc++6 && \
    pip3 install --no-cache-dir --break-system-packages requests && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy files
COPY murmur.ini /etc/murmur.ini
COPY authenticator.py /app/authenticator.py
ENV ICE_SLICE=/usr/share/mumble-server/MumbleServer.ice
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

# Create mumble-server user/group if not exists (Debian package usually creates it, but ensuring it)
RUN id -u mumble-server >/dev/null 2>&1 || useradd -r -m -d /var/lib/mumble-server -s /usr/sbin/nologin mumble-server

# Create data dir (mumble-server user is created by the apk package)
RUN mkdir -p /data && chown -R mumble-server:mumble-server /data

EXPOSE 64738/tcp 64738/udp 6502

CMD ["/app/start.sh"]
