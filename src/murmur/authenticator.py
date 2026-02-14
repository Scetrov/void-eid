import sys
import os
import Ice
import requests
import time
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger("MurmurAuthenticator")

# Load Murmur.ice
try:
    # Initialize include_path
    include_path = []

    # Attempt to locate slice dir
    slice_paths = [
        '/usr/share/slice',
        '/usr/share/ice/slice',
        '/usr/local/share/ice/slice',
        '/usr/share/mumble-server', # Common for Debian packages
        '/usr/local/lib/python3.11/dist-packages/slice',
        # Try to find relative to Ice module
        os.path.join(os.path.dirname(Ice.__file__), 'slice'),
        # Common pip install location
        os.path.join(sys.prefix, 'share', 'ice', 'slice'),
        # Check current dir
        os.getcwd()
    ]

    # Explicitly check ICE_SLICE env var first if it exists
    ice_slice_env = os.environ.get("ICE_SLICE")
    if ice_slice_env and os.path.exists(ice_slice_env):
        slice_file = ice_slice_env
        include_path.append('-I' + os.path.dirname(ice_slice_env))
    else:
        for p in slice_paths:
            if os.path.exists(os.path.join(p, 'Murmur.ice')):
                slice_file = os.path.join(p, 'Murmur.ice')
                include_path.append('-I' + p)
                break

    # Also find system slice dir for Ice/SliceChecksumDict.ice
    for p in slice_paths:
         if os.path.exists(os.path.join(p, 'Ice', 'SliceChecksumDict.ice')):
             include_path.append('-I' + p)
             break

    if not slice_file:
         # Fallback to local default if not found in system
         if os.path.exists('/app/Murmur.ice'):
             slice_file = '/app/Murmur.ice'
             include_path.append('-I.')

    if not slice_file:
        logger.error("Could not find Murmur.ice in common locations.")
        sys.exit(1)

    logger.info(f"Loading Slice: {slice_file}")
    Ice.loadSlice('', include_path + [slice_file])
    import Murmur
except ImportError:
    logger.error("Failed to load generic Ice modules or slice.")
    sys.exit(1)

class ServerAuthenticatorI(Murmur.ServerAuthenticator):
    def __init__(self, backend_url, internal_secret):
        self.backend_url = backend_url
        self.internal_secret = internal_secret

    def authenticate(self, name, pw, certificates, certhash, certstrong, ctx, current=None):
        logger.info(f"Authenticating user: {name}")

        # Call Backend
        try:
            payload = {
                "username": name,
                "password": pw,
                "extra": {} # Can pass certificate info here if needed
            }
            headers = {
                "X-Internal-Secret": self.internal_secret,
                "Content-Type": "application/json"
            }

            response = requests.post(f"{self.backend_url}/verify", json=payload, headers=headers, timeout=5)

            if response.status_code == 200:
                data = response.json()
                user_id = data.get("user_id", -1)
                new_name = data.get("username", name)
                logger.info(f"Authentication successful for {name} (ID: {user_id})")
                return user_id, new_name, [] # ID, Name, Groups
            else:
                logger.warning(f"Authentication failed for {name}: Backend returned {response.status_code}")
                return -1, None, []

        except Exception as e:
            logger.error(f"Error calling backend: {e}")
            return -1, None, []

    def getInfo(self, id, current=None):
        return False, {}

    def name(self, id, current=None):
        return None

    def id(self, name, current=None):
        return -1

def run():
    ice_host = os.environ.get("ICE_HOST", "127.0.0.1")
    ice_port = os.environ.get("ICE_PORT", "6502")
    ice_secret = os.environ.get("ICE_SECRET")

    if not ice_secret:
        logger.error("ERROR: ICE_SECRET environment variable is not set")
        sys.exit(1)

    backend_url = os.environ.get("BACKEND_URL", "http://backend:3000/api/internal/mumble")
    internal_secret = os.environ.get("INTERNAL_SECRET")

    if not internal_secret:
        logger.error("ERROR: INTERNAL_SECRET environment variable is not set")
        sys.exit(1)

    init_data = Ice.InitializationData()
    init_data.properties = Ice.createProperties()
    init_data.properties.setProperty("Ice.ImplicitContext", "Shared")

    # Enable this if using encryption
    # init_data.properties.setProperty("Ice.Default.Protocol", "ssl")

    communicator = Ice.initialize(init_data)

    logger.info(f"Using ICE Secret from environment")
    communicator.getImplicitContext().put("secret", ice_secret)

    logger.info(f"Connecting to Murmur Ice at {ice_host}:{ice_port}")

    base_str = f"Meta:tcp -h {ice_host} -p {ice_port}"
    meta = None

    # Retry loop for initial connection
    for i in range(10):
        try:
            base = communicator.stringToProxy(base_str)
            # Use uncheckedCast because checkedCast might fail if server not ready or slice mismatch logic
            meta = Murmur.MetaPrx.uncheckedCast(base)
            # Verify connection by calling a lightweight method
            meta.ice_ping()
            logger.info("Successfully connected to Murmur Meta proxy.")
            break
        except Exception as e:
            logger.warning(f"Connection attempt {i+1}/10 failed: {e}")
            time.sleep(2)

    if not meta:
        logger.error("Could not obtain Meta proxy after multiple attempts. Is Murmur running?")
        sys.exit(1)

    logger.info("Connected to Meta. Waiting for servers...")

    adapter = communicator.createObjectAdapterWithEndpoints("Callback.Client", "tcp")
    authenticator = ServerAuthenticatorI(backend_url, internal_secret)

    adapter.activate()

    # We need to attach to existing servers or listen for new ones
    # Simplified: Attach to server 1 (default)
    # Retry attachment as server might be booting
    attached = False
    for i in range(5):
        try:
            server = meta.getServer(1)
            if server:
                logger.info("Found Server 1. Setting authenticator...")
                server.setAuthenticator(Murmur.ServerAuthenticatorPrx.uncheckedCast(adapter.addWithUUID(authenticator)))
                logger.info("Authenticator attached.")
                attached = True
                break
        except Murmur.InvalidSecretException:
            logger.error("Invalid ICE secret!")
            break
        except Exception as e:
             logger.warning(f"Failed to attach authenticator (attempt {i+1}/5): {e}")
             time.sleep(2)

    if not attached:
        logger.error("Failed to attach authenticator to Server 1.")


    try:
        communicator.waitForShutdown()
    except KeyboardInterrupt:
        communicator.destroy()

if __name__ == "__main__":
    run()
