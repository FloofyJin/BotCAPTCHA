"""OpenAI Vision API integration for analyzing challenges."""

from openai import OpenAI
from config import Config


class VisionAnalyzer:
    """Uses OpenAI Vision API to analyze challenge screenshots."""

    def __init__(self):
        self.client = OpenAI(api_key=Config.OPENAI_API_KEY)

    def analyze_challenge(self, base64_image):
        """
        Analyze a challenge screenshot and return bounding boxes for text tiles.

        Args:
            base64_image: Base64 encoded PNG image

        Returns:
            list: List of bounding boxes [{"x": float, "y": float, "width": float, "height": float}, ...]
        """
        prompt = self._create_prompt()

        try:
            response = self.client.chat.completions.create(
                model=Config.OPENAI_MODEL,
                messages=[
                    {
                        "role": "system",
                        "content": "You are an expert computer vision system analyzing a challenge. You must identify tiles with text and return precise bounding box coordinates in JSON format.",
                    },
                    {
                        "role": "user",
                        "content": [
                            {"type": "text", "text": prompt},
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": f"data:image/png;base64,{base64_image}",
                                    "detail": "high",
                                },
                            },
                        ],
                    },
                ],
                max_tokens=1000,
                temperature=0,  # Deterministic for consistency
            )

            # Parse response
            content = response.choices[0].message.content

            if Config.VERBOSE:
                print(f"OpenAI Response:\n{content}\n")

            # Extract JSON from response
            bounding_boxes = self._parse_bounding_boxes(content)

            return bounding_boxes

        except Exception as e:
            print(f"Error calling OpenAI API: {e}")
            raise

    def _create_prompt(self):
        """Create the prompt for the vision model."""
        return """You are analyzing a BotCaptcha challenge. The image shows an 800x600 pixel canvas with ROTATED rectangular tiles.

Your task:
1. Identify ALL tiles that contain text labels (like "ALPHA", "BETA", "RED", etc.)
2. For each tile with text, draw an AXIS-ALIGNED bounding box that FULLY CONTAINS the rotated rectangle
3. Return bounding boxes in JSON format

CRITICAL: The tiles are ROTATED at various angles. You must draw AXIS-ALIGNED (non-rotated) bounding boxes.
- If a tile is rotated 45°, the bounding box needs to be larger to contain it
- The bounding box should fully enclose the entire rotated rectangle
- Think of it as drawing the smallest upright rectangle that contains the tilted rectangle

Visual example:
- Rotated tile: ◇ (rotated square)
- Correct bounding box: □ (axis-aligned box that contains it, will be larger)
- Wrong: Trying to match the exact rotated shape

Tiles are approximately 60x60 pixels BEFORE rotation. After rotation, the axis-aligned bounding box may be 70-85 pixels to fully contain the diagonal.

Grid dimensions:
- Width: 800 pixels
- Height: 600 pixels
- Coordinate system: (0,0) is top-left corner

Return ONLY a JSON array of bounding boxes, nothing else:
[
  {"x": 100, "y": 200, "width": 80, "height": 80},
  {"x": 350, "y": 150, "width": 75, "height": 75}
]

Where:
- x, y: Top-left corner of the AXIS-ALIGNED bounding box
- width, height: Size to FULLY CONTAIN the rotated tile (usually 70-85 pixels)

Important:
- Only include tiles that have visible text
- Ignore tiles without text
- Be generous with box size to fully contain rotated tiles
- The box should completely enclose the colored rectangle
- Return valid JSON only, no markdown or extra text"""

    def _parse_bounding_boxes(self, response_text):
        """
        Parse bounding boxes from OpenAI response.

        Args:
            response_text: Response from OpenAI

        Returns:
            list: List of bounding box dictionaries
        """
        import json
        import re

        # Try to extract JSON from response
        # Handle markdown code blocks
        json_match = re.search(r"```(?:json)?\s*([\s\S]*?)\s*```", response_text)
        if json_match:
            json_str = json_match.group(1)
        else:
            # Try to find array directly
            json_match = re.search(r"\[[\s\S]*\]", response_text)
            if json_match:
                json_str = json_match.group(0)
            else:
                json_str = response_text.strip()

        try:
            bounding_boxes = json.loads(json_str)

            # Validate format
            if not isinstance(bounding_boxes, list):
                raise ValueError("Response is not a list")

            for box in bounding_boxes:
                if not all(key in box for key in ["x", "y", "width", "height"]):
                    raise ValueError(f"Invalid bounding box format: {box}")

                # Convert to float
                box["x"] = float(box["x"])
                box["y"] = float(box["y"])
                box["width"] = float(box["width"])
                box["height"] = float(box["height"])

            return bounding_boxes

        except json.JSONDecodeError as e:
            print(f"Failed to parse JSON from response: {e}")
            print(f"Response text: {response_text}")
            return []
        except ValueError as e:
            print(f"Invalid bounding box format: {e}")
            return []
