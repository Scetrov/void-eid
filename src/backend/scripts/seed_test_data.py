import sqlite3
import uuid
import random
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

def seed_data():
    if not os.path.exists(DB_PATH):
        print(f"Error: Database not found at {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    tribes = ["Fire", "Water", "Earth", "Wind"]
    users_to_generate = 100

    print(f"Generating {users_to_generate} users...")

    for i in range(users_to_generate):
        # Generate random 64-bit integer for user_id (SQLite INTEGER is i64)
        user_id = random.randint(1, 2**62 - 1)  # Keep positive range for SQLite
        discord_id = str(100000000000000000 + i)
        username = f"User_{i}"
        discriminator = f"{random.randint(1000, 9999)}"

        # Insert user if not exists
        cursor.execute("""
            INSERT OR IGNORE INTO users (id, discord_id, username, discriminator, is_admin)
            VALUES (?, ?, ?, ?, ?)
        """, (user_id, discord_id, username, discriminator, 0))

        # Get the actual user_id (in case it existed)
        cursor.execute("SELECT id FROM users WHERE discord_id = ?", (discord_id,))
        actual_user_id = cursor.fetchone()[0]

        # Generate 1-2 wallets for each user
        num_wallets = random.randint(1, 2)
        for j in range(num_wallets):
            wallet_id = str(uuid.uuid4())
            address = f"0x{uuid.uuid4().hex}"

            cursor.execute("""
                INSERT OR IGNORE INTO wallets (id, user_id, address, verified_at)
                VALUES (?, ?, ?, datetime('now'))
            """, (wallet_id, actual_user_id, address))

            # Randomly assign to a tribe
            tribe = random.choice(tribes)
            cursor.execute("""
                INSERT OR IGNORE INTO user_tribes (user_id, tribe, wallet_id)
                VALUES (?, ?, ?)
            """, (actual_user_id, tribe, wallet_id))

    conn.commit()
    conn.close()
    print("Seeding complete.")

if __name__ == "__main__":
    seed_data()
