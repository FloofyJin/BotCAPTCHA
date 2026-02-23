# BotCaptcha Quick Start

## 1. Start the Server

```bash
cargo run          # development
cargo run --release  # production (optimized)
```

Server starts on `http://127.0.0.1:3000`. On first run, `config.toml` is auto-created with defaults.

## 2. Try the Browser Demo

Open `http://127.0.0.1:3000` to see both puzzles side-by-side:

- **Left panel**: A word passage — count frequencies and paste JSON into the textarea
- **Right panel**: A prose description of grid coordinates — click the matching cells in reading order

Submit before the timer expires (4–6 seconds). Both puzzles must score ≥80% to pass.

> This is intentionally hard for humans. That's the point.

## 3. Run the AI Test Agent

The `puzzleTest/` directory contains a Python agent that solves challenges automatically.

```bash
cd puzzleTest

# One-time setup
cp .env.example .env        # then add your OPENAI_API_KEY
python -m venv venv
source venv/bin/activate    # Windows: venv\Scripts\activate
pip install -r requirements.txt

# Run tests
python main.py -n 10        # 10 tests, prints summary
python debug_test.py        # single detailed run, saves debug_output.json
```

Expected output:
```
✓ PASSED - Score: 100.0% - Success! Word: 100.0%, Grid: 100.0% (2728ms)
✓ PASSED - Score: 100.0% - Success! Word: 100.0%, Grid: 100.0% (2063ms)
...
Successes: 9 (90.0%)
AI successfully passes the BotCaptcha! (>=80% success rate)
```

## Configuration

Edit `config.toml` to adjust difficulty:

```toml
[challenge]
word_count_min = 150       # words in the passage
word_count_max = 250
duration_ms_min = 4000     # timer range (ms)
duration_ms_max = 6000
grid_coords_min = 10       # number of grid cells to find
grid_coords_max = 15

[validation]
success_threshold = 0.8    # 80% required on each puzzle
max_time_ms = 10000        # hard deadline for submission
```

Restart the server after changes.

## File Structure

```
BotCaptcha/
├── Cargo.toml              # Rust dependencies
├── config.toml             # Challenge parameters (auto-created)
├── README.md               # Project overview
├── QUICKSTART.md           # You are here
├── CONTEXT.md              # Development history
├── src/
│   ├── main.rs             # Server init and routing
│   ├── config.rs           # Config loading
│   ├── models.rs           # Data structures
│   ├── handlers.rs         # API route handlers
│   └── utils.rs            # Scoring + prose generation + tests
├── static/
│   ├── index.html          # Browser demo UI
│   └── app.js              # Frontend logic
└── puzzleTest/
    ├── agent.py            # TestAgent class
    ├── main.py             # CLI test runner
    ├── debug_test.py       # Verbose single-run debugger
    ├── config.py           # Env var loading
    ├── requirements.txt    # Python deps
    └── .env.example        # Copy to .env and fill in API key
```

## Programmatic API (JavaScript)

```javascript
// Fetch a challenge
const challenge = await window.BotCaptchaAPI.getChallenge();
// { challenge_id, text_content, duration_ms, grid_size, grid_coords_text }

// Solve word puzzle
const wordFreq = {};
for (const word of challenge.text_content.split(' ')) {
    wordFreq[word] = (wordFreq[word] || 0) + 1;
}

// Solve grid puzzle (after LLM parsing + sorting)
const sortedCoords = [{ col: 1, row: 0 }, { col: 0, row: 1 }, ...];

// Submit
const result = await window.BotCaptchaAPI.submitChallenge(
    challenge.challenge_id,
    wordFreq,
    sortedCoords
);
// { success: true, message: "...", word_score: 1.0, grid_score: 1.0 }
```
