# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-07-01

### Added

- **Enhanced Logging System**
  - Custom log levels: `LOCAL` for HTTP server operations, `PROXY` for WebSocket/proxy operations
  - Human-readable timestamp format (YYYY-MM-DD HH:MM:SS.mmm) without timezone
  - Color-coded log levels: Magenta for PROXY, Cyan for LOCAL
  - Custom `local_log!()` and `proxy_log!()` macros for categorized logging
  - Professional logging output without emojis

- **WebSocket Token Authentication**
  - Token-based authentication via query parameters (`?token=<token>`)
  - Automatic token appending to WebSocket URLs from configuration
  - Improved WebSocket message handling for authentication responses
  - Better error handling for non-tunnel messages

- **Documentation and Code Quality**
  - Comprehensive API documentation with endpoint examples
  - Enhanced file organization instructions for development
  - Updated dependencies (chrono for timestamp formatting)
  - Removed redundant authentication messages for WebSocket connections

### Changed

- WebSocket connections now use token query parameters instead of separate auth messages
- Logging format shows `LOCAL:` and `PROXY:` instead of `INFO LOCAL:` and `INFO PROXY:`
- Improved error handling in WebSocket client for authentication responses
- Cleaner startup banner without emoji characters

### Fixed

- Compilation warnings for unused imports
- WebSocket authentication flow simplified and more reliable
- Professional logging output suitable for production environments

## [0.1.2] - 2025-06-30

### Added

- YAML configuration file support with `--yml` flag
- Support for multiple configuration file formats (YAML, TOML, JSON)
- Automatic configuration file detection in default locations:
  - `./pori.yml`, `./pori.yaml`, `./pori.toml`, `./pori.json`
  - `~/.pori.yml`, `~/.pori.yaml`, `~/.pori.toml`, `~/.pori.json`
  - `~/.config/pori/config.yml`, `~/.config/pori/config.yaml`, `~/.config/pori/config.toml`
- Optional CLI arguments when using configuration files
- Home directory expansion for config file paths
- PORI_YML environment variable for YAML config files

### Changed

- Renamed application from "tunnel-client" to "pori" (meaning wild, open)
- Updated all environment variables from `TUNNEL_*` to `PORI_*` prefix
- Made --url and --token optional when using configuration files
- Updated binary name from tunnel-client to pori
- Updated installation for global access like ngrok
- Updated dashboard default port to 7616
- Updated dependencies: dirs 5.0.1 → 6.0.0 and related transitive dependencies

### Fixed

- Configuration file loading with proper error handling
- Test suite compatibility with new CLI structure
- Clippy warnings: replaced `map_or` with `is_some_and` for better readability
- Clippy warnings: use `strip_prefix` instead of manual string slicing for safety
- Code formatting compliance for CI/CD pipeline

### Security

- Updated to latest dependency versions with security patches
- All dependencies now up-to-date (dirs, windows-sys, redox_users, etc.)

## [0.1.0] - 2025-06-30

### Added

- Initial release of tunnel-client
- Core WebSocket tunneling functionality
- HTTP proxy forwarding capabilities
- Dashboard server with monitoring interface
- Configuration management system
- Automated build and release pipeline
