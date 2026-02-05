FROM docker.io/debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    mumble-server \
    python3 \
    python3-pip \
    curl \
    build-essential \
    libbz2-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install python deps
# requests is needed
RUN rm -f /usr/lib/python3.11/EXTERNALLY-MANAGED
RUN pip3 install requests zeroc-ice

# Copy files
COPY murmur.ini /etc/murmur.ini
COPY authenticator.py /app/authenticator.py
COPY Murmur.ice /app/Murmur.ice

# Create data dir
RUN mkdir -p /data && chown -R mumble-server:mumble-server /data

# Entrypoint script
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

EXPOSE 64738/tcp 64738/udp 6502

CMD ["/app/start.sh"]
