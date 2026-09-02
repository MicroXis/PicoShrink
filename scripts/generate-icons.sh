#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

SOURCE="${PROJECT_DIR}/assets/icon.svg"
OUTPUT_DIR="${PROJECT_DIR}/assets"

SIZES=(16 24 32 48 64 128 256 512 1024)

clean() {
    echo "Cleaning generated icons..."

    rm -f "${OUTPUT_DIR}"/icon-*.png
    rm -f "${OUTPUT_DIR}/icon.ico"
    rm -f "${OUTPUT_DIR}/icon.icns"
}

if [[ "${1:-}" == "clean" ]]; then
    clean
    exit 0
fi

command -v inkscape >/dev/null 2>&1 || {
    echo "Error: inkscape is not installed."
    exit 1
}

command -v magick >/dev/null 2>&1 || {
    echo "Error: ImageMagick is not installed."
    exit 1
}

clean

for size in "${SIZES[@]}"; do
    echo "Generating ${size}x${size}..."

    inkscape "$SOURCE" \
        --export-filename="${OUTPUT_DIR}/icon-${size}.png" \
        --export-width="$size" \
        --export-height="$size"
done

echo "Generating Windows ICO..."

magick \
    "${OUTPUT_DIR}/icon-16.png" \
    "${OUTPUT_DIR}/icon-24.png" \
    "${OUTPUT_DIR}/icon-32.png" \
    "${OUTPUT_DIR}/icon-48.png" \
    "${OUTPUT_DIR}/icon-64.png" \
    "${OUTPUT_DIR}/icon-128.png" \
    "${OUTPUT_DIR}/icon-256.png" \
    "${OUTPUT_DIR}/icon.ico"

echo "Icons generated successfully."
