import sqlite3
import json
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
        # Fallback if root not found (though unlikely in this repo)
        return "../../data/void-eid.db"

    # Load .env manually to avoid dependencies
    env_path = os.path.join(root, '.env')
    db_relative_url = "../../data/void-eid.db" # Default fallback
    
    try:
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line.startswith('DATABASE_URL='):
                    db_relative_url = line.split('=', 1)[1].strip()
                    break
    except Exception:
        pass

    # The DATABASE_URL in .env is relative to src/backend (../../data/...)
    # We need to reconstruct the absolute path.
    # root/src/backend + db_relative_url
    
    # However, a cleaner way might be to resolve it from the root if we know the .env intent.
    # Given the .env has "../../data/void-eid.db", it assumes a specific CWD.
    # Let's resolve it relative to the 'src/backend' location which is where .env is effectively targeting?
    # Actually, the .env is in the ROOT. The DATABASE_URL is `../../data/void-eid.db`.
    # Wait, in the view_file of .env (Step 7), the .env content was:
    # DATABASE_URL=../../data/void-eid.db
    # If .env is at ROOT, then `../../data` is actually outside the repo??
    # Let's re-read the .env location. `view_file /home/scetrov/source/void-eid/.env`.
    # It is at the root.
    # If the database is at `data/void-eid.db` (as seen in list_dir Step 16), then `../../data/void-eid.db` from ROOT is definitely wrong if taken literally from root.
    # It implies the variable is intended for use by something deep in `src/backend`.
    # So we should treat it as relative to `src/backend`.
    
    src_backend_path = os.path.join(root, 'src', 'backend')
    return os.path.normpath(os.path.join(src_backend_path, db_relative_url))

def debug_json():
    db_path = get_db_path()
    if not os.path.exists(db_path):
        print(f"Error: Database not found at {db_path}")
        return

    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    # Simulate the query in roster.rs
    cursor.execute("SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id LIMIT 2")
    rows = cursor.fetchall()

    results = []
    for row in rows:
        d = dict(row)
        # Simulate serde(rename_all = "camelCase")
        camel_d = {
            "id": d["id"],
            "userId": d["user_id"],
            "address": d["address"],
            "verifiedAt": d["verified_at"],
            "tribe": d["tribe"]
        }
        results.append(camel_d)

    print(json.dumps(results, indent=2))
    conn.close()

if __name__ == "__main__":
    debug_json()
