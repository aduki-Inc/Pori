# Release Management

This directory contains scripts and tools for managing releases of the tunnel-client.

## Files

### Installation Script

**`install.sh`** - Cross-platform installation script for end users

Downloads and installs the latest (or specified) release of tunnel-client from GitHub releases.

#### Usage

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/your-org/tunnel-client/main/release/install.sh | bash

# Install specific version
curl -fsSL https://raw.githubusercontent.com/your-org/tunnel-client/main/release/install.sh | bash -s -- --version v1.0.0

# Install to custom directory
curl -fsSL https://raw.githubusercontent.com/your-org/tunnel-client/main/release/install.sh | bash -s -- --dir ~/.local/bin

# Dry run to see what would be done
./install.sh --dry-run
```

#### Features

- Automatic platform and architecture detection
- Support for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and Windows
- Configurable installation directory
- Version verification
- Force reinstall option
- Comprehensive error handling

### Release Preparation Script

**`prepare.sh`** - Automated release preparation for maintainers

Prepares a new release by updating version numbers, running tests, building binaries, and creating git tags.

#### Usage

```bash
# Prepare version 1.0.0
./prepare.sh 1.0.0

# Dry run to see what would be done
./prepare.sh --dry-run 1.0.0
```

#### Process

1. Validates git working directory is clean
2. Updates version in `Cargo.toml`
3. Runs test suite
4. Builds release binary
5. Verifies binary version
6. Updates or creates `CHANGELOG.md`
7. Commits changes and creates git tag

## Release Process

### For Maintainers

1. **Prepare the release:**

   ```bash
   cd release
   ./prepare.sh 1.0.0
   ```

2. **Review and edit CHANGELOG.md** with actual changes

3. **Push changes and tag:**

   ```bash
   git push origin main
   git push origin v1.0.0
   ```

4. **Monitor GitHub Actions** - The release workflow will automatically:
   - Build binaries for all supported platforms
   - Create GitHub release with download links
   - Upload release artifacts
   - Generate checksums

### For Users

#### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/your-org/tunnel-client/main/release/install.sh | bash
```

#### Manual Installation

1. Go to [Releases](https://github.com/your-org/tunnel-client/releases)
2. Download the appropriate binary for your platform
3. Extract the archive
4. Move the binary to a directory in your PATH
5. Make it executable (Linux/macOS): `chmod +x tunnel-client`

#### Supported Platforms

| Platform | Architecture | Package |
|----------|--------------|---------|
| Linux | x86_64 | `tunnel-client-linux-x86_64.tar.gz` |
| Linux | x86_64 (static) | `tunnel-client-linux-x86_64-musl.tar.gz` |
| Linux | ARM64 | `tunnel-client-linux-aarch64.tar.gz` |
| macOS | Intel | `tunnel-client-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon | `tunnel-client-darwin-aarch64.tar.gz` |
| Windows | x86_64 | `tunnel-client-windows-x86_64.zip` |

## GitHub Actions Workflow

The release workflow (`.github/workflows/release.yml`) is triggered by:

- **Git tags** starting with `v` (e.g., `v1.0.0`)
- **Manual dispatch** through GitHub UI

### Workflow Steps

1. **Build Matrix** - Builds binaries for all supported platforms in parallel
2. **Cross-compilation** - Uses appropriate tools for each target
3. **Asset Preparation** - Creates compressed archives for distribution
4. **Release Creation** - Creates GitHub release with formatted description
5. **Checksum Generation** - Provides SHA256 checksums for verification
6. **Install Script Test** - Validates the installation script works

### Build Targets

- `x86_64-unknown-linux-gnu` - Standard Linux x86_64
- `x86_64-unknown-linux-musl` - Static Linux x86_64 (no glibc dependency)
- `aarch64-unknown-linux-gnu` - Linux ARM64
- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS Apple Silicon
- `x86_64-pc-windows-msvc` - Windows x86_64

## Security

### Verification

All releases include SHA256 checksums. Verify downloads:

```bash
# Download checksum file
curl -fsSL https://github.com/your-org/tunnel-client/releases/download/v1.0.0/SHA256SUMS -o SHA256SUMS

# Verify specific file
sha256sum -c SHA256SUMS --ignore-missing
```

### Installation Script Security

The installation script:

- Downloads only from official GitHub releases
- Verifies binary integrity where possible
- Uses secure HTTPS connections
- Provides dry-run mode for inspection
- Includes comprehensive error handling

## Troubleshooting

### Common Issues

## Permission Denied

- Ensure you have write permissions to the installation directory
- Use `sudo` for system directories: `curl ... | sudo bash`

## Platform Not Supported

- Check if your platform/architecture is in the supported list
- Consider building from source for unsupported platforms

## Network Issues

- Check internet connectivity
- Verify GitHub is accessible
- Try manual download if script fails

## Version Mismatch

- Use `--force` flag to reinstall
- Check if the version actually exists in releases

### Getting Help

- Check the main project README
- Review GitHub Issues for known problems
- Create a new issue with detailed error information
