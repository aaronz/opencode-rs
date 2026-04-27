#!/bin/bash
set -e

REPO_URL="https://github.com/anomalyco/opencode-rs"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/opencode"
BINARY_NAME="opencode-rs"

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Install opencode-rs to your local machine.

OPTIONS:
    -h, --help          Show this help message
    -d, --dir DIR       Install to custom directory (default: ~/.local/bin)
    -c, --config DIR    Config directory (default: ~/.config/opencode)
    -r, --repo URL      Git repository URL (default: $REPO_URL)
    -b, --branch BRANCH Git branch to install from
    --uninstall         Remove opencode-rs from your system
    --skip-build        Skip building (use existing binary)

EXAMPLES:
    $0                          # Interactive install
    $0 -d ~/.local/bin          # Install to custom directory
    $0 --uninstall              # Remove installation

EOF
}

UNINSTALL=false
SKIP_BUILD=false
CUSTOM_DIR=""
CUSTOM_CONFIG=""
BRANCH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -d|--dir)
            CUSTOM_DIR="$2"
            shift 2
            ;;
        -c|--config)
            CUSTOM_CONFIG="$2"
            shift 2
            ;;
        -r|--repo)
            REPO_URL="$2"
            shift 2
            ;;
        -b|--branch)
            BRANCH="$2"
            shift 2
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if [[ -n "$CUSTOM_DIR" ]]; then
    INSTALL_DIR="$CUSTOM_DIR"
fi

if [[ -n "$CUSTOM_CONFIG" ]]; then
    CONFIG_DIR="$CUSTOM_CONFIG"
fi

uninstall() {
    echo "Removing opencode-rs..."

    if [[ -f "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        echo "  Removed ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    if [[ -d "$CONFIG_DIR" ]]; then
        rm -rf "$CONFIG_DIR"
        echo "  Removed $CONFIG_DIR"
    fi

    echo ""
    echo "opencode-rs has been uninstalled."
    exit 0
}

if [[ "$UNINSTALL" == true ]]; then
    uninstall
fi

echo "========================================"
echo "  opencode-rs Installer"
echo "========================================"
echo ""
echo "Install directory: $INSTALL_DIR"
echo "Config directory:  $CONFIG_DIR"
echo ""

if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/opencode-rust" && pwd)"

if [[ "$SKIP_BUILD" == false ]]; then
    echo "Building opencode-rs (release mode)..."
    echo "This may take a few minutes on first run."
    echo ""

    cd "$PROJECT_DIR"

    if [[ -n "$BRANCH" ]]; then
        git checkout "$BRANCH" 2>/dev/null || true
    fi

    cargo build --release

    BINARY_PATH="$PROJECT_DIR/target/release/${BINARY_NAME}"
else
    BINARY_PATH="$PROJECT_DIR/target/release/${BINARY_NAME}"
    if [[ ! -f "$BINARY_PATH" ]]; then
        echo "Error: Binary not found at $BINARY_PATH"
        echo "Run without --skip-build to build first."
        exit 1
    fi
fi

echo ""
echo "Installing to $INSTALL_DIR..."

mkdir -p "$INSTALL_DIR"

if cp "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"; then
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    echo "  Copied binary to ${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Error: Failed to copy binary."
    echo "You may need to create the directory first:"
    echo "  mkdir -p $INSTALL_DIR"
    exit 1
fi

echo ""
echo "Creating config directory..."
mkdir -p "$CONFIG_DIR"

if [[ ! -f "${CONFIG_DIR}/config.toml" ]]; then
    cat > "${CONFIG_DIR}/config.toml" <<EOF
# OpenCode RS Configuration
# https://github.com/anomalyco/opencode-rs

[general]
# Verbose logging
verbose = false

[llm]
# Default LLM provider
provider = "anthropic"

[server]
# Desktop server configuration
enabled = true
port = 3000
hostname = "127.0.0.1"

[server.desktop]
enabled = true
auto_open_browser = true

[server.acp]
enabled = true
server_id = "local"
version = "1.0"
EOF
    echo "  Created default config at ${CONFIG_DIR}/config.toml"
else
    echo "  Config already exists at ${CONFIG_DIR}/config.toml"
fi

echo ""
echo "========================================"
echo "  Installation Complete!"
echo "========================================"
echo ""
echo "Binary installed: ${INSTALL_DIR}/${BINARY_NAME}"
echo "Config located:   ${CONFIG_DIR}/config.toml"
echo ""

if [[ ":$PATH:" == *":${INSTALL_DIR}:"* ]]; then
    echo "The install directory is in your PATH."
else
    echo "IMPORTANT: Add the install directory to your PATH:"
    echo ""
    echo "  # Add to ~/.zshrc or ~/.bashrc:"
    echo "  export PATH=\"\${HOME}/.local/bin:\$PATH\""
    echo ""
fi

echo "Run 'opencode-rs --help' to get started."