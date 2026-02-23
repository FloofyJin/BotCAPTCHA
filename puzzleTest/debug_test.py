#!/usr/bin/env python3
"""Debug script to inspect challenge content and submission behaviour."""

import time
import json
import requests
from config import Config

BASE_URL = Config.BOTCAPTCHA_URL

print("=== BotCaptcha Debug Test ===\n")

session = requests.Session()
start_time = time.time()

# ── Fetch challenge ──────────────────────────────────────────────
resp = session.get(f"{BASE_URL}/api/challenge", timeout=10)
resp.raise_for_status()
challenge = resp.json()

load_time = time.time() - start_time
print(f"✓ Challenge loaded in {load_time*1000:.0f}ms\n")

print("Challenge Info:")
print(f"  ID:             {challenge['challenge_id']}")
print(f"  Duration limit: {challenge['duration_ms']}ms")

words = challenge["text_content"].split()
word_freq = {}
for word in words:
    word_freq[word] = word_freq.get(word, 0) + 1

total_words = len(words)
unique_words = len(word_freq)
print(f"  Total words:    {total_words}")
print(f"  Unique words:   {unique_words}")
print()

# ── Show passage preview ─────────────────────────────────────────
lines = challenge["text_content"].split("\n")
print(f"Passage ({len(lines)} lines):")
for i, line in enumerate(lines[:5]):        # first 5 lines
    print(f"  {line}")
if len(lines) > 5:
    print(f"  ... ({len(lines) - 5} more lines)")
print()

# ── Show word frequency table ────────────────────────────────────
print("Word frequencies (sorted by count desc):")
sorted_words = sorted(word_freq.items(), key=lambda kv: kv[1], reverse=True)
for word, count in sorted_words[:15]:
    print(f"  {word:12s} {count}")
if len(sorted_words) > 15:
    print(f"  ... ({len(sorted_words) - 15} more words)")
print()

# ── Timing checkpoint ────────────────────────────────────────────
elapsed_ms = (time.time() - start_time) * 1000
max_time_ms = challenge["duration_ms"]

print("=== TIMING (before submit) ===")
print(f"  Elapsed:     {elapsed_ms:.0f}ms")
print(f"  Limit:       {max_time_ms}ms")
if elapsed_ms > max_time_ms:
    print(f"  WARNING: Already over the limit by {elapsed_ms - max_time_ms:.0f}ms!")
else:
    print(f"  Remaining:   {max_time_ms - elapsed_ms:.0f}ms")
print()

# ── Submit ───────────────────────────────────────────────────────
print("Submitting answer...")
submit_start = time.time()

resp = session.post(
    f"{BASE_URL}/api/submit",
    json={"challenge_id": challenge["challenge_id"], "answer": word_freq},
    timeout=10,
)
resp.raise_for_status()
result = resp.json()

submit_ms = (time.time() - submit_start) * 1000
total_elapsed_ms = (time.time() - start_time) * 1000

print(f"✓ Submission completed in {submit_ms:.0f}ms")
print(f"  Total time: {total_elapsed_ms:.0f}ms\n")

# ── Result ───────────────────────────────────────────────────────
print("=== RESULT ===")
print(f"Success: {result.get('success', False)}")
print(f"Message: {result.get('message', 'N/A')}")
if result.get("score") is not None:
    print(f"Score:   {result['score'] * 100:.2f}%")
print()

# ── Save debug output ────────────────────────────────────────────
debug_info = {
    "challenge_id": challenge["challenge_id"],
    "duration_ms": challenge["duration_ms"],
    "total_words": total_words,
    "unique_words": unique_words,
    "word_frequencies": word_freq,
    "result": result,
    "timing": {
        "load_ms": load_time * 1000,
        "submit_ms": submit_ms,
        "total_ms": total_elapsed_ms,
        "limit_ms": max_time_ms,
        "exceeded": total_elapsed_ms > max_time_ms,
    },
}

with open("debug_output.json", "w") as f:
    json.dump(debug_info, f, indent=2)

print("✓ Debug info saved to debug_output.json")
