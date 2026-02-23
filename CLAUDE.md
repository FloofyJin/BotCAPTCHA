# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BotCaptcha is an **inverse CAPTCHA** system that filters OUT humans and only allows AI to pass. Unlike traditional CAPTCHAs that prove you're human, this system proves you're an AI by testing capabilities humans lack: parallel perception, temporal compression, and precision output.

**Key Concept**: Challenges are trivial for AI vision systems but nearly impossible for humans due to rotating objects, short time windows (2-10s), and requirement for precise bounding box coordinates.

## Build and Run

```bash
# Development (faster compile, includes debug info)
cargo run

# Production (optimized build)
cargo run --release

# Run unit tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

Server runs on `http://127.0.0.1:3000`

## Architecture

### Two-Layer Rendering System

The frontend uses **dual overlaid canvases** because WebGL and 2D contexts are mutually exclusive on the same canvas:

- **`glCanvas`** (bottom layer): WebGL shader renders rotating tile squares
- **`textCanvas`** (top layer): 2D context renders rotating text overlays
- CSS positioning: both absolute, text canvas has `pointer-events: none`

This architecture is critical - modifying the canvas structure will break text rendering.

### Challenge Flow

1. **`GET /api/challenge`** generates:
   - 20-40 tiles with random positions and rotation speeds
   - 5-10 tiles randomly selected to display text
   - Server stores `Challenge` with correct answers in HashMap
   - Client receives tile data including `text` field (demo only - production should hide this)

2. **Frontend renders**:
   - WebGL animates rotating squares (green = has text, gray = empty, yellow = user selected)
   - 2D canvas overlays text that rotates in sync
   - Click handler lets users select tiles

3. **`POST /api/submit`** validates:
   - Timing enforcement (2000-10000ms window)
   - IoU (Intersection over Union) matching between submitted and correct bounding boxes
   - 80% accuracy threshold required to pass

### State Management

- **Server**: `Arc<RwLock<HashMap<String, Challenge>>>` - in-memory storage keyed by UUID
  - No TTL/expiration currently implemented (TODO for production)
- **Client**: `selectedTiles[]` array tracks tile indices user clicked
  - In AI mode, this would be populated by computer vision instead of clicks

### Server Architecture (Axum 0.7)

Uses manual connection handling due to Axum 0.7 API:
- `TokioIo` wrapper for socket
- `hyper::service::service_fn` to convert Tower service
- `Builder::serve_connection` for each accepted connection
- This is a workaround for edition compatibility issues during development

## Code Organization

The backend is modularized into:

- **`src/main.rs`** - Server initialization, config loading, and routing setup
- **`src/config.rs`** - Configuration structure and file loading
- **`src/models.rs`** - Data structures (Challenge, Tile, BoundingBox, API request/response types, AppState)
- **`src/handlers.rs`** - API route handlers (`create_challenge`, `submit_answer`)
- **`src/utils.rs`** - Helper functions (IoU calculation, scoring algorithm) with unit tests
- **`config.toml`** - Configuration file (auto-created if missing)

## Configuration System

All challenge parameters are configured in `config.toml`:

```toml
[challenge]
num_tiles_min = 20           # Total tiles rendered (min)
num_tiles_max = 40           # Total tiles rendered (max)
num_text_tiles_min = 5       # Tiles with text (min)
num_text_tiles_max = 10      # Tiles with text (max)
duration_ms_min = 2000       # Time window min (ms)
duration_ms_max = 10000      # Time window max (ms)
grid_width = 800.0           # Canvas width
grid_height = 600.0          # Canvas height
tile_size = 60.0             # Tile size for bounding boxes
rotation_speed_min = -50.0   # Rotation speed min (deg/s)
rotation_speed_max = 50.0    # Rotation speed max (deg/s)

[validation]
success_threshold = 0.8      # 80% accuracy required
iou_threshold = 0.5          # IoU threshold for box matching
min_time_ms = 2000           # Minimum submission time
max_time_ms = 10000          # Maximum submission time
```

The config is loaded on startup and shared across all handlers via `AppState`.

## AI Integration API

Frontend exposes `window.BotCaptchaAPI`:

```javascript
// Get challenge
const challenge = await window.BotCaptchaAPI.getChallenge();

// Run computer vision on challenge.tiles
const detectedBoxes = yourVisionModel(challenge);

// Submit bounding boxes
const result = await window.BotCaptchaAPI.submitChallenge(
    challenge.challenge_id,
    boundingBoxes  // Array of {x, y, width, height}
);
```

## Security Considerations

**Current Implementation (Demo Mode)**:
- Server sends `text` field in challenge JSON
- Client knows which tiles have text
- Users can cheat by inspecting network traffic

**Production Requirements**:
- Remove `text` field from `Tile` serialization before sending to client
- AI must use computer vision on rendered canvas output only
- Add rate limiting to prevent brute force
- Implement challenge expiration/cleanup

## Critical Implementation Details

### IoU Calculation (`src/utils.rs`)
Uses Intersection over Union algorithm to match bounding boxes with 0.5 IoU threshold. This is standard in object detection - don't change without understanding impact on scoring. Includes unit tests for verification.

### Timing Validation (`src/handlers.rs` in `submit_answer`)
Server-side timing prevents client manipulation. Both min (2000ms) and max (10000ms) enforced. Stored as Unix timestamp milliseconds.

### Canvas Click Detection (`static/app.js`)
Calculates Euclidean distance from click to tile center. 30px radius threshold. Toggles selection on/off for same tile.

### Challenge Generation (`src/handlers.rs` in `create_challenge`)
Procedurally generates tiles with random positions, rotation speeds, and text assignments. Creates ground-truth bounding boxes for validation.

## Known Issues / TODOs

- HashMap has no expiration - challenges accumulate in memory (add TTL cleanup)
- No rate limiting on API endpoints
- Text field sent to client enables cheating (remove for production)
- Manual Hyper connection loop could be simplified with updated dependencies
- Unused `tiles` field warning in `Challenge` struct (used implicitly via serialization)

## Testing

Manual testing flow in browser:
1. Start challenge
2. Click tiles with visible text (green squares with rotating labels)
3. Selected tiles turn yellow
4. Submit before timer expires
5. Score must be 80%+ to pass

Edge cases to test:
- Submit too early (<2s) → "Too fast" error
- Submit after timeout (>duration_ms) → "Too slow" error
- Zero selections → 0% score
- Partial selections → proportional score based on IoU matching
