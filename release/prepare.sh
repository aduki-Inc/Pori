#!/bin/bash

# Prepare script for tunnel-client releases

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
PACKAGE_NAME="tunnel-client"

show_help() {
    cat << EOF
Prepare Script for Tunnel Client Releases

USAGE:
    prepare.sh [OPTIONS] <VERSION>

OPTIONS:
    --dry-run           Show what would be done without making changes
    -h, --help          Show this help message

ARGUMENTS:
    VERSION             Version to release (e.g., 1.0.0, 2.1.0-rc1)

EXAMPLES:
    ./prepare.sh 1.0.0
    ./prepare.sh --dry-run 1.1.0

EOF
}

# Parse arguments
DRY_RUN=false
VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        -*)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            if [ -z "$VERSION" ]; then
                VERSION="$1"
            else
                print_error "Too many arguments"
                show_help
                exit 1
            fi
            shift
            ;;
    esac
done

if [ -z "$VERSION" ]; then
    print_error "Version is required"
    show_help
    exit 1
fi

# Validate version format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$'; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.0.0, 1.0.0-rc1)"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check if git is clean
check_git_status() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would check git status"
        return
    fi
    
    if ! git diff-index --quiet HEAD --; then
        print_error "Git working directory is not clean. Commit or stash changes first."
        exit 1
    fi
    
    if [ "$(git rev-parse --abbrev-ref HEAD)" != "main" ]; then
        print_warning "Not on main branch. Current branch: $(git rev-parse --abbrev-ref HEAD)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Update version in Cargo.toml
update_cargo_version() {
    local current_version
    current_version=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
    
    if [ "$DRY_RUN" = true ]; then
        print_info "Would update Cargo.toml version from $current_version to $VERSION"
        return
    fi
    
    print_info "Updating version in Cargo.toml: $current_version -> $VERSION"
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    rm Cargo.toml.bak
}

# Run tests
run_tests() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would run: cargo test"
        return
    fi
    
    print_info "Running tests..."
    if ! cargo test; then
        print_error "Tests failed. Fix issues before releasing."
        exit 1
    fi
}

# Build release binary
build_release() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would run: cargo build --release"
        return
    fi
    
    print_info "Building release binary..."
    if ! cargo build --release; then
        print_error "Release build failed"
        exit 1
    fi
}

# Check binary
check_binary() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would check binary version"
        return
    fi
    
    local binary_version
    binary_version=$(./target/release/$PACKAGE_NAME --version | cut -d' ' -f2)
    
    if [ "$binary_version" != "$VERSION" ]; then
        print_error "Binary version ($binary_version) doesn't match expected version ($VERSION)"
        exit 1
    fi
    
    print_success "Binary version check passed: $binary_version"
}

# Create changelog entry
update_changelog() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would create changelog entry for version $VERSION"
        return
    fi
    
    if [ ! -f "CHANGELOG.md" ]; then
        print_info "Creating CHANGELOG.md"
        cat > CHANGELOG.md << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [$VERSION] - $(date +%Y-%m-%d)

### Added
- Initial release

### Changed

### Deprecated

### Removed

### Fixed

### Security

EOF
    else
        # Add new version entry
        sed -i.bak "/## \[Unreleased\]/a\\
\\
## [$VERSION] - $(date +%Y-%m-%d)\\
\\
### Added\\
\\
### Changed\\
\\
### Deprecated\\
\\
### Removed\\
\\
### Fixed\\
\\
### Security\\
" CHANGELOG.md
        rm CHANGELOG.md.bak
    fi
    
    print_warning "Please update CHANGELOG.md with the changes for version $VERSION"
}

# Commit changes
commit_changes() {
    if [ "$DRY_RUN" = true ]; then
        print_info "Would commit changes and create tag v$VERSION"
        return
    fi
    
    print_info "Committing changes..."
    git add Cargo.toml CHANGELOG.md
    git commit -m "Prepare release $VERSION"
    
    print_info "Creating tag v$VERSION"
    git tag -a "v$VERSION" -m "Release $VERSION"
}

# Main execution
main() {
    print_info "Preparing release $VERSION for $PACKAGE_NAME"
    
    if [ "$DRY_RUN" = true ]; then
        print_warning "DRY RUN MODE - No changes will be made"
    fi
    
    echo
    
    check_git_status
    update_cargo_version
    run_tests
    build_release
    check_binary
    update_changelog
    commit_changes
    
    echo
    print_success "Release preparation completed!"
    
    if [ "$DRY_RUN" = false ]; then
        print_info "Next steps:"
        print_info "1. Review and edit CHANGELOG.md if needed"
        print_info "2. Push the changes: git push origin main"
        print_info "3. Push the tag to trigger release: git push origin v$VERSION"
        print_info "4. Monitor the GitHub Actions workflow"
    fi
}

main "$@"
