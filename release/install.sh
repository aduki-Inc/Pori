#!/bin/bash

# Tunnel Client Installation Script
# This script downloads and installs the latest release of tunnel-client

set -e

# Configuration
REPO="your-org/tunnel-client"
BINARY_NAME="tunnel-client"
INSTALL_DIR="/usr/local/bin"
TMP_DIR="/tmp/tunnel-client-install"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Tunnel Client Installation Script

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    -v, --version VERSION    Install specific version (default: latest)
    -d, --dir DIRECTORY      Installation directory (default: /usr/local/bin)
    -f, --force              Force installation even if already exists
    --dry-run                Show what would be done without installing
    -h, --help               Show this help message

EXAMPLES:
    # Install latest version
    ./install.sh

    # Install specific version
    ./install.sh --version v1.0.0

    # Install to custom directory
    ./install.sh --dir ~/.local/bin

    # Force reinstall
    ./install.sh --force

REQUIREMENTS:
    - curl or wget
    - tar (for Linux/macOS) or unzip (for Windows)
    - sudo privileges (if installing to system directory)

EOF
}

# Parse command line arguments
VERSION=""
FORCE=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Detect platform and architecture
detect_platform() {
    local os arch
    
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)
    
    case "$os" in
        linux*)
            case "$arch" in
                x86_64|amd64)
                    echo "linux-x86_64"
                    ;;
                aarch64|arm64)
                    echo "linux-aarch64"
                    ;;
                *)
                    print_error "Unsupported architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        darwin*)
            case "$arch" in
                x86_64)
                    echo "darwin-x86_64"
                    ;;
                arm64)
                    echo "darwin-aarch64"
                    ;;
                *)
                    print_error "Unsupported architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file
download_file() {
    local url="$1"
    local output="$2"
    
    if command_exists curl; then
        curl -fsSL "$url" -o "$output"
    elif command_exists wget; then
        wget -q "$url" -O "$output"
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
}

# Get latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local version
    
    if command_exists curl; then
        version=$(curl -fsSL "$api_url" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    elif command_exists wget; then
        version=$(wget -qO- "$api_url" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    else
        print_error "Cannot fetch latest version. Neither curl nor wget is available."
        exit 1
    fi
    
    if [ -z "$version" ]; then
        print_error "Failed to get latest version"
        exit 1
    fi
    
    echo "$version"
}

# Check if binary is already installed
check_existing() {
    if command_exists "$BINARY_NAME" && [ "$FORCE" = false ]; then
        local existing_version
        existing_version=$($BINARY_NAME --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")
        print_warning "$BINARY_NAME is already installed (version: $existing_version)"
        print_info "Use --force to reinstall or --help for more options"
        exit 0
    fi
}

# Create installation directory
create_install_dir() {
    if [ ! -d "$INSTALL_DIR" ]; then
        if [ "$DRY_RUN" = true ]; then
            print_info "Would create directory: $INSTALL_DIR"
            return
        fi
        
        print_info "Creating installation directory: $INSTALL_DIR"
        if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            print_info "Trying with sudo..."
            sudo mkdir -p "$INSTALL_DIR"
        fi
    fi
}

# Install binary
install_binary() {
    local platform version download_url archive_name extract_dir
    
    platform=$(detect_platform)
    
    if [ -z "$VERSION" ]; then
        VERSION=$(get_latest_version)
    fi
    
    print_info "Installing tunnel-client $VERSION for $platform"
    
    if [ "$DRY_RUN" = true ]; then
        print_info "Would download from: https://github.com/$REPO/releases/download/$VERSION/tunnel-client-$platform.tar.gz"
        print_info "Would install to: $INSTALL_DIR/$BINARY_NAME"
        return
    fi
    
    # Setup temporary directory
    rm -rf "$TMP_DIR"
    mkdir -p "$TMP_DIR"
    cd "$TMP_DIR"
    
    # Download archive
    archive_name="tunnel-client-$platform.tar.gz"
    download_url="https://github.com/$REPO/releases/download/$VERSION/$archive_name"
    
    print_info "Downloading $archive_name..."
    if ! download_file "$download_url" "$archive_name"; then
        print_error "Failed to download $download_url"
        exit 1
    fi
    
    # Extract archive
    print_info "Extracting archive..."
    tar -xzf "$archive_name"
    
    if [ ! -f "$BINARY_NAME" ]; then
        print_error "Binary not found in archive"
        exit 1
    fi
    
    # Make binary executable
    chmod +x "$BINARY_NAME"
    
    # Install binary
    print_info "Installing to $INSTALL_DIR/$BINARY_NAME"
    if ! cp "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
        print_info "Trying with sudo..."
        sudo cp "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    # Cleanup
    cd /
    rm -rf "$TMP_DIR"
    
    print_success "tunnel-client $VERSION installed successfully!"
}

# Verify installation
verify_installation() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would verify installation by running: $BINARY_NAME --version"
        return
    fi
    
    if command_exists "$BINARY_NAME"; then
        local installed_version
        installed_version=$($BINARY_NAME --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")
        print_success "Verification successful! Installed version: $installed_version"
        
        # Check if binary is in PATH
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            print_warning "$INSTALL_DIR is not in your PATH"
            print_info "Add this to your shell profile (.bashrc, .zshrc, etc.):"
            print_info "    export PATH=\"$INSTALL_DIR:\$PATH\""
        fi
    else
        print_error "Installation verification failed. Binary not found in PATH."
        exit 1
    fi
}

# Show usage information
show_usage() {
    cat << EOF

USAGE:
    tunnel-client --url <WEBSOCKET_URL> --token <AUTH_TOKEN>

EXAMPLES:
    # Basic usage
    tunnel-client --url wss://proxy.example.com --token your-token

    # With custom local server
    tunnel-client --url wss://proxy.example.com --token your-token --local http://localhost:8080

    # With configuration file
    tunnel-client --config /path/to/config.toml

For more options, run: tunnel-client --help

EOF
}

# Main execution
main() {
    print_info "Tunnel Client Installation Script"
    print_info "Repository: https://github.com/$REPO"
    echo
    
    if [ "$DRY_RUN" = true ]; then
        print_info "DRY RUN MODE - No changes will be made"
        echo
    fi
    
    check_existing
    create_install_dir
    install_binary
    verify_installation
    
    if [ "$DRY_RUN" = false ]; then
        echo
        show_usage
    fi
}

# Run main function
main "$@"
