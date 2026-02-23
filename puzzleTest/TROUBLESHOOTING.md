# Troubleshooting Guide

## Problem: AI always gets 0% score

### Symptoms
- OpenAI successfully detects tiles
- Submission completes without errors
- But score is always 0.00%

### Diagnosis Steps

#### 1. Run the debug test

```bash
python debug_test.py
```

This will:
- Show actual tile positions vs AI detected positions
- Check timing (if you're exceeding the time limit)
- Save detailed debug info
- Keep browser open so you can inspect

**What to look for:**
- Compare "Detected bounding boxes" vs "Actual bounding boxes"
- Check if total time exceeds max allowed time
- Look for timing warnings

#### 2. Visualize the boxes

```bash
python debug_test.py      # Run first to generate data
python visualize_boxes.py  # Then visualize
```

Open `screenshots/debug_annotated.png`:
- **GREEN boxes** = Actual tiles with text
- **RED boxes** = What AI detected

**If boxes don't overlap:** AI is detecting wrong positions

**If boxes overlap well:** Problem is elsewhere (timing, validation, etc.)

#### 3. Test with perfect answers

```bash
python test_perfect_submission.py
```

This submits the **actual correct answers** (known positions).

**Expected result:** 100% score

**If you get 100%:**
- ✅ Submission path works
- ❌ Problem is AI detection accuracy

**If you get 0%:**
- ❌ Problem is submission or server validation
- Check server logs
- Check coordinate system

### Common Causes & Fixes

#### Cause 1: Timing Issue (Too Slow)

**Symptom:**
```
Total elapsed: 6234ms
Max allowed: 4000ms
⚠️  WARNING: Already exceeded time limit
```

**Why:** OpenAI API takes too long (2-4 seconds), challenge expires

**Fix:**
- Increase `max_time_ms` in `../config.toml`:
  ```toml
  [validation]
  max_time_ms = 10000  # 10 seconds
  ```
- Restart BotCaptcha server
- Re-run test

#### Cause 2: AI Detecting Wrong Positions

**Symptom:**
- `debug_annotated.png` shows RED boxes far from GREEN boxes
- AI detects 7 tiles but none match actual positions

**Why:**
- Screenshot quality issue
- Text not clearly visible
- AI misinterpreting visual elements

**Fix:**
1. Check `screenshots/debug_screenshot.png` manually
   - Can YOU see the text on tiles?
   - Is text rotated but readable?
   - Is contrast good?

2. Improve screenshot timing:
   ```python
   # In .env, increase delay
   SCREENSHOT_DELAY_MS=500  # Give more time to render
   ```

3. Try better prompt in `vision.py`:
   - Make prompt more specific
   - Add examples of what tiles look like
   - Emphasize rotation

#### Cause 3: Coordinate System Mismatch

**Symptom:**
- Boxes are offset by a constant amount
- Example: All AI boxes are 50px higher than actual

**Why:**
- Canvas position vs viewport coordinates
- Bounding box x,y meaning (center vs top-left)

**Check:**
```python
# Actual tiles use CENTER coordinates
tile.x, tile.y = center of tile

# Bounding boxes use TOP-LEFT coordinates
bbox.x, bbox.y = top-left corner
```

**Fix:**
Make sure conversion is correct:
```python
# In JavaScript (get_challenge_data):
bbox.x = tile.x - 30  # tile_size/2
bbox.y = tile.y - 30
bbox.width = 60
bbox.height = 60
```

#### Cause 4: Server Configuration Mismatch

**Symptom:**
- Perfect submission test also fails
- Server says "Too fast" or "Too slow" for valid times

**Check server config:** `../config.toml`

```toml
[challenge]
duration_ms_max = 10000  # Must match validation

[validation]
min_time_ms = 2000
max_time_ms = 10000  # Must be >= duration_ms_max
```

**Fix:** Make sure these match and restart server

#### Cause 5: IoU Threshold Too Strict

**Symptom:**
- Boxes look close in visualization
- But still 0% score

**Check:** `../config.toml`
```toml
[validation]
iou_threshold = 0.5  # Try lowering to 0.3
```

**Fix:**
1. Lower threshold temporarily:
   ```toml
   iou_threshold = 0.3
   ```
2. Restart server
3. Re-test

### Debug Checklist

- [ ] Run `debug_test.py` - check timing and positions
- [ ] Run `visualize_boxes.py` - visual comparison
- [ ] Run `test_perfect_submission.py` - verify submission works
- [ ] Check `screenshots/debug_screenshot.png` - is text visible?
- [ ] Check `screenshots/debug_annotated.png` - do boxes align?
- [ ] Check `debug_output.json` - review all data
- [ ] Check server logs - any errors?
- [ ] Verify `../config.toml` - timing settings correct?

### Still Stuck?

1. **Share debug data:**
   - `debug_output.json`
   - `screenshots/debug_screenshot.png`
   - `screenshots/debug_annotated.png`
   - Server logs

2. **Try manual test:**
   - Open http://127.0.0.1:3000 in browser
   - Start challenge
   - Manually click tiles
   - Submit
   - Does THAT work?

3. **Check if it's the backend:**
   ```bash
   cd ..
   cargo test  # Run backend tests
   ```

## Quick Diagnostic Commands

```bash
# Full diagnosis
python debug_test.py && python visualize_boxes.py

# Test submission path
python test_perfect_submission.py

# Check screenshot quality
open screenshots/debug_screenshot.png  # macOS
xdg-open screenshots/debug_screenshot.png  # Linux

# Check visualization
open screenshots/debug_annotated.png  # macOS
xdg-open screenshots/debug_annotated.png  # Linux

# Review debug data
cat debug_output.json | python -m json.tool
```

## Understanding the Output

### Good Output (AI Working):
```
Actual tiles with text: 7
AI detected tiles: 7
✓ Still have 3245ms remaining
Success: True
Score %: 85.7%
```

### Bad Output (Timing Issue):
```
Total elapsed: 6234ms
Max allowed: 4000ms
⚠️  WARNING: Already exceeded time limit
Success: False
Message: Too slow: 6234ms (maximum 4000ms)
```

### Bad Output (Detection Issue):
```
Actual tiles with text: 7
AI detected tiles: 7
✓ Still have 3245ms remaining
Success: False
Score %: 0.0%
⚠️  0% score means NO bounding boxes matched!
```

## Next Steps Based on Score

| Score | Meaning | Action |
|-------|---------|--------|
| 0% | No boxes matched | Check positions (run visualize_boxes.py) |
| 1-50% | Some matches | Improve AI prompt, check timing |
| 51-79% | Most match | Tweak threshold, improve screenshot |
| 80-100% | Success! | AI is working correctly |

## Getting Help

Include this info when asking for help:
1. Output from `debug_test.py`
2. Contents of `debug_output.json`
3. The annotated screenshot
4. Server config (`../config.toml`)
5. Any server error logs
