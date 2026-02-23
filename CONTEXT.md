# BotCaptcha Development Context

## Project Goal

Create an inverse CAPTCHA that filters OUT humans and only allows AI to pass. Challenges are trivial for AI but effectively impossible for humans within the time constraints.

## Current State

The system presents two simultaneous puzzles. Both must score ≥80% within a 4–6 second window.

### Puzzle 1: Word Frequency Count
- Server generates a 150–250 word passage from a 50-word pool
- Respondent returns a JSON map of `{word: count}` for every word
- Trivial for AI (direct string operation); extremely tedious for humans

### Puzzle 2: Grid Coordinate Ordering (natural language)
- Server picks 10–15 random cells from a 10×10 grid
- Describes them in scrambled prose with varied phrasings
- Respondent must parse prose, extract coordinates, sort into reading order
- Requires an LLM to parse reliably; simple scripts and regex fail

### Test Agent
- Python agent in `puzzleTest/` solves both puzzles automatically
- Word puzzle: direct word count
- Grid puzzle: OpenAI LLM extracts coordinates from prose, then sorted by `(row, col)`
- **90%+ pass rate** in testing (well above the 80% threshold)

## Completed ✅

### Architecture Redesign (Text-Based Puzzles)
- [x] Replaced WebGL rotating-tile visual challenge with text-based dual puzzles
- [x] Word frequency puzzle: passage generation, frequency scoring
- [x] Grid ordering puzzle: 10×10 grid, natural-language coordinate descriptions
- [x] Both puzzles scored independently; combined score = average

### Natural Language Grid Descriptions
- [x] `generate_grid_description()` in `src/utils.rs` — varied prose templates
- [x] 5 unambiguous coordinate description phrasings (all column-first)
- [x] Exact count embedded in prose opening ("The 12 cells you need to select...")
- [x] Scrambled order noted explicitly at the end of the prose
- [x] Removed ambiguous templates: ordinal phrasing (0-indexed confusion), reversed "row X and column Y" order

### Backend
- [x] `src/config.rs` — word count, duration, grid size params
- [x] `src/models.rs` — `GridCoord`, `ChallengeResponse` with `grid_coords_text`, `SubmitRequest` with `grid_answer`
- [x] `src/handlers.rs` — generates both puzzles, scores both, returns per-puzzle scores
- [x] `src/utils.rs` — `score_word_frequencies`, `score_grid_answer`, `generate_grid_description`, `describe_coord`; 10 unit tests
- [x] TOML-based configuration system with auto-creation of defaults

### Frontend
- [x] Two-column layout: word puzzle (left) + grid puzzle (right)
- [x] Interactive 10×10 CSS grid with click-to-number selection
- [x] Prose description displayed above grid
- [x] `window.BotCaptchaAPI` for programmatic AI integration

### Test Agent (`puzzleTest/`)
- [x] Direct HTTP requests (no browser/Playwright/vision model needed)
- [x] OpenAI SDK for grid LLM parsing (`gpt-4o` / `gpt-4o-mini`)
- [x] `agent.py` — `TestAgent` class
- [x] `main.py` — CLI runner with `-n` flag
- [x] `debug_test.py` — verbose single run, saves `debug_output.json`
- [x] `config.py` — env var loading and validation
- [x] `.env.example` — documents required env vars

## Known Issues / To Do

### Medium Priority
- [ ] HashMap has no expiration — challenges accumulate in memory (add TTL cleanup)
- [ ] No rate limiting on API endpoints
- [ ] Grid puzzle: occasional LLM failure (~10%) when it miscounts extracted coordinates

### Low Priority
- [ ] Show correct answers after submission in browser demo
- [ ] Analytics: track per-puzzle pass rates over time
- [ ] Multiple word pool themes (tech, nature, etc.)

## Design Decisions

### Why Natural Language for Grid Coordinates?
A structured JSON array of coordinates is trivially parseable by any script — it provides no meaningful barrier. Natural language prose with varied phrasings requires genuine language understanding (an LLM) and is far harder to parse with simple regex or pattern matching.

### Why Embed the Count in the Prose?
Without an explicit count, LLMs tend to hallucinate additional coordinates or miss some. Saying "The 12 cells..." gives the model a hard constraint and reliably eliminates over-generation.

### Why Remove Ordinal and Reversed Templates?
- **Ordinal** ("the ninth column of the fifth row"): 0-indexed ordinals conflict with natural language 1-indexed convention, causing systematic off-by-one errors in LLM parsing
- **Reversed** ("row X and column Y"): Unusual ordering confused the LLM about which value was col vs row, causing coordinate swaps

### Why OpenAI Instead of Anthropic for the Test Agent?
User preference. The agent uses `gpt-4o` / `gpt-4o-mini` via the OpenAI SDK.

### Why In-Memory Storage?
Proof-of-concept simplicity. Production would use Redis/DB with TTL for challenge expiration.

## Scoring Logic

```
word_score  = exactly_correct_words / total_unique_words
grid_score  = positional_matches / total_coords   (0.0 if lengths differ)
success     = word_score >= 0.8 AND grid_score >= 0.8
combined    = (word_score + grid_score) / 2.0
```

## File Structure

```
BotCaptcha/
├── Cargo.toml              # Rust dependencies
├── config.toml             # Challenge parameters (auto-created)
├── README.md               # User documentation
├── QUICKSTART.md           # Quick start guide
├── CONTEXT.md              # This file — development tracking
├── CLAUDE.md               # Claude Code guidance
├── src/
│   ├── main.rs             # Server init and routing
│   ├── config.rs           # Configuration structure & loading
│   ├── models.rs           # Data structures & types
│   ├── handlers.rs         # API route handlers
│   └── utils.rs            # Scoring, prose generation, unit tests
├── static/
│   ├── index.html          # Browser demo UI
│   └── app.js              # Frontend logic
└── puzzleTest/
    ├── agent.py            # TestAgent class
    ├── main.py             # CLI test runner (-n NUM_TESTS)
    ├── debug_test.py       # Single verbose run
    ├── config.py           # Env var loading
    ├── requirements.txt    # openai, requests, python-dotenv
    └── .env.example        # OPENAI_API_KEY, OPENAI_MODEL, BOTCAPTCHA_URL
```
