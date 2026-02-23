# Configuration Guide

BotCaptcha uses a TOML configuration file (`config.toml`) to customize all challenge parameters.

## Auto-Generation

If `config.toml` doesn't exist when you start the server, it will automatically create one with default values.

## Configuration Sections

### [server]
Controls server binding.

```toml
host = "127.0.0.1"  # IP address to bind to
port = 3000         # Port to listen on
```

**Examples**:
- Listen on all interfaces: `host = "0.0.0.0"`
- Different port: `port = 8080`

### [challenge]
Controls challenge generation parameters.

```toml
num_tiles_min = 20              # Minimum tiles per challenge
num_tiles_max = 40              # Maximum tiles per challenge
num_text_tiles_min = 5          # Minimum tiles with text
num_text_tiles_max = 10         # Maximum tiles with text
duration_ms_min = 2000          # Minimum challenge duration (ms)
duration_ms_max = 10000         # Maximum challenge duration (ms)
grid_width = 800.0              # Canvas width in pixels
grid_height = 600.0             # Canvas height in pixels
tile_size = 60.0                # Tile size for bounding boxes
rotation_speed_min = -50.0      # Min rotation (degrees/second)
rotation_speed_max = 50.0       # Max rotation (degrees/second)
text_pool = ["ALPHA", ...]      # Available text labels
```

**Difficulty Tuning**:
- **Harder**: Increase tile count, decrease duration, increase rotation speed
- **Easier**: Decrease tile count, increase duration, decrease rotation speed

**Examples**:

Hard mode:
```toml
num_tiles_min = 30
num_tiles_max = 50
num_text_tiles_min = 8
num_text_tiles_max = 12
duration_ms_min = 1500
duration_ms_max = 3000
rotation_speed_min = -100.0
rotation_speed_max = 100.0
```

Easy mode:
```toml
num_tiles_min = 10
num_tiles_max = 20
num_text_tiles_min = 3
num_text_tiles_max = 5
duration_ms_min = 5000
duration_ms_max = 15000
rotation_speed_min = -20.0
rotation_speed_max = 20.0
```

### [validation]
Controls answer validation.

```toml
min_time_ms = 2000          # Minimum time before submission allowed
max_time_ms = 10000         # Maximum time before submission rejected
iou_threshold = 0.5         # IoU threshold for bounding box matching (0.0-1.0)
success_threshold = 0.8     # Accuracy required to pass (0.0-1.0)
```

**Notes**:
- `min_time_ms` prevents instant bot submissions
- `max_time_ms` should match `duration_ms_max` from challenge config
- `iou_threshold`: Higher = stricter matching (typical: 0.3-0.7)
- `success_threshold`: 0.8 = 80% of tiles must be correct

**Examples**:

Stricter validation:
```toml
iou_threshold = 0.7
success_threshold = 0.9
```

More lenient:
```toml
iou_threshold = 0.3
success_threshold = 0.6
```

## Custom Text Labels

You can customize the text pool with any labels you want:

```toml
text_pool = [
    "CAT", "DOG", "BIRD", "FISH",
    "🌟", "🎯", "🔥", "⚡",  # Emojis work too!
    "123", "456", "789",
]
```

## Applying Changes

1. Edit `config.toml`
2. Restart the server (Ctrl+C then `cargo run`)
3. Changes take effect immediately

## Troubleshooting

**Server won't start**:
- Check for syntax errors in `config.toml`
- Ensure min values are less than max values
- Verify port is not already in use

**Config not loading**:
- Delete `config.toml` to regenerate defaults
- Check server logs for error messages
- Ensure file is named exactly `config.toml`

**Validation failing unexpectedly**:
- Check that `max_time_ms` matches your `duration_ms_max`
- Ensure `success_threshold` is between 0.0 and 1.0
- Verify `iou_threshold` is reasonable (0.3-0.7 typical)
