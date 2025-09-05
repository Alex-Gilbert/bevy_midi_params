# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-04

### Added
- Initial release with core functionality
- `MidiParams` derive macro for automatic MIDI parameter binding
- Support for various control types:
  - Scalar values (f32, i32, etc.)
  - Vector types (Vec2, Vec3, Vec4)
  - Color types (LinearRgba)
  - Boolean toggles
- Auto-discovery of connected MIDI controllers
- Hot-reload support for mapping changes
- Comprehensive examples demonstrating usage

[0.1.0]: https://github.com/Alex-Gilbert/bevy_midi_params/releases/tag/v0.1.0