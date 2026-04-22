#!/bin/bash
set -e

# ---------------------------------------
# Project-specific configuration
# ---------------------------------------
BINARY_NAME="verdocs"
# Automatically detect the project root relative to this script's location
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "[INFO] Starting Linux release build"
echo "[INFO] Binary: $BINARY_NAME"
echo "[INFO] Project directory: $PROJECT_DIR"

# ---------------------------------------
# Validate project directory
# ---------------------------------------
if [ ! -d "$PROJECT_DIR" ]; then
  echo "[ERROR] Project directory not found: $PROJECT_DIR"
  exit 1
fi

cd "$PROJECT_DIR"

if [ ! -f "Cargo.toml" ]; then
  echo "[ERROR] Cargo.toml not found in $PROJECT_DIR"
  exit 1
fi

# ---------------------------------------
# Prepare dist directory
# ---------------------------------------
echo "[INFO] Cleaning dist directory"
rm -rf dist
mkdir -p dist

# ---------------------------------------
# Ensure required targets exist
# ---------------------------------------
echo "[INFO] Ensuring Rust targets are installed"

rustup target add x86_64-unknown-linux-gnu >/dev/null 2>&1 || true

# ---------------------------------------
# Build for x86_64 Linux
# ---------------------------------------
echo "[INFO] Building for x86_64-unknown-linux-gnu"

cargo build --release --target x86_64-unknown-linux-gnu

X86_BIN="target/x86_64-unknown-linux-gnu/release/$BINARY_NAME"

if [ ! -f "$X86_BIN" ]; then
  echo "[ERROR] Build failed: $X86_BIN not found"
  exit 1
fi

cp "$X86_BIN" "dist/${BINARY_NAME}-linux-x86_64"

# ---------------------------------------
# Make binaries executable
# ---------------------------------------
chmod +x dist/*

# ---------------------------------------
# Strip binaries (optional)
# ---------------------------------------
echo "[INFO] Stripping binaries (if supported)"
strip dist/* 2>/dev/null || true

# ---------------------------------------
# Create tar.gz archives
# ---------------------------------------
echo "[INFO] Creating archives"

cd dist

for file in ${BINARY_NAME}-linux-*; do
  tar -czf "${file}.tar.gz" "$file"
done

cd ..

# ---------------------------------------
# Verify outputs
# ---------------------------------------
echo "[INFO] Build complete"
echo "[INFO] Generated artifacts:"

ls -lh dist/

echo ""
echo "[INFO] Architecture verification:"
file dist/${BINARY_NAME}-linux-* | sed 's/^/  /'

echo ""
echo "[INFO] Linux release artifacts are ready for upload"
