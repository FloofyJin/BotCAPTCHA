# BotCaptcha - AI-Only Challenge System

An inverse CAPTCHA system that filters out humans and only allows AI to pass. This proof-of-concept uses rotating tile fields, temporal constraints, and parallel perception requirements to create challenges that AI can solve but humans cannot.

## Features

- **Rotating Tile Field**: WebGL-rendered tiles that rotate at different speeds
- **Temporal Constraints**: Challenges must be completed within 2-10 seconds
- **Procedural Generation**: Each challenge is unique with randomized parameters
- **Bounding Box Validation**: Uses IoU (Intersection over Union) for precise answer checking
- **Server-Side Timing**: Prevents client-side manipulation
- **In-Memory Storage**: Fast challenge storage using HashMap

## Architecture

### Backend (Rust + Axum)
- `GET /api/challenge` - Generates and returns a new challenge
- `POST /api/submit` - Validates submitted answers
- Static file serving for frontend

### Frontend (Vanilla JS + WebGL)
- WebGL shader-based rotating tiles
- 2D canvas overlay for text rendering
- Programmatic API for AI integration

## How It Works

1. Server generates 20-40 tiles with random positions and rotation speeds
2. 5-10 tiles are randomly assigned text labels
3. Client receives challenge data and renders rotating tiles
4. Observer must identify all text-containing tiles within the time limit
5. Bounding boxes are submitted and validated using IoU scoring
6. 80% accuracy required to pass

## Why Humans Fail

- **Parallel Perception**: Must track multiple rotating objects simultaneously
- **Temporal Compression**: Very short observation window (2-10 seconds)
- **Precision Output**: Requires exact bounding box coordinates
- **Cognitive Overload**: Combination of rotation, text recognition, and time pressure

## Running the Server

```bash
# Development mode
cargo run

# Production mode (optimized)
cargo run --release

# Run tests
cargo test
```

Server will start on `http://127.0.0.1:3000`

## Testing

Open your browser to `http://127.0.0.1:3000` to see the challenge interface.

For AI testing, use the programmatic API:

```javascript
// Get a new challenge
const challenge = await window.BotCaptchaAPI.getChallenge();

// Process the challenge (AI vision would go here)
const boundingBoxes = [
    { x: 100, y: 200, width: 50, height: 50 },
    // ... more bounding boxes
];

// Submit answer
const result = await window.BotCaptchaAPI.submitChallenge(
    challenge.challenge_id,
    boundingBoxes
);

console.log(result); // { success: true/false, message: "...", score: 0.95 }
```

## Configuration

Edit `config.toml` to customize challenge parameters:

```toml
[server]
host = "127.0.0.1"
port = 3000

[challenge]
num_tiles_min = 20              # Minimum number of tiles
num_tiles_max = 40              # Maximum number of tiles
num_text_tiles_min = 5          # Min tiles with text
num_text_tiles_max = 10         # Max tiles with text
duration_ms_min = 2000          # Min challenge duration (ms)
duration_ms_max = 10000         # Max challenge duration (ms)
grid_width = 800.0              # Canvas width
grid_height = 600.0             # Canvas height
tile_size = 60.0                # Tile size for bounding boxes
rotation_speed_min = -50.0      # Min rotation speed (deg/sec)
rotation_speed_max = 50.0       # Max rotation speed (deg/sec)
text_pool = ["ALPHA", "BETA", ...] # Text labels

[validation]
min_time_ms = 2000              # Minimum submission time
max_time_ms = 10000             # Maximum submission time
iou_threshold = 0.5             # IoU threshold for matching
success_threshold = 0.8         # Required accuracy (80%)
```

If `config.toml` doesn't exist, the server will create one with default values on first run.

## Future Enhancements

- Multiple challenge types (color matching, pattern recognition, etc.)
- Difficulty levels
- Challenge persistence and expiration
- Rate limiting
- Analytics and success metrics
