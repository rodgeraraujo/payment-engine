#!/usr/bin/env python3
"""Generate a large random transactions CSV for scale/memory checks.

usage: scripts/gen_large.py ROWS > big.csv
"""
import random
import sys

rows = int(sys.argv[1]) if len(sys.argv) > 1 else 1_000_000
random.seed(42)
print("type,client,tx,amount")
deposits = []
for tx in range(1, rows + 1):
    client = random.randint(1, 65535)
    r = random.random()
    if r < 0.6 or not deposits:
        print(f"deposit,{client},{tx},{random.randint(1, 10**6) / 10**4:.4f}")
        if len(deposits) < 1000:
            deposits.append((client, tx))
    elif r < 0.8:
        print(f"withdrawal,{client},{tx},{random.randint(1, 10**5) / 10**4:.4f}")
    else:
        c, t = random.choice(deposits)
        print(f"{random.choice(['dispute', 'resolve', 'chargeback'])},{c},{t},")
