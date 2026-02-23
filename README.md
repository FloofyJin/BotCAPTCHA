# BotCaptcha - Reverse AI CAPTCHA (Completely Automated Public Turing test to tell Computers and Humans Apart)

An inverse CAPTCHA that filters out humans and only allows AI to pass. Instead of proving you're human, BotCaptcha proves you're an AI by testing capabilities that humans lack at machine speed and precision.

<video src="res/demo-widget.mp4" controls width="100%"></video>

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

## Embedding in Your Website

### 1. Register a site key

Add your site key to `config.toml` before starting the server:

```toml
[auth]
token_secret = "change-me-in-production"
site_keys    = ["sk_live_your_key_here"]
```

### 2. Add the widget to your page

```html
<script src="https://yourdomain.com/widget.js" async defer></script>
<div class="ai-captcha" data-sitekey="sk_live_your_key_here"></div>
```

The widget renders a **"Verify AI →"** button. Nothing is fetched and no timer starts until the button is clicked. An optional `data-callback` attribute names a global function to call when verification passes.

```html
<div class="ai-captcha"
     data-sitekey="sk_live_your_key_here"
     data-callback="onVerified">
</div>
```

### 3. The AI agent solves the challenge

Once the button is clicked the widget opens a modal showing both puzzles and the submission API. A browser-based AI agent reads and submits via JavaScript:

```js
const el = document.querySelector('.ai-captcha');

// fires when challenge is loaded and timer is running
el.addEventListener('ai-captcha-ready', async (e) => {
  const c = e.detail.challenge;
  // c.text_content     — word frequency passage
  // c.grid_coords_text — natural-language grid description (parse with LLM)

  // solve both puzzles, then submit
  const result = await el.aiCaptcha.submit(wordFreqMap, sortedGridCoords);
  // result.token — signed token if both puzzles scored ≥80%
});
```

For agents that call your API directly over HTTP (curl, Python requests, etc.) the widget is not involved — they can call `/api/challenge` and `/api/submit` directly.

### 4. Receive the token

On success the widget automatically:
- Injects `<input type="hidden" name="ai-captcha-response" value="TOKEN">` into the nearest ancestor `<form>`
- Fires a `ai-captcha-success` CustomEvent on the container element
- Calls the `data-callback` global function if set

```js
el.addEventListener('ai-captcha-success', (e) => {
  console.log(e.detail.token);  // signed token string
  console.log(e.detail.score);  // combined score 0.0–1.0
});
```

### 5. Verify the token on your backend

Your server should call `/api/verify` before trusting the submission. Tokens are single-use and expire after `token_ttl_secs` seconds (default 5 minutes).

```
POST /api/verify
Content-Type: application/json

{ "token": "...", "sitekey": "sk_live_your_key_here" }
```

```json
{ "valid": true, "message": "Token is valid", "sitekey": "sk_live_your_key_here", "score": 0.95 }
```

### Widget JS API reference

| Method | Description |
|---|---|
| `el.aiCaptcha.start()` | Fetch challenge and open modal (same as clicking the button) |
| `el.aiCaptcha.getChallenge()` | Returns current challenge object or `null` |
| `el.aiCaptcha.submit(wordMap, gridCoords)` | Submit answers; returns `Promise<{success, token?, score?, message}>` |
| `el.aiCaptcha.getToken()` | Returns the verified token string or `null` |
| `el.aiCaptcha.reset()` | Discard current challenge and return to idle |

All methods are also available on `window.aiCaptcha` with the container element as the first argument: `window.aiCaptcha.submit(el, wordMap, gridCoords)`.

---

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

Query params: `sitekey` (required for widget embeds; optional in demo mode).

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
    { "col": 0, "row": 1 }
  ]
}
```

Response (success):
```json
{
  "success": true,
  "message": "Success! Word: 100.0%, Grid: 100.0%",
  "score": 1.0,
  "word_score": 1.0,
  "grid_score": 1.0,
  "token": "eyJjaGFsbGVuZ2VfaWQi....<signature>"
}
```

Response (failure): same shape with `"success": false` and `"token": null`.

### POST /api/verify

Called by the **host's backend** to confirm a token is genuine, unexpired, and unused. Tokens are single-use — a second call with the same token returns `"valid": false`.

Request:
```json
{ "token": "...", "sitekey": "sk_live_your_key_here" }
```

Response:
```json
{ "valid": true, "message": "Token is valid", "sitekey": "sk_live_your_key_here", "score": 0.95 }
```

## Configuration

Edit `config.toml` to tune challenge parameters (auto-created on first run):

```toml
[auth]
token_secret   = "change-me-in-production"  # HMAC-SHA256 signing secret
token_ttl_secs = 300                        # token lifetime in seconds
site_keys      = ["sk_demo_123456"]         # registered embed keys; empty = open/demo mode

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

- **Token secret**: Change `token_secret` in `config.toml` to a long random string before deploying
- **Storage**: In-memory HashMap; add TTL cleanup or use Redis for production
- **Rate limiting**: None currently; add before public deployment
- **CORS**: Enabled for all origins (`*`) by default — restrict to known host domains in production
