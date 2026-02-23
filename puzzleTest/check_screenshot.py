#!/usr/bin/env python3
"""Check if screenshot actually contains readable text."""

from openai import OpenAI
import base64
from config import Config

Config.validate()

# Load the debug screenshot
screenshot_path = "screenshots/debug_screenshot.png"

print(f"Checking screenshot: {screenshot_path}\n")

with open(screenshot_path, "rb") as f:
    screenshot_bytes = f.read()
    screenshot_base64 = base64.b64encode(screenshot_bytes).decode("utf-8")

client = OpenAI(api_key=Config.OPENAI_API_KEY)

# Simple test - just ask what text it sees
print("Asking AI: What text do you see in this image?\n")

response = client.chat.completions.create(
    model=Config.OPENAI_MODEL,
    messages=[
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": """Look at this image carefully.

I see colored square tiles on a dark background. Some tiles are green and contain text labels.

Please list ALL the text you can see on the green tiles. Just list the words, nothing else.

If you see text like "ALPHA", "BETA", "RED", "BLUE", etc., list them all."""
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": f"data:image/png;base64,{screenshot_base64}",
                        "detail": "high"
                    }
                }
            ]
        }
    ],
    max_tokens=300,
    temperature=0
)

result = response.choices[0].message.content
print("AI Response:")
print("=" * 60)
print(result)
print("=" * 60)
print()

# Now check what SHOULD be there
import json
with open("debug_output.json") as f:
    data = json.load(f)

actual_texts = [tile['text'] for tile in data['actual_tiles']]
print(f"\nActual text that SHOULD be visible: {', '.join(actual_texts)}")
print()

# Compare
ai_found_texts = result.upper().split()
matches = [text for text in actual_texts if text in ' '.join(ai_found_texts)]

if matches:
    print(f"✓ AI found these correct texts: {', '.join(matches)}")
else:
    print("✗ AI found NONE of the correct texts!")
    print()
    print("Possible issues:")
    print("  1. Text is too blurry or small in screenshot")
    print("  2. Rotation makes text unreadable")
    print("  3. Screenshot doesn't capture the text layer")
    print()
    print("Next step: Manually open screenshots/debug_screenshot.png")
    print("Can YOU read the text? If yes, increase SCREENSHOT_DELAY_MS")
    print("If no, the screenshot capture is broken.")
