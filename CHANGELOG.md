# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of RustLlama CLI
- Support for GGUF model format
- Configurable sampling parameters (temperature, top-k, top-p)
- Colored output with progress indicators
- Verbose mode and generation statistics
- Flexible context size and threading options
- Input validation and error handling
- Comprehensive help system

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- N/A (initial release)

## [0.1.0] - 2025-09-07

### Added
- Initial implementation of RustLlama CLI
- Basic LLaMA model inference capabilities
- Command-line argument parsing with clap
- Integration with llama-cpp-2 for model loading and inference
- Greedy sampling for token generation
- Real-time output streaming
- Error handling and validation
- MIT license
- Basic README and documentation

[Unreleased]: https://github.com/sangam14/rustlama/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sangam14/rustlama/releases/tag/v0.1.0
