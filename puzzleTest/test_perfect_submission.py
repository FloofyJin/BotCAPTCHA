#!/usr/bin/env python3
"""Test submission with perfect (known correct) answers via direct API calls."""

import time
import requests
from config import Config

BASE_URL = Config.BOTCAPTCHA_URL

print("=== Testing Perfect Submission ===")
print("Fetches a challenge, counts word frequencies exactly, and submits.\n")

session = requests.Session()
start_time = time.time()

# Fetch challenge
resp = session.get(f"{BASE_URL}/api/challenge", timeout=10)
resp.raise_for_status()
challenge = resp.json()

print(f"Challenge {challenge['challenge_id']}")
print(f"  Duration limit: {challenge['duration_ms']}ms")

# Count word frequencies directly from the text
word_freq = {}
for word in challenge["text_content"].split():
    word_freq[word] = word_freq.get(word, 0) + 1

total_words = sum(word_freq.values())
unique_words = len(word_freq)
print(f"  Total words: {total_words}, Unique words: {unique_words}")
print(f"\nSubmitting PERFECT answer (exact word counts)...\n")

# Submit
resp = session.post(
    f"{BASE_URL}/api/submit",
    json={"challenge_id": challenge["challenge_id"], "answer": word_freq},
    timeout=10,
)
resp.raise_for_status()
result = resp.json()

elapsed_ms = (time.time() - start_time) * 1000

print("=== RESULT ===")
print(f"Elapsed time: {elapsed_ms:.0f}ms / {challenge['duration_ms']}ms")
print(f"Success:  {result.get('success', False)}")
print(f"Message:  {result.get('message', 'N/A')}")

if result.get("score") is not None:
    score_pct = result["score"] * 100
    print(f"Score:    {score_pct:.2f}%")

    if score_pct >= 99.9:
        print("\n✓ PERFECT! Submission and validation are working correctly.")
    elif score_pct >= 80:
        print(f"\n✓ Passed with {score_pct:.1f}% (above 80% threshold)")
    else:
        print(f"\n✗ Only {score_pct:.1f}% — something is wrong with scoring.")
        print("  Check that the server is running the latest build.")
