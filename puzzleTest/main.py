#!/usr/bin/env python3
"""Main entry point for BotCaptcha test agent."""

import argparse
import json
from pathlib import Path
from datetime import datetime
from config import Config
from agent import TestAgent


def save_results(results, filename="results.json"):
    """Save test results to JSON file."""
    results_dir = Path("results")
    results_dir.mkdir(exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filepath = results_dir / f"{timestamp}_{filename}"

    with open(filepath, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to: {filepath}")
    return filepath


def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="BotCaptcha Test Agent - word frequency challenge")
    parser.add_argument(
        "-n",
        "--num-tests",
        type=int,
        default=Config.NUM_TESTS,
        help=f"Number of tests to run (default: {Config.NUM_TESTS})",
    )
    parser.add_argument(
        "--headless", action="store_true", help="Run browser in headless mode"
    )
    parser.add_argument(
        "--no-screenshots",
        action="store_true",
        help="Don't save screenshots (saves disk space)",
    )
    parser.add_argument(
        "--url", type=str, help="BotCaptcha server URL (overrides .env)"
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Verbose output"
    )

    args = parser.parse_args()

    # Override config if command-line args provided
    if args.url:
        Config.BOTCAPTCHA_URL = args.url
    if args.verbose:
        Config.VERBOSE = True

    # Validate configuration
    try:
        Config.validate()
    except ValueError as e:
        print(f"Configuration Error: {e}")
        return 1

    # Print configuration
    Config.print_config()

    # Create agent
    agent = TestAgent(
        headless=args.headless, save_screenshots=not args.no_screenshots
    )

    try:
        # Run tests
        results = agent.run_multiple_tests(args.num_tests)

        # Print summary
        agent.print_summary()

        # Save results
        test_summary = {
            "timestamp": datetime.now().isoformat(),
            "config": {
                "url": Config.BOTCAPTCHA_URL,
                "num_tests": args.num_tests,
            },
            "statistics": {
                "total_attempts": agent.attempts,
                "successes": agent.successes,
                "failures": agent.failures,
                "errors": agent.errors,
                "success_rate": (agent.successes / agent.attempts * 100)
                if agent.attempts > 0
                else 0,
            },
            "results": results,
        }

        save_results(test_summary)

        return 0

    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        agent.print_summary()
        return 130
    except Exception as e:
        print(f"\nFatal error: {e}")
        import traceback

        traceback.print_exc()
        return 1


if __name__ == "__main__":
    exit(main())
