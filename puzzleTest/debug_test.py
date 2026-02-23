#!/usr/bin/env python3
"""Debug script to inspect both puzzle challenges and submission behaviour."""

import json
import re
import time
import requests
from openai import OpenAI
from config import Config

Config.validate()

BASE_URL = Config.BOTCAPTCHA_URL

print("=== BotCaptcha Debug Test ===\n")

session = requests.Session()
start_time = time.time()

# ── Fetch challenge ───────────────────────────────────────────────────────────
resp = session.get(f"{BASE_URL}/api/challenge", timeout=10)
resp.raise_for_status()
challenge = resp.json()

load_ms = (time.time() - start_time) * 1000
print(f"✓ Challenge loaded in {load_ms:.0f}ms\n")

print("Challenge Info:")
print(f"  ID:             {challenge['challenge_id']}")
print(f"  Duration limit: {challenge['duration_ms']}ms")

# ── Word puzzle ───────────────────────────────────────────────────────────────
words = challenge["text_content"].split()
word_freq = {}
for word in words:
    word_freq[word] = word_freq.get(word, 0) + 1

print(f"\nWord Puzzle:")
print(f"  Total words:  {len(words)}")
print(f"  Unique words: {len(word_freq)}")
print(f"  Preview: {' '.join(words[:15])} ...")
print(f"\n  Top 10 by frequency:")
for word, count in sorted(word_freq.items(), key=lambda kv: -kv[1])[:10]:
    print(f"    {word:12s} {count}")

# ── Grid puzzle ───────────────────────────────────────────────────────────────
print(f"\nGrid Puzzle ({challenge['grid_size']}x{challenge['grid_size']}):")
print(f"  Prose description:\n  \"{challenge['grid_coords_text']}\"\n")

print(f"  Calling {Config.OPENAI_MODEL} to parse coordinates...")
llm_start = time.time()

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
llm_ms = (time.time() - llm_start) * 1000
content = response.choices[0].message.content.strip()
match = re.search(r"\[[\s\S]*\]", content)
raw_coords = json.loads(match.group(0) if match else content)

print(f"  ✓ LLM responded in {llm_ms:.0f}ms")
print(f"  Raw output: {content}")
print(f"  Extracted {len(raw_coords)} coords: {[(c['col'], c['row']) for c in raw_coords]}")

sorted_coords = sorted(raw_coords, key=lambda c: (c["row"], c["col"]))
print(f"  Sorted (reading order): {[(c['col'], c['row']) for c in sorted_coords]}")

# ── Timing before submit ──────────────────────────────────────────────────────
elapsed_ms = (time.time() - start_time) * 1000
limit_ms = challenge["duration_ms"]
print(f"\nTiming before submit: {elapsed_ms:.0f}ms elapsed / {limit_ms}ms limit")
if elapsed_ms > limit_ms:
    print(f"  WARNING: exceeded limit by {elapsed_ms - limit_ms:.0f}ms!")

# ── Submit ────────────────────────────────────────────────────────────────────
print("\nSubmitting both answers...")
submit_start = time.time()

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

submit_ms = (time.time() - submit_start) * 1000
total_ms = (time.time() - start_time) * 1000
print(f"✓ Done in {submit_ms:.0f}ms  (total: {total_ms:.0f}ms)\n")

print("=== RESULT ===")
print(f"Success:    {result.get('success', False)}")
print(f"Message:    {result.get('message', 'N/A')}")
print(f"Word score: {result.get('word_score', 'N/A')}")
print(f"Grid score: {result.get('grid_score', 'N/A')}")
print(f"Combined:   {result.get('score', 'N/A')}")

# ── Save debug output ─────────────────────────────────────────────────────────
debug_info = {
    "challenge_id": challenge["challenge_id"],
    "duration_ms": challenge["duration_ms"],
    "word_puzzle": {"total_words": len(words), "unique_words": len(word_freq), "frequencies": word_freq},
    "grid_puzzle": {
        "grid_size": challenge["grid_size"],
        "prose": challenge["grid_coords_text"],
        "llm_raw_output": content,
        "extracted_coords": raw_coords,
        "sorted_coords": sorted_coords,
    },
    "result": result,
    "timing": {"load_ms": load_ms, "llm_ms": llm_ms, "submit_ms": submit_ms, "total_ms": total_ms, "limit_ms": limit_ms},
}

with open("debug_output.json", "w") as f:
    json.dump(debug_info, f, indent=2)
print("\n✓ Debug info saved to debug_output.json")
