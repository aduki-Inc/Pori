# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2025-07-13

### Fixed

- **Code Formatting & CI/CD Compliance**
  - Fixed Rust code formatting to comply with rustfmt standards across all modules
  - Resolved import ordering issues in `src/websocket/mod.rs` and `src/protocol/websocket.rs`
  - Fixed multi-line formatting for macro calls in `src/main.rs` and `src/lib.rs`
  - Corrected string literal formatting in dashboard initialization logging
  - Fixed method chaining formatting in WebSocket message compression handling
  - Resolved all CI/CD formatting check failures that were blocking releases

- **Build System**
  - Ensured clean compilation with no formatting warnings
  - Updated build target files to reflect version 0.1.4
  - Validated release build process works correctly

### Changed

- **Code Quality Improvements**
  - Enhanced code readability with consistent formatting across the entire codebase
  - Improved import organization following Rust style guidelines
  - Standardized macro usage formatting for better maintainability
  - Updated CI/CD pipeline compliance to prevent future formatting issues

### Technical Details

- All source files now pass `cargo fmt --all -- --check` validation
- Import statements reorganized to follow `use crate::` before external crates pattern
- Multi-line function calls properly formatted with appropriate indentation
- Method chaining now follows consistent line-breaking rules

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
