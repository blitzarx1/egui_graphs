#!/bin/bash
set -e

echo "🎵 Starting Music Visualizer dev server..."

# Check for trunk
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk not found. Install with: cargo install trunk"
    exit 1
fi

# Serve with trunk
trunk serve --port 8087 --open
