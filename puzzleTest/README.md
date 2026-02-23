# BotCaptcha Test Agent

AI-powered test agent that uses OpenAI's Vision API to attempt solving BotCaptcha challenges.

## Purpose

This agent tests whether AI can successfully pass the BotCaptcha inverse CAPTCHA system. It:
1. Loads the BotCaptcha web interface
2. Captures screenshots of challenge tiles
3. Uses OpenAI Vision API to identify tiles with text
4. Submits bounding boxes for validation
5. Tracks success rates and statistics

## Prerequisites

- Python 3.8 or higher
- OpenAI API key with vision model access
- Running BotCaptcha server (default: http://127.0.0.1:3000)

## Installation

### 1. Install Python Dependencies

```bash
cd puzzleTest
pip install -r requirements.txt
```

### 2. Install Playwright Browsers

```bash
playwright install-deps
playwright install chromium
```

### 3. Configure API Key

Copy the example environment file and add your OpenAI API key:

```bash
cp .env.example .env
```

Edit `.env` and set your API key:

```
OPENAI_API_KEY=sk-your-actual-api-key-here
```

## Configuration

Edit `.env` to customize:

```bash
# OpenAI Configuration
OPENAI_API_KEY=your-api-key-here
OPENAI_MODEL=gpt-4o  # or gpt-4-vision-preview

# BotCaptcha Server
BOTCAPTCHA_URL=http://127.0.0.1:3000

# Test Settings
NUM_TESTS=10
SCREENSHOT_DELAY_MS=100
VERBOSE=true
```

## Usage

### Basic Usage

Run 10 tests with default settings:

```bash
python main.py
```

### Advanced Options

```bash
# Run 5 tests
python main.py -n 5

# Run in headless mode (no visible browser)
python main.py --headless

# Don't save screenshots (saves disk space)
python main.py --no-screenshots

# Use specific model
python main.py --model gpt-4-vision-preview

# Custom server URL
python main.py --url http://localhost:8080

# Verbose output
python main.py -v

# Combine options
python main.py -n 20 --headless -v
```

### Help

```bash
python main.py --help
```

## How It Works

### 1. Browser Automation
- Uses Playwright to control a Chromium browser
- Navigates to BotCaptcha interface
- Clicks "Start Challenge" button
- Waits for canvas to render with rotating tiles

### 2. Screenshot Capture
- Captures the canvas element containing the challenge
- Converts to base64 PNG for OpenAI API
- Optionally saves to `screenshots/` directory

### 3. Vision Analysis
- Sends screenshot to OpenAI Vision API (GPT-4o or GPT-4 Vision)
- Provides detailed prompt explaining the task
- Requests JSON array of bounding boxes for tiles with text
- Parses response and validates format

### 4. Answer Submission
- Submits bounding boxes via `/api/submit` endpoint
- Receives validation result (success/failure, score)
- Tracks statistics across multiple tests

## Output

### Console Output

```
Configuration:
  OpenAI Model: gpt-4o
  API Key: ********************
  BotCaptcha URL: http://127.0.0.1:3000
  Number of Tests: 10
  Verbose: True

============================================================
Running 10 tests...
============================================================

--- Test 1/10 ---
Loading http://127.0.0.1:3000...
Starting challenge...
Screenshot saved to screenshots/test_001_abc123.png

Challenge abc123:
  Tiles: 32
  Duration: 5432ms

Analyzing with OpenAI Vision API...
  Detected 7 text tiles
    Box 1: x=120.0, y=200.0, w=60.0, h=60.0
    Box 2: x=350.0, y=150.0, w=60.0, h=60.0
    ...

Submitting answer...
✓ PASSED - Score: 85.7% - Success! Score: 85.71%

--- Test 2/10 ---
...

============================================================
TEST SUMMARY
============================================================
Total Attempts:  10
Successes:       8 (80.0%)
Failures:        2
Errors:          0
============================================================

🎉 AI successfully passes the BotCaptcha! (≥80% success rate)

Results saved to: results/20260222_143022_results.json
```

### Results File

Results are saved to `results/TIMESTAMP_results.json`:

```json
{
  "timestamp": "2026-02-22T14:30:22",
  "config": {
    "model": "gpt-4o",
    "url": "http://127.0.0.1:3000",
    "num_tests": 10
  },
  "statistics": {
    "total_attempts": 10,
    "successes": 8,
    "failures": 2,
    "errors": 0,
    "success_rate": 80.0
  },
  "results": [
    {
      "test_number": 1,
      "success": true,
      "score": 0.857,
      "message": "Success! Score: 85.71%",
      "challenge_id": "abc-123-def",
      "num_tiles": 32,
      "num_detected": 7,
      "elapsed_ms": 2341.5
    }
  ]
}
```

## Directory Structure

```
puzzleTest/
├── main.py              # Entry point
├── agent.py             # Test agent logic
├── vision.py            # OpenAI Vision API integration
├── screenshot.py        # Browser automation
├── config.py            # Configuration management
├── requirements.txt     # Python dependencies
├── .env.example         # Example environment file
├── .env                 # Your API keys (create this)
├── README.md            # This file
├── screenshots/         # Saved screenshots (created)
└── results/             # Test results (created)
```

## Interpreting Results

### Success Rate Interpretation

- **≥80%**: AI successfully bypasses BotCaptcha
- **50-79%**: Moderate AI success, inconsistent
- **<50%**: BotCaptcha successfully filters AI

### Common Issues

**Challenge ID not found:**
- Ensure BotCaptcha server is running
- Check URL in `.env` matches server

**OpenAI API errors:**
- Verify API key is valid
- Check you have vision model access
- Monitor API rate limits

**Low success rate:**
- This is expected! BotCaptcha is designed to be hard for AI
- Try adjusting challenge difficulty in `../config.toml`
- Check screenshots to verify rendering quality

**High success rate (>80%):**
- AI is successfully solving the challenges
- Consider making BotCaptcha harder:
  - Increase rotation speed
  - Decrease duration
  - Add more tiles
  - Reduce tile size

## Cost Estimation

OpenAI Vision API costs (as of 2026):
- GPT-4o: ~$0.005-0.01 per image
- 10 tests ≈ $0.05-0.10
- 100 tests ≈ $0.50-1.00

## Troubleshooting

### Playwright Installation Issues

```bash
# Install browsers manually
playwright install chromium

# Or install all browsers
playwright install
```

### Module Import Errors

```bash
# Ensure you're in the puzzleTest directory
cd puzzleTest
python main.py
```

### Browser Not Starting

```bash
# Run in visible mode for debugging
python main.py  # without --headless

# Check Playwright installation
playwright --version
```

## License

Same as parent BotCaptcha project.
