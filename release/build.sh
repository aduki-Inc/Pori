#!/bin/bash

# Build and release script for Pori
# This script builds the project, creates a git tag, and prepares for release

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Pori Build and Release Script${NC}"
echo "=================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Please run this script from the project root.${NC}"
    exit 1
fi

# Extract version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
TAG="v$VERSION"

echo -e "${YELLOW}Current version: $VERSION${NC}"
echo -e "${YELLOW}Git tag: $TAG${NC}"

# Check if tag already exists
if git rev-parse --verify "refs/tags/$TAG" >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: Tag $TAG already exists${NC}"
    read -p "Do you want to continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Parse command line arguments
SKIP_TESTS=false
SKIP_BUILD=false
SKIP_TAG=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-tag)
            SKIP_TAG=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --skip-tests    Skip running tests"
            echo "  --skip-build    Skip building the project"
            echo "  --skip-tag      Skip creating git tag"
            echo "  --dry-run       Show what would be done without executing"
            echo "  -h, --help      Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No actual changes will be made${NC}"
fi

# Check formatting
echo -e "${GREEN}Checking code formatting...${NC}"
if [ "$DRY_RUN" = false ]; then
    cargo fmt --all -- --check
fi

# Run clippy
echo -e "${GREEN}Running clippy...${NC}"
if [ "$DRY_RUN" = false ]; then
    cargo clippy --all-targets --all-features -- -D warnings
fi

# Run tests
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${GREEN}Running tests...${NC}"
    if [ "$DRY_RUN" = false ]; then
        cargo test --verbose
    fi
else
    echo -e "${YELLOW}Skipping tests${NC}"
fi

# Build release
if [ "$SKIP_BUILD" = false ]; then
    echo -e "${GREEN}Building release...${NC}"
    if [ "$DRY_RUN" = false ]; then
        cargo build --release --verbose
    fi
else
    echo -e "${YELLOW}Skipping build${NC}"
fi

# Test binary
if [ "$SKIP_BUILD" = false ] && [ "$DRY_RUN" = false ]; then
    echo -e "${GREEN}Testing binary...${NC}"
    ./target/release/pori --version
fi

# Create git tag
if [ "$SKIP_TAG" = false ]; then
    echo -e "${GREEN}Creating git tag...${NC}"
    if [ "$DRY_RUN" = false ]; then
        # Extract changelog for current version
        CHANGELOG_CONTENT=$(awk '/^## \['"$VERSION"'\]/{flag=1; next} /^## \[/{flag=0} flag' CHANGELOG.md)
        if [ -z "$CHANGELOG_CONTENT" ]; then
            CHANGELOG_CONTENT="Release $TAG"
        fi
        
        git tag -a "$TAG" -m "$CHANGELOG_CONTENT"
        echo -e "${GREEN}Created tag: $TAG${NC}"
        echo -e "${YELLOW}To push the tag, run: git push origin $TAG${NC}"
    else
        echo "Would create tag: $TAG"
    fi
else
    echo -e "${YELLOW}Skipping tag creation${NC}"
fi

echo -e "${GREEN}Build and release preparation completed!${NC}"
echo
echo "Next steps:"
echo "1. Push the tag: git push origin $TAG"
echo "2. The GitHub Actions will automatically build and create the release"
echo "3. Monitor the release at: https://github.com/aduki-Inc/Pori/actions"
