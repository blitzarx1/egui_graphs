#!/bin/bash
set -e

echo "🎵 Building Music Visualizer..."

# Check for trunk
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk not found. Install with: cargo install trunk"
    exit 1
fi

# Build in release mode
trunk build --release

echo "✅ Build complete! Output in ./dist"
