#!/bin/sh

set -e

# Generate the runtime config file from environment variables
# We only care about VITE_ prefixed variables or specific ones like VITE_API_URL
echo "window.ENV = {" > /usr/share/nginx/html/env-config.js
echo "  VITE_API_URL: \"${VITE_API_URL}\"," >> /usr/share/nginx/html/env-config.js
echo "  VITE_BLOCK_EXPLORER_URL: \"${VITE_BLOCK_EXPLORER_URL}\"," >> /usr/share/nginx/html/env-config.js
echo "  VITE_MUMBLE_SERVER_URL: \"${VITE_MUMBLE_SERVER_URL}\"," >> /usr/share/nginx/html/env-config.js
echo "  VITE_SUI_NETWORK: \"${VITE_SUI_NETWORK}\"" >> /usr/share/nginx/html/env-config.js
echo "};" >> /usr/share/nginx/html/env-config.js

echo "Generated runtime config:"
cat /usr/share/nginx/html/env-config.js

# Ensure temp directories exist and are writable
mkdir -p /tmp/nginx
chmod 0755 /tmp/nginx

# Start nginx
exec nginx -g "daemon off;"
