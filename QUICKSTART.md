# BotCaptcha Quick Start Guide

## Running the Server

```bash
# Development mode (faster build, includes debug info)
cargo run

# Production mode (optimized, smaller binary)
cargo run --release
```

Server starts on: `http://127.0.0.1:3000` (configurable in `config.toml`)

**First Run**: If `config.toml` doesn't exist, the server automatically creates one with default values.

## Configuring the Challenge

Edit `config.toml` to customize:

```toml
[challenge]
num_tiles_min = 20              # Min number of tiles
num_tiles_max = 40              # Max number of tiles
duration_ms_min = 2000          # Min challenge time
duration_ms_max = 10000         # Max challenge time
rotation_speed_min = -50.0      # Rotation speed range
rotation_speed_max = 50.0

[validation]
success_threshold = 0.8         # 80% accuracy required
iou_threshold = 0.5             # Bounding box match threshold
```

Changes take effect on next server restart.

## Testing the Demo

1. **Open browser** to `http://127.0.0.1:3000`

2. **Start a challenge**
   - Click "Start Challenge" button
   - Timer begins counting down (2-10 seconds)
   - Observe rotating tiles

3. **Select tiles with text**
   - Look for green tiles with text labels
   - Click on tiles to select them
   - Selected tiles turn yellow
   - Click again to deselect

4. **Submit your answer**
   - Click "Submit Answer" before time expires
   - View your score (need 80% to pass)

## Expected Behavior

### What You Should See
- ✅ Rotating square tiles (gray and green)
- ✅ Text labels on green tiles (ALPHA, BETA, etc.)
- ✅ Text continuously rotates throughout entire challenge
- ✅ Submit button enabled immediately when challenge starts
- ✅ Status shows: "Time remaining: Xms | Selected: Y"

### Interaction
- ✅ Click tiles to select/deselect
- ✅ Selected tiles turn yellow
- ✅ Submit works any time during challenge (before timeout)
- ✅ Crosshair cursor over canvas
- ✅ Text keeps rotating even after timeout

### Challenge Flow
**Submit with incorrect/incomplete answer:**
- Shows: "✗ Failed: Score X% (need 80%)"
- Can restart challenge

**Timer expires:**
- Shows: "Time expired! Click 'Start Challenge' to try again"
- Submit button disabled
- Animation continues (text keeps rotating)
- Can restart challenge

**Submit with correct answer (80%+):**
- Shows: "✓ Success! Score: X%"
- Can restart challenge

**All outcomes:**
- "Start Challenge" button re-enabled
- State fully reset for next attempt

### Scoring
- **100%** = All text tiles selected, no extras
- **80%+** = Pass (green success message)
- **<80%** = Fail (red error message)
- **Too fast** = Submitted in <2 seconds (server validates)
- **Too slow** = Submitted after time expired (submit disabled)

## For AI Integration

```javascript
// Get challenge data
const challenge = await window.BotCaptchaAPI.getChallenge();

// Process with computer vision (your code here)
const detectedTiles = yourVisionModel(challenge);

// Convert to bounding boxes
const boundingBoxes = detectedTiles.map(tile => ({
    x: tile.x - 25,
    y: tile.y - 25,
    width: 50,
    height: 50
}));

// Submit
const result = await window.BotCaptchaAPI.submitChallenge(
    challenge.challenge_id,
    boundingBoxes
);

console.log(result);
// { success: true, message: "Success! Score: 100.00%", score: 1.0 }
```

## Common Issues

### Text Not Visible
- **Fixed!** Now uses dual-canvas approach
- If still missing, check browser console for errors

### Always 100% Score
- **Fixed!** No longer auto-submits all text tiles
- Must manually click tiles to select them

### Submit Button Disabled
- **Fixed!** Enabled immediately when challenge starts
- If disabled, click "Start Challenge" first

## File Overview

```
BotCaptcha/
├── QUICKSTART.md          ← You are here
├── CONTEXT.md             ← Development tracking
├── README.md              ← Project overview
├── Cargo.toml             ← Rust dependencies
├── src/main.rs            ← Backend server
└── static/
    ├── index.html         ← UI
    └── app.js             ← Frontend logic
```

## Next Steps

1. **Test the demo** - Make sure everything works
2. **Read CONTEXT.md** - See what's been done and what's next
3. **Integrate AI** - Use the programmatic API
4. **Customize** - Adjust tile count, timing, difficulty in `src/main.rs`

## Production Considerations

⚠️ **Security**: Server currently sends `text` field in challenge JSON. In production, remove this to prevent cheating. AI should use computer vision on rendered output only.

⚠️ **Storage**: Currently uses in-memory HashMap. Add TTL cleanup or use Redis for production.

⚠️ **Rate Limiting**: Add rate limiting to prevent abuse.

⚠️ **CORS**: Configure CORS if frontend is on different domain.
