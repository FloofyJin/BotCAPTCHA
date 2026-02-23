# BotCaptcha - AI-Only Challenge System

An inverse CAPTCHA that filters out humans and only allows AI to pass. Instead of proving you're human, BotCaptcha proves you're an AI by testing capabilities that humans lack at machine speed and precision.

## How It Works

Each challenge presents two simultaneous puzzles. Both must be solved correctly (≥80% each) within a 4–6 second window.

### Puzzle 1: Word Frequency Count

The server generates a 150–250 word passage sampled from a word pool. The respondent must count every word's frequency and return a JSON map:

```json
{ "the": 7, "node": 6, "loop": 5, ... }
```

**Why humans fail**: Manual counting of ~200 words in under 6 seconds is effectively impossible, and formatting the result as JSON adds another barrier.

**Why AI passes**: Direct text from the API response — no vision needed. Word counting is a trivial string operation.

### Puzzle 2: Grid Coordinate Ordering

The server picks 10–15 random cells from a 10×10 grid and describes them in scrambled natural-language prose:

> "The 12 cells you need to select are located at: the cell where column 5 meets row 1, and then 3 columns from the left, 3 rows from the top; continuing with the square at column 9 in row 3..."

The respondent must extract all coordinates and submit them sorted in reading order (top-to-bottom, left-to-right).

**Why humans fail**: Parsing varied prose phrasings, extracting coordinates, and sorting them in 4–6 seconds is extremely difficult.

**Why AI passes**: An LLM trivially parses the prose, and sorting by `(row, col)` is a one-liner.

## Architecture

### Backend (Rust + Axum)

- `GET /api/challenge` — Generates and returns a new dual-puzzle challenge
- `POST /api/submit` — Validates both answers, enforces timing, returns scores
- Static file serving for the browser demo

### Frontend (Vanilla JS)

- Displays the word passage and grid puzzle side-by-side
- Interactive 10×10 grid for manual testing
- `window.BotCaptchaAPI` for programmatic AI integration

### Test Agent (Python, `puzzleTest/`)

- Fetches challenges via HTTP
- Solves word puzzle with direct word counting
- Solves grid puzzle by calling an OpenAI LLM to parse the prose
- **90%+ pass rate** observed in testing

## Running the Server

```bash
# Development
cargo run

# Production (optimized)
cargo run --release

# Unit tests (10 tests)
cargo test
```

Server starts on `http://127.0.0.1:3000`

## API

### GET /api/challenge

```json
{
  "challenge_id": "uuid",
  "text_content": "node host byte man was too how...",
  "duration_ms": 5200,
  "grid_size": 10,
  "grid_coords_text": "The 12 cells you need to select are located at: ..."
}
```

### POST /api/submit

Request:
```json
{
  "challenge_id": "uuid",
  "answer": { "node": 6, "host": 4, "byte": 3 },
  "grid_answer": [
    { "col": 1, "row": 0 },
    { "col": 0, "row": 1 },
    ...
  ]
}
```

Response:
```json
{
  "success": true,
  "message": "Success! Word: 100.0%, Grid: 100.0%",
  "score": 1.0,
  "word_score": 1.0,
  "grid_score": 1.0
}
```

## Configuration

Edit `config.toml` to tune challenge parameters (auto-created on first run):

```toml
[challenge]
word_count_min = 150
word_count_max = 250
duration_ms_min = 4000
duration_ms_max = 6000
grid_size = 10
grid_coords_min = 10
grid_coords_max = 15

[validation]
success_threshold = 0.8   # 80% required on each puzzle
min_time_ms = 0
max_time_ms = 10000
```

## Test Agent

```bash
cd puzzleTest
cp .env.example .env      # add your OPENAI_API_KEY
python -m venv venv && source venv/bin/activate
pip install -r requirements.txt

python main.py -n 10      # run 10 tests
python debug_test.py      # single verbose run with debug output
```

## Scoring

- `word_score` = fraction of words with exactly correct count
- `grid_score` = fraction of coordinates in the correct position (0.0 if wrong count submitted)
- `success` = both scores ≥ 80%
- `combined_score` = average of both scores

## Production Considerations

- **Storage**: In-memory HashMap; add TTL cleanup or use Redis for production
- **Rate limiting**: None currently; add before public deployment
- **CORS**: Configure if frontend is on a different domain
