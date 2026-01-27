#!/bin/bash
set -e

# CodeScope MCP Setup Script
# Downloads pre-built binary from GitHub Releases if not present

REPO="codescope-mcp/codescope-mcp"
BINARY_NAME="codescope-mcp"
VERSION="v0.1.1"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_ROOT="$(dirname "$SCRIPT_DIR")"
BIN_DIR="$PLUGIN_ROOT/bin"
BINARY_PATH="$BIN_DIR/$BINARY_NAME"

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin)
            os="apple-darwin"
            ;;
        Linux)
            os="unknown-linux-gnu"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="pc-windows-msvc"
            ;;
        *)
            echo "Unsupported OS: $(uname -s)" >&2
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        *)
            echo "Unsupported architecture: $(uname -m)" >&2
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

# Download binary from GitHub Releases
download_binary() {
    local platform="$1"
    local extension="tar.gz"

    if [[ "$platform" == *"windows"* ]]; then
        extension="zip"
    fi

    local url="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME-$platform.$extension"
    local temp_dir=$(mktemp -d)
    local archive_path="$temp_dir/$BINARY_NAME.$extension"

    echo "Downloading $BINARY_NAME for $platform..."
    echo "URL: $url"

    if command -v curl &> /dev/null; then
        curl -fsSL -o "$archive_path" "$url"
    elif command -v wget &> /dev/null; then
        wget -q -O "$archive_path" "$url"
    else
        echo "Error: Neither curl nor wget is available" >&2
        exit 1
    fi

    # Create bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"

    # Extract binary
    echo "Extracting..."
    if [[ "$extension" == "tar.gz" ]]; then
        tar -xzf "$archive_path" -C "$BIN_DIR"
    else
        unzip -q "$archive_path" -d "$BIN_DIR"
    fi

    # Cleanup
    rm -rf "$temp_dir"

    # Set executable permission
    chmod +x "$BINARY_PATH"

    echo "Successfully installed $BINARY_NAME to $BINARY_PATH"
}

main() {
    # Check if binary already exists and is executable
    if [[ -x "$BINARY_PATH" ]]; then
        echo "$BINARY_NAME is already installed at $BINARY_PATH"
        exit 0
    fi

    local platform=$(detect_platform)
    echo "Detected platform: $platform"

    download_binary "$platform"
}

main "$@"
