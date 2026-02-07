FROM docker.io/library/alpine:latest

# Install runtime dependencies
# mumble-server is in the community repository
RUN apk add --no-cache \
    mumble-server \
    python3 \
    py3-pip \
    curl \
    libbz2 \
    openssl \
    libstdc++

WORKDIR /app

# Install build dependencies, build zeroc-ice, then remove build deps
RUN apk add --no-cache --virtual .build-deps \
    build-base \
    python3-dev \
    bzip2-dev \
    openssl-dev \
    && rm -f /usr/lib/python3.*/EXTERNALLY-MANAGED 2>/dev/null || true \
    && pip3 install --no-cache-dir requests zeroc-ice \
    && apk del .build-deps

# Copy files
COPY murmur.ini /etc/murmur.ini
COPY authenticator.py /app/authenticator.py
COPY Murmur.ice /app/Murmur.ice
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

# Create mumble-server user/group if not exists
RUN addgroup -S mumble-server 2>/dev/null || true && adduser -S mumble-server -G mumble-server 2>/dev/null || true

# Create data dir (mumble-server user is created by the apk package)
RUN mkdir -p /data && chown -R mumble-server:mumble-server /data

EXPOSE 64738/tcp 64738/udp 6502

CMD ["/app/start.sh"]
