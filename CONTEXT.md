# BotCaptcha Development Context

## Project Goal
Create an inverse CAPTCHA that filters OUT humans and only allows AI to pass. The challenge should be trivial for AI vision systems but nearly impossible for humans.

## Completed ✅

### Backend (Rust + Axum)
- [x] Initialize Rust project with Axum 0.7
- [x] Implement `GET /api/challenge` endpoint
  - Generates 20-40 tiles with random positions
  - Assigns text to 5-10 random tiles
  - Returns challenge ID, tiles array, duration, grid size
- [x] Implement `POST /api/submit` endpoint
  - Validates timing (2-10 second window)
  - Checks bounding box overlap using IoU algorithm
  - Requires 80% accuracy to pass
- [x] In-memory HashMap storage for challenges
- [x] Static file serving for frontend
- [x] Server successfully builds with Rust 1.93.1

### Frontend (WebGL + JavaScript)
- [x] WebGL shader-based rendering
- [x] Rotating tile field with configurable speeds
- [x] Challenge timer and status display
- [x] API integration (fetch challenge, submit answer)
- [x] Programmatic API via `window.BotCaptchaAPI`

## Fixed Issues ✅

### 1. Text Not Visible - FIXED
**Problem**: Only seeing colored boxes (gray/green), no text labels
**Root Cause**: Canvas context conflict - can't use both WebGL and 2D context on same canvas
**Solution**: Implemented two overlaid canvases (WebGL layer + 2D text layer)
- Added `textCanvas` element in HTML
- Separated rendering: WebGL for tiles, 2D context for text
- CSS positioning to overlay canvases

### 2. Always 100% Success - FIXED
**Problem**: Submit always succeeds even when it shouldn't
**Root Cause**: Code was cheating by reading challenge data directly
**Solution**: Implemented click-to-select demo mode
- Added `selectedTiles` array to track user selections
- Canvas click handler detects tile clicks
- Visual feedback: selected tiles turn yellow
- Submit uses only selected tiles, not all text tiles

### 3. Submit Button Timing - FIXED
**Problem**: Submit button only becomes clickable after timer expires
**Root Cause**: Button only enabled in timeout handler
**Solution**: Enable submit button immediately when challenge starts
- Button enabled when challenge begins
- Status shows selected count: "Selected: X"

### 4. Text Rotation Not Working - FIXED
**Problem**: Text labels were not continuously rotating
**Root Cause**: Animation was stopped when timer expired
**Solution**: Keep animation running throughout challenge lifecycle
- Animation only stops when user submits or starts new challenge
- Text rotates continuously based on elapsed time calculation
- Timer expiration disables submit but keeps animation running

### 5. Challenge Flow Not Working - FIXED
**Problem**: No proper restart flow after success/failure/timeout
**Root Cause**: Incomplete state cleanup and button management
**Solution**: Implemented proper challenge lifecycle
- **Timeout**: Disables submit, shows "Time expired", enables restart
- **Success**: Shows success message, enables restart, cleans up
- **Failure**: Shows failure message, enables restart, cleans up
- All paths properly reset state and re-enable "Start Challenge" button

## To Do 📋

### Recently Completed
- [x] Fix canvas layering for text visibility (two-canvas solution)
- [x] Fix submit button to be enabled during challenge
- [x] Replace auto-submit with proper interaction model
- [x] Add click-to-select demo mode for testing
- [x] Add visual feedback for selected tiles (yellow highlight)
- [x] Implement proper answer collection (user clicks)
- [x] Modularize backend code into separate files
- [x] Add unit tests for IoU and scoring functions
- [x] Clean up main.rs to only handle server initialization
- [x] Create TOML-based configuration system
- [x] Make all challenge parameters configurable
- [x] Auto-create default config.toml on first run
- [x] Update all documentation for config system

### High Priority
- [ ] Test end-to-end flow with manual clicking
- [ ] Add AI integration example/documentation
- [ ] Server should NOT send `text` field in production (prevent cheating)
- [ ] Add keyboard shortcuts (spacebar to submit, ESC to cancel)

### Medium Priority
- [ ] Add challenge expiration/cleanup (TTL on HashMap entries)
- [ ] Add rate limiting on endpoints
- [ ] Show correct answers after submission for learning
- [ ] Add difficulty selector (easy/medium/hard)

### Low Priority
- [ ] Multiple challenge types (color match, pattern recognition)
- [ ] Analytics dashboard (success rate, timing stats)
- [ ] Challenge history/replay
- [ ] Leaderboard system

## Recent Changes

### Configuration System (Latest Session)
Added TOML-based configuration system:
- **`config.toml`** - User-editable configuration file
- **`src/config.rs`** - Config struct and loading logic
- Auto-creates default config on first run if missing
- All challenge parameters now configurable:
  - Tile count ranges
  - Grid size
  - Rotation speed ranges
  - Timer durations
  - Text pool
  - Validation thresholds
  - Server host/port

