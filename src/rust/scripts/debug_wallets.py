import sqlite3
import json

def debug_json():
    conn = sqlite3.connect("../../data/void-eid.db")
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
