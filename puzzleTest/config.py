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

    # OpenAI API — used to parse the natural-language grid description
    OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
    OPENAI_MODEL = os.getenv("OPENAI_MODEL", "gpt-4o-mini")

    @classmethod
    def validate(cls):
        """Validate that required API keys are set."""
        if not cls.OPENAI_API_KEY:
            raise ValueError(
                "OPENAI_API_KEY not set. Add it to your .env file."
            )

    @classmethod
    def print_config(cls):
        """Print current configuration."""
        print("Configuration:")
        print(f"  BotCaptcha URL:  {cls.BOTCAPTCHA_URL}")
        print(f"  OpenAI Model:    {cls.OPENAI_MODEL}")
        print(f"  API Key:         {'set' if cls.OPENAI_API_KEY else 'NOT SET'}")
        print(f"  Number of Tests: {cls.NUM_TESTS}")
        print(f"  Verbose:         {cls.VERBOSE}")
        print()
