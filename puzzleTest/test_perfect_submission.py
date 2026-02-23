#!/usr/bin/env python3
"""Test perfect submission — uses an LLM to parse the grid prose, then submits both answers."""

import json
import re
import time
import requests
from openai import OpenAI
from config import Config

Config.validate()

BASE_URL = Config.BOTCAPTCHA_URL

print("=== Testing Perfect Submission (Both Puzzles) ===\n")

session = requests.Session()
start_time = time.time()

# ── Fetch challenge ───────────────────────────────────────────────────────────
resp = session.get(f"{BASE_URL}/api/challenge", timeout=10)
resp.raise_for_status()
challenge = resp.json()

print(f"Challenge {challenge['challenge_id']}")
print(f"  Duration limit: {challenge['duration_ms']}ms")

# ── Puzzle 1: word frequencies ────────────────────────────────────────────────
word_freq = {}
for word in challenge["text_content"].split():
    word_freq[word] = word_freq.get(word, 0) + 1
print(f"  Word puzzle:  {sum(word_freq.values())} words, {len(word_freq)} unique")

# ── Puzzle 2: LLM parses the prose description ────────────────────────────────
print(f"\n  Grid prose: \"{challenge['grid_coords_text']}\"\n")
print(f"  Calling {Config.OPENAI_MODEL} to extract coordinates...")

client = OpenAI(api_key=Config.OPENAI_API_KEY)
grid_size = challenge["grid_size"]
prompt = (
    f"Extract all grid cell coordinates from this description.\n"
    f"The grid uses 0-based indexing: columns 0–{grid_size - 1} (left = 0) "
    f"and rows 0–{grid_size - 1} (top = 0).\n\n"
    f"Description:\n{challenge['grid_coords_text']}\n\n"
    f'Return ONLY a JSON array, e.g. [{{"col": 3, "row": 0}}, {{"col": 5, "row": 2}}]. '
    f"No explanation."
)
response = client.chat.completions.create(
    model=Config.OPENAI_MODEL,
    max_tokens=512,
    temperature=0,
    messages=[{"role": "user", "content": prompt}],
)
content = response.choices[0].message.content.strip()
match = re.search(r"\[[\s\S]*\]", content)
raw_coords = json.loads(match.group(0) if match else content)

sorted_coords = sorted(raw_coords, key=lambda c: (c["row"], c["col"]))
print(f"  Extracted: {[(c['col'], c['row']) for c in raw_coords]}")
print(f"  Sorted:    {[(c['col'], c['row']) for c in sorted_coords]}")

# ── Submit ────────────────────────────────────────────────────────────────────
print("\nSubmitting both answers...")
resp = session.post(
    f"{BASE_URL}/api/submit",
    json={
        "challenge_id": challenge["challenge_id"],
        "answer": word_freq,
        "grid_answer": sorted_coords,
    },
    timeout=10,
)
resp.raise_for_status()
result = resp.json()

elapsed_ms = (time.time() - start_time) * 1000
print(f"\n=== RESULT ({elapsed_ms:.0f}ms) ===")
print(f"Success:    {result.get('success', False)}")
print(f"Message:    {result.get('message', 'N/A')}")
print(f"Word score: {result.get('word_score', 'N/A')}")
print(f"Grid score: {result.get('grid_score', 'N/A')}")

if result.get("success"):
    print("\n✓ PASSED — both puzzles validated correctly.")
else:
    print("\n✗ FAILED — check grid score; LLM may have misread the coordinates.")
