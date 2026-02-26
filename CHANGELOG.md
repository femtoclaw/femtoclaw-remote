# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-02-25

### Added
- Axum-based HTTP server
- WebSocket support for real-time communication
- State management for agent sessions
- REST API endpoints

### Changed
- Version bump from 1.0.0 to 1.0.1
- Added Unpin bound for WebSocket futures

### Fixed
- Build errors resolved (missing uuid, Unpin bound)

## [1.0.0] - 2026-02-25

### Added
- Initial release of femtoclaw-remote
- Basic server functionality
