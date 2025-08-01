#!/bin/bash

# Windows build script

set -e

echo "Building for Windows..."

# Create assets directory if it doesn't exist
mkdir -p assets

# Generate icons if they don't exist
if [ ! -f "assets/icon.ico" ]; then
    echo "Generating icons..."
    chmod +x scripts/create_icon.sh
    ./scripts/create_icon.sh
fi

# Generate resource file
if [ ! -f "assets/icon.rc" ]; then
    echo "Generating resource file..."
    cat > assets/icon.rc << EOF
APP_ICON ICON "icon.ico"
EOF
fi

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

mkdir -p ~/Downloads/release
target_dir=$(cargo metadata --format-version=1 | jq -r '.target_directory')
cp $target_dir/x86_64-pc-windows-gnu/release/mousepilot.exe ~/Downloads/release/

echo "Windows build complete"
