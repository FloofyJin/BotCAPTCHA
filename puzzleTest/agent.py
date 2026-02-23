"""Main test agent for BotCaptcha text frequency + grid ordering challenge."""

import json
import re
import time
import requests
from openai import OpenAI
from config import Config


class TestAgent:
    """Agent that solves BotCaptcha word-frequency challenges via direct API calls."""

    def __init__(self, headless=False, save_screenshots=False):
        # headless / save_screenshots kept for call-site compatibility but unused
        self.session = requests.Session()

        # Statistics
        self.attempts = 0
        self.successes = 0
        self.failures = 0
        self.errors = 0

    def _get_challenge(self):
        """Fetch a new challenge from the server."""
        resp = self.session.get(f"{Config.BOTCAPTCHA_URL}/api/challenge", timeout=10)
        resp.raise_for_status()
        return resp.json()

    def _count_words(self, text_content):
        """
        Count word frequencies in the challenge text.
        Words are already lowercase and space-separated — just split and count.
        """
        freq = {}
        for word in text_content.split():
            freq[word] = freq.get(word, 0) + 1
        return freq

    def _parse_grid_coords_with_llm(self, grid_coords_text: str, grid_size: int) -> list:
        """
        Use an LLM to extract grid coordinates from the natural-language paragraph.
        The prose uses varied phrasings so a simple regex can't reliably parse it.
        """
        client = OpenAI(api_key=Config.OPENAI_API_KEY)

        prompt = (
            f"Extract all grid cell coordinates from this description.\n"
            f"The grid uses 0-based indexing: columns 0–{grid_size - 1} (left = 0) "
            f"and rows 0–{grid_size - 1} (top = 0).\n\n"
            f"Description:\n{grid_coords_text}\n\n"
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
        return json.loads(match.group(0) if match else content)

    def _sort_grid_coords(self, coords: list) -> list:
        """Sort extracted coordinates into reading order (row asc, then col asc)."""
        return sorted(coords, key=lambda c: (c["row"], c["col"]))

    def _submit_answer(self, challenge_id, word_frequencies, sorted_coords):
        """Submit both puzzle answers to the server."""
        resp = self.session.post(
            f"{Config.BOTCAPTCHA_URL}/api/submit",
            json={
                "challenge_id": challenge_id,
                "answer": word_frequencies,
                "grid_answer": sorted_coords,
            },
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

    def run_single_test(self, test_number=None):
        """
        Run a single challenge: fetch → count → submit.

        Returns:
            dict: Test result
        """
        result = {
            "test_number": test_number,
            "success": False,
            "score": None,
            "message": "",
            "challenge_id": None,
            "num_words": None,
            "num_unique": None,
            "elapsed_ms": None,
            "error": None,
        }

        start_time = time.time()

        try:
            # Fetch challenge
            challenge = self._get_challenge()
            result["challenge_id"] = challenge["challenge_id"]

            # Puzzle 1: count word frequencies
            word_freq = self._count_words(challenge["text_content"])
            result["num_words"] = sum(word_freq.values())
            result["num_unique"] = len(word_freq)

            # Puzzle 2: parse prose description with LLM, then sort into reading order
            raw_coords = self._parse_grid_coords_with_llm(
                challenge["grid_coords_text"], challenge["grid_size"]
            )
            sorted_coords = self._sort_grid_coords(raw_coords)

            if Config.VERBOSE:
                print(f"\nChallenge {challenge['challenge_id']}:")
                print(f"  Words: {result['num_words']} total, {result['num_unique']} unique")
                print(f"  Grid text: \"{challenge['grid_coords_text'][:80]}...\"")
                print(f"  {Config.OPENAI_MODEL} extracted {len(raw_coords)} coords → sorted: {[(c['col'], c['row']) for c in sorted_coords]}")
                print(f"  Duration limit: {challenge['duration_ms']}ms")

            # Submit both answers
            submission_result = self._submit_answer(
                challenge["challenge_id"], word_freq, sorted_coords
            )

            result["success"] = submission_result.get("success", False)
            result["score"] = submission_result.get("score")
            result["message"] = submission_result.get("message", "")

            elapsed_ms = (time.time() - start_time) * 1000
            result["elapsed_ms"] = elapsed_ms

            # Update statistics
            self.attempts += 1
            if result["success"]:
                self.successes += 1
            else:
                self.failures += 1

            status = "✓ PASSED" if result["success"] else "✗ FAILED"
            score_str = (
                f"{result['score'] * 100:.1f}%" if result["score"] is not None else "N/A"
            )
            print(f"{status} - Score: {score_str} - {result['message']} ({elapsed_ms:.0f}ms)")

        except Exception as e:
            result["error"] = str(e)
            result["message"] = f"Error: {e}"
            self.errors += 1
            print(f"✗ ERROR: {e}")

        return result

    def run_multiple_tests(self, num_tests):
        """
        Run multiple challenge tests.

        Returns:
            list: List of test result dicts
        """
        print(f"\n{'='*60}")
        print(f"Running {num_tests} tests...")
        print(f"{'='*60}\n")

        results = []
        for i in range(1, num_tests + 1):
            print(f"\n--- Test {i}/{num_tests} ---")
            result = self.run_single_test(test_number=i)
            results.append(result)

        return results

    def print_summary(self):
        """Print test summary statistics."""
        total = self.attempts
        if total == 0:
            print("\nNo tests run.")
            return

        success_rate = (self.successes / total) * 100

        print(f"\n{'='*60}")
        print("TEST SUMMARY")
        print(f"{'='*60}")
        print(f"Total Attempts:  {total}")
        print(f"Successes:       {self.successes} ({success_rate:.1f}%)")
        print(f"Failures:        {self.failures}")
        print(f"Errors:          {self.errors}")
        print(f"{'='*60}\n")

        if success_rate >= 80:
            print("AI successfully passes the BotCaptcha! (>=80% success rate)")
        elif success_rate >= 50:
            print("AI has moderate success, but not consistent enough")
        else:
            print("BotCaptcha successfully filters AI (AI success rate < 50%)")
