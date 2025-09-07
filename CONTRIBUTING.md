# Contributing to RustLlama

Thank you for your interest in contributing to RustLlama! We welcome contributions of all kinds.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/rustlama.git
   cd rustlama
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test
   ```

## 🔧 Development Guidelines

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Ensure code passes linting: `cargo clippy`
- Add tests for new features
- Update documentation as needed

### Commit Guidelines

- Use clear, descriptive commit messages
- Follow conventional commits format when possible:
  - `feat: add new sampling method`
  - `fix: resolve memory leak in token processing`
  - `docs: update README examples`

### Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature-name`
2. Make your changes
3. Add tests if applicable
4. Run the full test suite: `cargo test`
5. Update documentation if needed
6. Submit a pull request with a clear description

## 🐛 Reporting Bugs

When reporting bugs, please include:

- Operating system and version
- Rust version (`rustc --version`)
- Model file details (format, size)
- Full command that caused the issue
- Complete error output
- Steps to reproduce

## 💡 Feature Requests

We're always interested in new ideas! For feature requests:

- Check if the feature already exists or is planned
- Open an issue describing the feature
- Explain the use case and benefits
- Consider contributing the implementation

## 📖 Documentation

Help improve our documentation by:

- Adding examples
- Fixing typos
- Improving clarity
- Adding missing information

## 🔄 Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag
4. Build and test release binary
5. Create GitHub release

## 🙏 Recognition

All contributors will be recognized in our README and release notes. Thank you for making RustLlama better!

## 📞 Questions?

Feel free to open an issue for questions or join our discussions.
