"""Browser automation and screenshot capture."""

import time
import base64
from pathlib import Path
from playwright.sync_api import sync_playwright, Page
from config import Config


class BrowserController:
    """Controls browser for challenge capture."""

    def __init__(self, headless=False):
        self.headless = headless
        self.playwright = None
        self.browser = None
        self.page = None

        # Create screenshot directory
        Path(Config.SCREENSHOT_DIR).mkdir(exist_ok=True)

    def __enter__(self):
        """Context manager entry."""
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()

    def start(self):
        """Start browser."""
        self.playwright = sync_playwright().start()
        self.browser = self.playwright.chromium.launch(headless=self.headless)
        self.page = self.browser.new_page(viewport={"width": 1024, "height": 768})

    def close(self):
        """Close browser."""
        if self.page:
            self.page.close()
        if self.browser:
            self.browser.close()
        if self.playwright:
            self.playwright.stop()

    def load_challenge(self):
        """Load BotCaptcha page and start a challenge."""
        print(f"Loading {Config.BOTCAPTCHA_URL}...")
        self.page.goto(Config.BOTCAPTCHA_URL)

        start_btn = self.page.locator("#startBtn")
        start_btn.wait_for(state="visible", timeout=5000)

        print("Starting challenge...")
        start_btn.click()

        # wait for challenge data
        self.page.wait_for_function(
            "() => window.BotCaptchaAPI?.getCurrentChallenge?.() !== null",
            timeout=10000
        )

        # ⭐ NEW — wait for first rendered frame ⭐
        self.page.wait_for_function(
            "() => window.BotCaptchaAPI?.isFrameReady?.() === true",
            timeout=10000
        )

        print("Challenge data is ready")

        # Wait for canvas to be rendered
        self.page.wait_for_selector("#glCanvas", timeout=5000)

        # Give extra time for WebGL and text to render
        time.sleep(Config.SCREENSHOT_DELAY_MS / 1000)

    def capture_canvas_screenshot(self, filepath=None):
        """
        Capture screenshot of the canvas area.
        Composites both WebGL and text canvases into a single image.

        Args:
            filepath: Optional path to save screenshot

        Returns:
            bytes: Screenshot as bytes
        """
        # Create a composite canvas using JavaScript
        # This merges the WebGL canvas and text canvas into one image
        screenshot_data_url = self.page.evaluate("""
            () => {
                // Create a temporary canvas for compositing
                const composite = document.createElement('canvas');
                composite.width = 800;
                composite.height = 600;
                const ctx = composite.getContext('2d');

                // Draw WebGL canvas first (background)
                const glCanvas = document.getElementById('glCanvas');
                ctx.drawImage(glCanvas, 0, 0);

                // Draw text canvas on top
                const textCanvas = document.getElementById('textCanvas');
                ctx.drawImage(textCanvas, 0, 0);

                // Return as data URL
                return composite.toDataURL('image/png');
            }
        """)

        # Convert data URL to bytes
        import re
        # Remove data:image/png;base64, prefix
        base64_data = re.sub('^data:image/.+;base64,', '', screenshot_data_url)
        screenshot_bytes = base64.b64decode(base64_data)

        # Save to file if path provided
        if filepath:
            with open(filepath, "wb") as f:
                f.write(screenshot_bytes)
            print(f"Screenshot saved to {filepath}")

        return screenshot_bytes

    def capture_canvas_base64(self):
        """
        Capture canvas screenshot as base64 string for OpenAI API.

        Returns:
            str: Base64 encoded PNG image
        """
        screenshot_bytes = self.capture_canvas_screenshot()
        return base64.b64encode(screenshot_bytes).decode("utf-8")

    def get_challenge_data(self):
        """
        Get challenge data from the page via JavaScript.

        Returns:
            dict: Challenge data including challenge_id
        """
        challenge_data = self.page.evaluate("""
        () => {
            const ch = window.BotCaptchaAPI?.getCurrentChallenge?.();
            if (ch) {
                return {
                    challenge_id: ch.challenge_id,
                    duration_ms: ch.duration_ms,
                    num_tiles: ch.tiles.length,
                    grid_width: ch.grid_size.width,
                    grid_height: ch.grid_size.height
                };
            }
            return null;
        }
        """)

        if not challenge_data:
            raise ValueError("Challenge data not available")

        return challenge_data

    def submit_answer(self, bounding_boxes):
        """
        Submit answer by calling the JavaScript API.

        Args:
            bounding_boxes: List of dicts with x, y, width, height

        Returns:
            dict: Result from submission
        """
        # Convert bounding boxes to JSON string
        import json

        boxes_json = json.dumps(bounding_boxes)

        # Submit via JavaScript
        result = self.page.evaluate(
            """
            async (boxes) => {
                const ch = window.BotCaptchaAPI?.getCurrentChallenge?.();

                if (!ch) {
                    return { success: false, message: "No active challenge" };
                }

                const response = await fetch('/api/submit', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        challenge_id: ch.challenge_id,
                        answers: boxes
                    })
                });

                return await response.json();
            }
            """,
            bounding_boxes,
        )

        return result
