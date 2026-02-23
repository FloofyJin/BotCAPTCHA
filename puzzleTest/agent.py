"""Main test agent for BotCaptcha text frequency challenge."""

import time
import requests
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

    def _submit_answer(self, challenge_id, word_frequencies):
        """Submit the word frequency answer to the server."""
        resp = self.session.post(
            f"{Config.BOTCAPTCHA_URL}/api/submit",
            json={"challenge_id": challenge_id, "answer": word_frequencies},
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

            # Count word frequencies
            word_freq = self._count_words(challenge["text_content"])
            result["num_words"] = sum(word_freq.values())
            result["num_unique"] = len(word_freq)

            if Config.VERBOSE:
                print(f"\nChallenge {challenge['challenge_id']}:")
                print(f"  Words: {result['num_words']} total, {result['num_unique']} unique")
                print(f"  Duration limit: {challenge['duration_ms']}ms")

            # Submit answer
            submission_result = self._submit_answer(challenge["challenge_id"], word_freq)

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
