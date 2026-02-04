import sqlite3
import uuid
import random
import os

DB_PATH = "../../data/void-eid.db"

def seed_data():
    if not os.path.exists(DB_PATH):
        print(f"Error: Database not found at {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    tribes = ["Fire", "Water"]
    users_to_generate = 100

    print(f"Generating {users_to_generate} users...")

    for i in range(users_to_generate):
        user_id = str(uuid.uuid4())
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
