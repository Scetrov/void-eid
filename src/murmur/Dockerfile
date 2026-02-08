FROM docker.io/library/debian:trixie-slim

# Install runtime dependencies
# python3-zeroc-ice is available in sid
RUN apt-get update && apt-get install -y \
    mumble-server \
    python3 \
    python3-zeroc-ice \
    python3-requests \
    curl \
    bzip2 \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# No need to pip install zeroc-ice or requests as they are installed via apt

# Copy files
COPY murmur.ini /etc/murmur.ini
COPY authenticator.py /app/authenticator.py
COPY Murmur.ice /app/Murmur.ice
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

# Create mumble-server user/group if not exists (invoke-rc.d might complain if not present, but for docker build it's fine)
# The package usually creates the user 'mumble-server'
RUN getent group mumble-server || groupadd -r mumble-server && \
    getent passwd mumble-server || useradd -r -g mumble-server mumble-server

# Create data dir
RUN mkdir -p /data && chown -R mumble-server:mumble-server /data

EXPOSE 64738/tcp 64738/udp 6502

CMD ["/app/start.sh"]
