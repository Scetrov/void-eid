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
    Ice.loadSlice('', ['-I/usr/share/slice', 'Murmur.ice'])
    import Murmur
except ImportError:
    logger.error("Failed to load Murmur.ice or import Murmur module.")
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
    ice_secret = os.environ.get("ICE_SECRET", "")
    
    backend_url = os.environ.get("BACKEND_URL", "http://backend:3000/api/internal/mumble")
    internal_secret = os.environ.get("INTERNAL_SECRET", "changeme")

    init_data = Ice.InitializationData()
    init_data.properties = Ice.createProperties()
    init_data.properties.setProperty("Ice.ImplicitContext", "Shared")
    
    # Enable this if using encryption
    # init_data.properties.setProperty("Ice.Default.Protocol", "ssl")

    communicator = Ice.initialize(init_data)
    
    if ice_secret:
        communicator.getImplicitContext().put("secret", ice_secret)

    logger.info(f"Connecting to Murmur Ice at {ice_host}:{ice_port}")
    
    base = communicator.stringToProxy(f"Meta:tcp -h {ice_host} -p {ice_port}")
    meta = Murmur.MetaPrx.checkedCast(base)
    
    if not meta:
        logger.error("Invalid proxy")
        sys.exit(1)

    logger.info("Connected to Meta. Waiting for servers...")

    adapter = communicator.createObjectAdapterWithEndpoints("Callback.Client", "tcp")
    authenticator = ServerAuthenticatorI(backend_url, internal_secret)
    
    # We need to attach to existing servers or listen for new ones
    # Simplified: Attach to server 1 (default)
    try:
        server = meta.getServer(1)
        if server:
            logger.info("Found Server 1. Setting authenticator...")
            server.setAuthenticator(Murmur.ServerAuthenticatorPrx.uncheckedCast(adapter.addWithUUID(authenticator)))
            logger.info("Authenticator attached.")
    except Exception as e:
        logger.error(f"Failed to attach authenticator: {e}")

    adapter.activate()
    
    try:
        communicator.waitForShutdown()
    except KeyboardInterrupt:
        communicator.destroy()

if __name__ == "__main__":
    run()
