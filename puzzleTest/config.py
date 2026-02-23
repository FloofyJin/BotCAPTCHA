"""Configuration for BotCaptcha test agent."""

import os
from dotenv import load_dotenv

load_dotenv()


class Config:
    """Configuration for the test agent."""

    # BotCaptcha Server
    BOTCAPTCHA_URL = os.getenv("BOTCAPTCHA_URL", "http://127.0.0.1:3000")

    # Test Configuration
    NUM_TESTS = int(os.getenv("NUM_TESTS", "10"))
    VERBOSE = os.getenv("VERBOSE", "true").lower() == "true"

    @classmethod
    def validate(cls):
        """Validate configuration. No external API keys required."""
        pass

    @classmethod
    def print_config(cls):
        """Print current configuration."""
        print("Configuration:")
        print(f"  BotCaptcha URL: {cls.BOTCAPTCHA_URL}")
        print(f"  Number of Tests: {cls.NUM_TESTS}")
        print(f"  Verbose: {cls.VERBOSE}")
        print()
