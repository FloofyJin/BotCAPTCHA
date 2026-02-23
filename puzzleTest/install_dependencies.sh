#!/bin/bash
# Install system dependencies for Playwright on Ubuntu/Debian/WSL

echo "Installing Playwright system dependencies..."
echo ""

# Check if running on Ubuntu/Debian
if ! command -v apt-get &> /dev/null; then
    echo "This script is for Ubuntu/Debian-based systems."
    echo "Please install dependencies manually for your distribution."
    exit 1
fi

# Install required libraries for Chromium
echo "Installing required libraries..."
sudo apt-get update
sudo apt-get install -y \
    libnss3 \
    libnspr4 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libdbus-1-3 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libpango-1.0-0 \
    libcairo2 \
    libasound2 \
    libatspi2.0-0 \
    libxshmfence1

echo ""
echo "✓ Dependencies installed!"
echo ""
echo "Now run: playwright install chromium"
echo "Then try: python quick_test.py"
