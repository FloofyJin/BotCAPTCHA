#!/bin/bash
# Setup script for BotCaptcha test agent

set -e

echo "==================================="
echo "BotCaptcha Test Agent Setup"
echo "==================================="
echo ""

# Check Python version
echo "Checking Python version..."
python3 --version || {
    echo "Error: Python 3 not found. Please install Python 3.8 or higher."
    exit 1
}

# Create virtual environment (optional but recommended)
echo ""
read -p "Create virtual environment? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
    source venv/bin/activate
    echo "Virtual environment activated"
fi

# Install Python dependencies
echo ""
echo "Installing Python dependencies..."
pip install -r requirements.txt

# Install Playwright browsers
echo ""
echo "Installing Playwright browsers..."
playwright install chromium

# Setup environment file
echo ""
if [ ! -f .env ]; then
    echo "Creating .env file from template..."
    cp .env.example .env
    echo ""
    echo "⚠️  IMPORTANT: Edit .env and add your OpenAI API key!"
    echo ""
else
    echo ".env file already exists"
fi

# Create output directories
echo "Creating output directories..."
mkdir -p screenshots
mkdir -p results

echo ""
echo "==================================="
echo "Setup Complete!"
echo "==================================="
echo ""
echo "Next steps:"
echo "1. Edit .env and add your OPENAI_API_KEY"
echo "2. Start BotCaptcha server: cd .. && cargo run"
echo "3. Run tests: python main.py"
echo ""
echo "For help: python main.py --help"
echo ""
