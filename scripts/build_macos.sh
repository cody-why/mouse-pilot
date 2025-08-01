#!/bin/bash

echo "Building for macOS..."

cargo build --release

# Create app bundle directory
target_app="$HOME/Downloads/release/MousePilot.app"
mkdir -p "$target_app/Contents/MacOS"
mkdir -p "$target_app/Contents/Resources"

target_dir=$(cargo metadata --format-version=1 | jq -r '.target_directory')
cp "$target_dir/release/mousepilot" "$target_app/Contents/MacOS/"
cp "assets/Info.plist" "$target_app/Contents/"
cp "assets/icon.icns" "$target_app/Contents/Resources/" 2>/dev/null || echo "Icon not found, run: chmod +x scripts/create_icon.sh && ./scripts/create_icon.sh"