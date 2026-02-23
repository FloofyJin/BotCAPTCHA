# Quick Start Guide

Get the BotCaptcha test agent running in 5 minutes.

## Prerequisites

- Python 3.8+
- OpenAI API key

## 1. Setup (One-time)

### Option A: Automated Setup (Recommended)

```bash
cd puzzleTest
./setup.sh
```

Follow the prompts. This will:
- Install Python dependencies
- Install Playwright browser
- Create `.env` file

### Option B: Manual Setup

```bash
cd puzzleTest

# Install dependencies
pip install -r requirements.txt
playwright install chromium

# Configure environment
cp .env.example .env
# Edit .env and add your OPENAI_API_KEY
```

## 2. Add Your API Key

Edit `.env`:

```bash
OPENAI_API_KEY=sk-your-actual-openai-api-key-here
```

Get an API key from: https://platform.openai.com/api-keys

## 3. Start BotCaptcha Server

In another terminal:

```bash
cd ..
cargo run
```

Server should start at http://127.0.0.1:3000

## 4. Run Tests

### Quick Single Test (with browser visible)

```bash
python quick_test.py
```

This runs one test with the browser visible so you can see what's happening.

### Full Test Suite

```bash
# Run 10 tests
python main.py

# Run 5 tests in headless mode
python main.py -n 5 --headless

# Run 20 tests with verbose output
python main.py -n 20 -v
```

## Expected Output

```
Configuration:
  OpenAI Model: gpt-4o
  API Key: ********************
  BotCaptcha URL: http://127.0.0.1:3000
  Number of Tests: 10

============================================================
Running 10 tests...
============================================================

--- Test 1/10 ---
Loading http://127.0.0.1:3000...
Starting challenge...
Analyzing with OpenAI Vision API...
  Detected 7 text tiles
Submitting answer...
✓ PASSED - Score: 85.7% - Success! Score: 85.71%

...

============================================================
TEST SUMMARY
============================================================
Total Attempts:  10
Successes:       8 (80.0%)
Failures:        2
Errors:          0
============================================================

Results saved to: results/20260222_143022_results.json
```

## Output Files

- **Screenshots**: `screenshots/test_XXX_*.png` - Visual captures of each challenge
- **Results**: `results/TIMESTAMP_results.json` - Detailed test results and statistics

## Common Issues

### "OPENAI_API_KEY not set"
Edit `.env` and add your API key.

### "Connection refused"
Make sure BotCaptcha server is running (`cargo run` in parent directory).

### "Playwright not found"
Run: `playwright install chromium`

## What's Next?

- Check `README.md` for detailed documentation
- Analyze results in `results/` directory
- Review screenshots in `screenshots/` directory
- Adjust BotCaptcha difficulty in `../config.toml` if AI success rate is too high

## Interpreting Results

- **Success rate ≥80%**: AI bypasses BotCaptcha (make it harder!)
- **Success rate 50-79%**: Moderate AI success
- **Success rate <50%**: BotCaptcha successfully filters AI

The goal is to keep AI success rate **below 50%** while allowing manual testing to still work.
