import sqlite3
import uuid
import sys
import os

DB_PATH = "../../data/void-eid.db"

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

    for wallet_id, address in wallets:
        for tribe in ["Fire", "Water"]:
            print(f"Assigning wallet {address[:10]}... to {tribe} tribe.")
            cursor.execute("""
                INSERT OR IGNORE INTO user_tribes (user_id, tribe, wallet_id)
                VALUES (?, ?, ?)
            """, (user_id, tribe, wallet_id))

    conn.commit()
    conn.close()
    print(f"Successfully set {username} as Admin in Fire and Water tribes.")

if __name__ == "__main__":
    setup_admin()
