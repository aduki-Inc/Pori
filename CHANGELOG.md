# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Updated all environment variables from TUNNEL_* to PORI_* prefix
- Made --url and --token optional when using configuration files
- Updated binary name from tunnel-client to pori
- Updated installation for global access like ngrok
- Updated dashboard default port to 7616

### Fixed

- Configuration file loading with proper error handling
- Test suite compatibility with new CLI structure

## [0.1.0] - 2025-06-30

### Added

- Initial release of tunnel-client
- Core WebSocket tunneling functionality
- HTTP proxy forwarding capabilities
- Dashboard server with monitoring interface
- Configuration management system
- Automated build and release pipeline
