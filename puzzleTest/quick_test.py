#!/usr/bin/env python3
"""Quick single-test script for debugging."""

from config import Config
from agent import TestAgent

Config.validate()

print("Running single test...")
print(f"URL: {Config.BOTCAPTCHA_URL}")
print()

agent = TestAgent()
result = agent.run_single_test(test_number=1)

print("\nResult:")
print(f"  Success:      {result['success']}")
print(f"  Score:        {result['score'] * 100:.1f}%" if result['score'] is not None else "  Score:        N/A")
print(f"  Message:      {result['message']}")
print(f"  Total words:  {result['num_words']}")
print(f"  Unique words: {result['num_unique']}")
print(f"  Elapsed:      {result['elapsed_ms']:.0f}ms" if result['elapsed_ms'] else "  Elapsed:      N/A")
print(f"  Challenge ID: {result['challenge_id']}")

if result['error']:
    print(f"  Error: {result['error']}")
