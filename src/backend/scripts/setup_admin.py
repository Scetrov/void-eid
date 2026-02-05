import sqlite3
import uuid
import sys
import os

def get_db_path():
    # Find project root by looking for .env or data directory
    current_dir = os.path.dirname(os.path.abspath(__file__))
    root = current_dir
    while root != os.path.dirname(root):
        if os.path.exists(os.path.join(root, '.env')):
            break
        root = os.path.dirname(root)
    else:
        return "../../data/void-eid.db"

    env_path = os.path.join(root, '.env')
    db_relative_url = "../../data/void-eid.db"

    try:
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line.startswith('DATABASE_URL='):
                    db_relative_url = line.split('=', 1)[1].strip()
                    break
    except Exception:
        pass

    src_backend_path = os.path.join(root, 'src', 'backend')
    return os.path.normpath(os.path.join(src_backend_path, db_relative_url))

DB_PATH = get_db_path()

def setup_admin():
    if not os.path.exists(DB_PATH):
        print(f"Error: Database not found at {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # Use the Discord ID from the provided JSON for a precise match
    target_discord_id = "26555034" + "7433410560"
    username = "scetrov"

    cursor.execute("SELECT id FROM users WHERE discord_id = ? OR username = ?", (target_discord_id, username))
    row = cursor.fetchone()

    if not row:
        print(f"User '{username}' or Discord ID '{target_discord_id}' not found. Creating...")
        user_id = str(uuid.uuid4())
        cursor.execute("""
            INSERT INTO users (id, discord_id, username, discriminator, is_admin)
            VALUES (?, ?, ?, ?, ?)
        """, (user_id, target_discord_id, username, "0", 1))
    else:
        user_id = row[0]
        cursor.execute("UPDATE users SET is_admin = 1 WHERE id = ?", (user_id,))
        print(f"Updated user '{username}' (ID: {user_id}) to Admin.")

    # Assign all existing wallets for this user to the Fire tribe
    cursor.execute("SELECT id, address FROM wallets WHERE user_id = ?", (user_id,))
    wallets = cursor.fetchall()

    if not wallets:
        print("No wallets found for user, creating a test wallet...")
        wallet_id = str(uuid.uuid4())
        address = f"0x{uuid.uuid4().hex}"
        cursor.execute("""
            INSERT INTO wallets (id, user_id, address, verified_at)
            VALUES (?, ?, ?, datetime('now'))
        """, (wallet_id, user_id, address))
        wallets = [(wallet_id, address)]

    # Assign wallets to different tribes (cycling through available tribes)
    tribes = ["Fire", "Water", "Earth", "Wind"]
    for i, (wallet_id, address) in enumerate(wallets):
        tribe = tribes[i % len(tribes)]
        print(f"Assigning wallet {address[:10]}... to {tribe} tribe (as admin).")
        cursor.execute("""
            INSERT OR IGNORE INTO user_tribes (user_id, tribe, wallet_id, is_admin)
            VALUES (?, ?, ?, 1)
        """, (user_id, tribe, wallet_id))

    conn.commit()
    conn.close()
    assigned_tribes = [tribes[i % len(tribes)] for i in range(len(wallets))]
    print(f"Successfully set {username} as Admin in tribes: {', '.join(set(assigned_tribes))}.")

if __name__ == "__main__":
    setup_admin()
