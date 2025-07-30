#!/bin/bash

# This script converts SVG to .icns format

ICON_DIR="assets"
ICON_NAME="icon"

# Check if ImageMagick is installed
if ! command -v magick &> /dev/null; then
    echo "ImageMagick not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install imagemagick
    else
        echo "Please install ImageMagick manually: https://imagemagick.org/"
        exit 1
    fi
fi

# Create icon directory if it doesn't exist
mkdir -p "$ICON_DIR"

# Convert SVG to different sizes for .icns
echo "Converting SVG to PNG files..."

# 裁剪圆角边缘留空
magick "$ICON_DIR/${ICON_NAME}.svg" \
  \( -size 512x512 xc:none -draw "roundrectangle 60,60,452,452,100,100" \) \
  -alpha set -compose DstIn -composite "$ICON_DIR/icon_s.png"

# Generate different sizes
magick $ICON_DIR/icon_s.png -resize 16x16 "$ICON_DIR/icon_16x16.png"
magick $ICON_DIR/icon_s.png -resize 32x32 "$ICON_DIR/icon_32x32.png"
magick $ICON_DIR/icon_s.png -resize 48x48 "$ICON_DIR/icon_48x48.png"
magick $ICON_DIR/icon_s.png -resize 64x64 "$ICON_DIR/icon_64x64.png"
magick $ICON_DIR/icon_s.png -resize 128x128 "$ICON_DIR/icon_128x128.png"
magick $ICON_DIR/icon_s.png -resize 256x256 "$ICON_DIR/icon_256x256.png"

# if [[ "$OSTYPE" == "darwin"* ]]; then
echo "Creating .icns file..."

# Create iconset directory
mkdir -p "$ICON_DIR/${ICON_NAME}.iconset"

# Copy PNG files to iconset with proper naming
cp "$ICON_DIR/icon_16x16.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_16x16.png"
cp "$ICON_DIR/icon_32x32.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_16x16@2x.png"
cp "$ICON_DIR/icon_32x32.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_32x32.png"
cp "$ICON_DIR/icon_64x64.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_32x32@2x.png"
cp "$ICON_DIR/icon_128x128.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_128x128.png"
cp "$ICON_DIR/icon_256x256.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_128x128@2x.png"
cp "$ICON_DIR/icon_256x256.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_256x256.png"
cp "$ICON_DIR/icon_256x256.png" "$ICON_DIR/${ICON_NAME}.iconset/icon_256x256@2x.png"

# Generate .icns file
iconutil -c icns "$ICON_DIR/${ICON_NAME}.iconset" -o "$ICON_DIR/${ICON_NAME}.icns"

# Clean up iconset directory
rm -rf "$ICON_DIR/${ICON_NAME}.iconset"

echo "Icon created: $ICON_DIR/${ICON_NAME}.icns"

# Create Windows .ico file
echo "Creating Windows .ico file..."
magick "$ICON_DIR/icon_16x16.png" "$ICON_DIR/icon_32x32.png" "$ICON_DIR/icon_48x48.png" "$ICON_DIR/icon_64x64.png" "$ICON_DIR/icon_128x128.png" "$ICON_DIR/icon_256x256.png" "$ICON_DIR/icon.ico"

echo "Windows icon created: $ICON_DIR/icon.ico" 

cp "$ICON_DIR/icon_256x256.png" "$ICON_DIR/icon.png"

# Clean up temporary PNG files
rm -f "$ICON_DIR"/icon_*.png

# 转换icon为 Rust 代码
cargo test --test convert_icon

echo "Icon creation complete!"