Benefits:
- No code changes needed to adjust parameters
- Easy experimentation with difficulty levels
- Clear documentation of all settings
- Can deploy with different configs per environment

### Code Modularization (Previous Session)
Refactored monolithic `src/main.rs` into modular architecture:
- **`src/models.rs`** - All data structures and type definitions
- **`src/handlers.rs`** - API route handler functions
- **`src/utils.rs`** - Helper functions with unit tests
- **`src/main.rs`** - Clean server initialization only

Benefits:
- Better code organization and maintainability
- Unit tests for IoU and scoring algorithms
- Easier to extend with new features
- Clear separation of concerns

### Previous Session Changes

### Canvas Layering Fix
- Added second canvas (`textCanvas`) for 2D text rendering
- Positioned absolutely over WebGL canvas
- Set `pointer-events: none` on text canvas
- Added crosshair cursor for better UX

### Click-to-Select Implementation
- Added `selectedTiles` array to track user selections
- Canvas click handler calculates tile distance
- Visual feedback: yellow for selected, green for text, gray for empty
- Status bar shows selection count

### Submit Button Fix
- Enabled immediately on challenge start (line 212)
- No longer requires waiting for timeout
- Still validates timing on server side

### Answer Submission Fix
- Changed from auto-submitting all text tiles (cheating)
- Now submits only user-selected tiles
- Allows for genuine testing and failure cases

## Architecture Decisions

### Why Two Canvases?
WebGL and 2D contexts are mutually exclusive on same canvas. Solution: Stack two canvases with absolute positioning. Text canvas overlays WebGL but doesn't capture clicks.

### Why Client-Side Text Data?
For demo purposes only. In production, server wouldn't send `text` field to prevent cheating. AI would need to use computer vision on rendered output.

### Why In-Memory Storage?
Proof-of-concept simplicity. Production would use Redis/DB with TTL for challenge expiration.

## Testing Strategy

### Manual Testing
1. Start server: `cargo run` or `cargo run --release`
2. Open `http://127.0.0.1:3000`
3. Click "Start Challenge"
4. **Verify initial state:**
   - Text is visible on green tiles and continuously rotating
   - Submit button is enabled immediately
   - Timer counts down
   - Status shows "Selected: 0"

5. **Test tile selection:**
   - Click on tiles with text
   - Selected tiles turn yellow
   - Status updates: "Selected: X"
   - Click again to deselect (yellow → green)
   - Text continues rotating throughout

6. **Test Challenge Flow Scenarios:**

   **Scenario A: Submit Incorrect Answer**
   - Start challenge
   - Select wrong tiles or miss some correct tiles
   - Click "Submit Answer" before timeout
   - Should show: "✗ Failed: Score X% (need 80%)"
   - "Start Challenge" button re-enabled
   - Can start new challenge

   **Scenario B: Timeout**
   - Start challenge
   - Wait for timer to reach 0
   - Should show: "Time expired! Click 'Start Challenge' to try again"
   - Submit button disabled
   - "Start Challenge" button enabled
   - Text continues rotating even after timeout
   - Can start new challenge

   **Scenario C: Submit Correct Answer**
   - Start challenge
   - Select all tiles with text (green ones)
   - Click "Submit Answer" before timeout
   - Should show: "✓ Success! Score: 100.00%"
   - "Start Challenge" button re-enabled
   - Can start new challenge

7. **Test Restart:**
   - After any scenario above, click "Start Challenge"
   - Old challenge clears completely
   - New challenge loads with different tiles
   - Everything works from fresh state

### AI Testing
```javascript
const challenge = await window.BotCaptchaAPI.getChallenge();
// AI vision processing here
const boxes = detectTextTiles(challenge);
const result = await window.BotCaptchaAPI.submitChallenge(challenge.challenge_id, boxes);
```

## Next Steps
1. Fix the three current issues
2. Add click-to-select functionality for demo
3. Test end-to-end flow
4. Document AI integration approach

## File Structure
```
BotCaptcha/
├── Cargo.toml              # Dependencies
├── config.toml             # Configuration file (auto-created)
├── README.md               # User documentation
├── CONTEXT.md              # This file - dev tracking
├── QUICKSTART.md           # Quick start guide
├── CLAUDE.md               # Claude Code guidance
├── src/
│   ├── main.rs            # Server initialization + config loading
│   ├── config.rs          # Configuration structure & loading
│   ├── models.rs          # Data structures & types
│   ├── handlers.rs        # API route handlers
│   └── utils.rs           # Helper functions (IoU, scoring) + tests
└── static/
    ├── index.html         # UI
    └── app.js             # Frontend logic
```

## Key Metrics
- Build time: ~15 seconds
- Binary size: TBD
- Challenge generation: <1ms
- Target response time: <100ms
