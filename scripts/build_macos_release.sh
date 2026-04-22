#!/bin/bash
set -e

# ---------------------------------------
# Project-specific configuration
# ---------------------------------------
BINARY_NAME="verdocs"
# Automatically detect the project root relative to this script's location
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "[INFO] Starting macOS release build"
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

rustup target add aarch64-apple-darwin >/dev/null 2>&1 || true
rustup target add x86_64-apple-darwin >/dev/null 2>&1 || true

# ---------------------------------------
# Build for Apple Silicon
# ---------------------------------------
echo "[INFO] Building for aarch64-apple-darwin"

cargo build --release --target aarch64-apple-darwin

ARM_BIN="target/aarch64-apple-darwin/release/$BINARY_NAME"

if [ ! -f "$ARM_BIN" ]; then
  echo "[ERROR] Build failed: $ARM_BIN not found"
  exit 1
fi

cp "$ARM_BIN" "dist/${BINARY_NAME}-macos-arm64"

# ---------------------------------------
# Build for Intel macOS
# ---------------------------------------
echo "[INFO] Building for x86_64-apple-darwin"

cargo build --release --target x86_64-apple-darwin

INTEL_BIN="target/x86_64-apple-darwin/release/$BINARY_NAME"

if [ ! -f "$INTEL_BIN" ]; then
  echo "[ERROR] Build failed: $INTEL_BIN not found"
  exit 1
fi

cp "$INTEL_BIN" "dist/${BINARY_NAME}-macos-x86_64"

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

for file in ${BINARY_NAME}-macos-*; do
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
file dist/${BINARY_NAME}-macos-* | sed 's/^/  /'

echo ""
echo "[INFO] macOS release artifacts are ready for upload"
