#!/bin/bash
set -e

# CONSTANTS
# Analytics setup (early check for new install)
IS_NEW_INSTALL=false
if ! command -v verdocs >/dev/null 2>&1; then
    IS_NEW_INSTALL=true
fi

REPO="Abhijeet-Gautam5702/verdocs"
BINARY_NAME="verdocs"

# DETECT USER OS & ARCHITECTURE
ARCH=$(uname -m)
OS=$(uname -s)

if [[ "$OS" == "Darwin" ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        FILE="verdocs-macos-arm64.tar.gz"
    else
        FILE="verdocs-macos-x86_64.tar.gz"
    fi
elif [[ "$OS" == "Linux" ]]; then
    if [[ "$ARCH" == "x86_64" ]]; then
        FILE="verdocs-linux-x86_64.tar.gz"
    else
        echo "[ERROR] Unsupported Linux Architecture: $ARCH"
        exit 1
    fi
else
    echo "[ERROR] Unsupported OS: $OS"
    exit 1
fi

# DOWNLOAD THE LATEST BINARY FROM GITHUB RELEASES
LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')
echo "Installing verdocs v$LATEST_VERSION"

# Release URL (LATEST VERSION)
URL="https://github.com/$REPO/releases/latest/download/$FILE"

echo "Fetching from $URL"
echo "Downloading $FILE..."

curl -L "$URL" -o "$FILE" || {
 echo "[ERROR] Download failed"
 exit 1
}

echo "Extracting $FILE ...."
tar -xzf "$FILE" || {
 echo "[ERROR] Extraction failed"
 exit 1
}
EXTRACTED_FILE="${FILE%.tar.gz}"
chmod +x "$EXTRACTED_FILE"
mv "$EXTRACTED_FILE" "$BINARY_NAME"
echo "Cleanup: Removing $FILE"
rm "$FILE"

# INSTALL THE BINARY INTO THE USER SYSTEM
# IF NOT ABLE TO INSTALL GLOBALLY (usr/local/bin) => INSTALL IN LOCAL USER BIN ($HOME/.local/bin)
# Install location
INSTALL_DIR="/usr/local/bin"

# Write permission denied to /usr/local/bin -> Install to local bin of the user
if [[ ! -w "$INSTALL_DIR" ]]; then
  echo "[WARN] No write access to /usr/local/bin"
  echo "Installing to ~/.local/bin instead..."
  echo "To install globally, re-run with 'sudo'"
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

# OVERWRITE THE BINARY IF ALREADY PRESENT IN THE INSTALL LOCATION
if [[ -f "$INSTALL_DIR/$BINARY_NAME" ]]; then
  echo "Existing installation found at $INSTALL_DIR/$BINARY_NAME"
  echo "Overwriting the existing binary..."
fi
mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo "Installed to $INSTALL_DIR/$BINARY_NAME"

# VERIFY INSTALLATION
echo ""
echo "Verifying installation..."

INSTALLATION_SUCCESS=false
if command -v verdocs >/dev/null 2>&1; then
  echo "verdocs v$LATEST_VERSION Installation successful!"
  echo "Run: verdocs --help"
  INSTALLATION_SUCCESS=true
else
# IF PATH NOT FOUND => DETECT SHELL (BASH/ZSH) AND PROVIDE HELP SETTING PATH
  echo "[ERROR] 'verdocs' is not in your PATH"

  # Detect shell
  SHELL_NAME=$(basename "$SHELL")

  echo ""
  echo "👉 To fix this, add ~/.local/bin to your PATH"

  if [[ "$SHELL_NAME" == "zsh" ]]; then
    echo ""
    echo "For zsh (default on macOS):"
    echo "  nano ~/.zshrc"
    echo ""
    echo "Add this line:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    echo ""
    echo "Then run:"
    echo "  source ~/.zshrc"

  elif [[ "$SHELL_NAME" == "bash" ]]; then
    echo ""
    echo "For bash:"
    echo "  nano ~/.bashrc"
    echo ""
    echo "Add this line:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    echo ""
    echo "Then run:"
    echo "  source ~/.bashrc"

  else
    echo ""
    echo "Unknown shell: $SHELL_NAME"
    echo "Add this line to your shell config file:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
  fi

  echo ""
  echo "After updating PATH, restart your terminal or source your config file."
fi

# POSTHOG ANALYTICS (UNIQUE DOWNLOADS)
# --- POSTHOG ANALYTICS ---
POSTHOG_PROJECT_KEY="phc_BhYzF5XGyASH4h5nVLkhuxPMSW7DUFoyz4mgtQ59Qr3P"
POSTHOG_API_HOST="https://eu.i.posthog.com"

if [[ "$INSTALLATION_SUCCESS" == "true" && "$IS_NEW_INSTALL" == "true" && -z "$DO_NOT_TRACK" ]]; then
    (
        VERDOCS_DIR="$HOME/.verdocs"
        mkdir -p "$VERDOCS_DIR"
        UID_FILE="$VERDOCS_DIR/.uid"

        if [ ! -f "$UID_FILE" ]; then
            if command -v uuidgen >/dev/null 2>&1; then
                uuidgen > "$UID_FILE"
            else
                LC_ALL=C tr -dc 'a-zA-Z0-9' < /dev/urandom | fold -w 32 | head -n 1 > "$UID_FILE"
            fi
        fi

        USER_ID=$(cat "$UID_FILE")

        curl -s -X POST "$POSTHOG_API_HOST/capture/" \
            -H "Content-Type: application/json" \
            -d "{
                \"api_key\": \"$POSTHOG_PROJECT_KEY\",
                \"event\": \"verdocs_install_completed\",
                \"distinct_id\": \"$USER_ID\",
                \"properties\": {
                    \"os\": \"$OS\",
                    \"arch\": \"$ARCH\",
                    \"version\": \"$LATEST_VERSION\",
                    \"binary_name\": \"$BINARY_NAME\",
                    \"source\": \"manual_script\"
                }
            }" > /dev/null 2>&1
    ) &
fi
