#!/usr/bin/env python3
"""Visualize detected vs actual bounding boxes on screenshot."""

import json
from PIL import Image, ImageDraw, ImageFont

def visualize_boxes(screenshot_path, detected_boxes, actual_tiles, output_path):
    """
    Draw bounding boxes on screenshot for comparison.

    Args:
        screenshot_path: Path to screenshot image
        detected_boxes: List of detected boxes from AI
        actual_tiles: List of actual tile data
        output_path: Where to save annotated image
    """
    # Load image
    img = Image.open(screenshot_path)
    draw = ImageDraw.Draw(img)

    # Draw actual boxes (GREEN)
    if actual_tiles:
        for tile in actual_tiles:
            bbox = tile['bbox']
            x = bbox['x']
            y = bbox['y']
            w = bbox['width']
            h = bbox['height']

            # Draw green rectangle
            draw.rectangle(
                [x, y, x + w, y + h],
                outline=(0, 255, 0),
                width=3
            )

            # Draw label
            label = f"ACTUAL: {tile['text']}"
            draw.text((x, y - 15), label, fill=(0, 255, 0))

    # Draw detected boxes (RED)
    for i, box in enumerate(detected_boxes):
        x = box['x']
        y = box['y']
        w = box['width']
        h = box['height']

        # Draw red rectangle
        draw.rectangle(
            [x, y, x + w, y + h],
            outline=(255, 0, 0),
            width=2
        )

        # Draw label
        label = f"AI #{i+1}"
        draw.text((x, y + h + 5), label, fill=(255, 0, 0))

    # Save
    img.save(output_path)
    print(f"✓ Annotated image saved to: {output_path}")


if __name__ == "__main__":
    # Load debug data
    try:
        with open("debug_output.json") as f:
            data = json.load(f)

        visualize_boxes(
            "screenshots/debug_screenshot.png",
            data['detected_boxes'],
            data['actual_tiles'],
            "screenshots/debug_annotated.png"
        )

        print("\nVisualization complete!")
        print("  GREEN boxes = Actual tiles with text")
        print("  RED boxes = AI detected tiles")
        print("\nIf they don't overlap, the AI is detecting wrong positions.")

    except FileNotFoundError:
        print("Error: Run debug_test.py first to generate debug_output.json")
