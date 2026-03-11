# ah-ah-ah

[![CI](https://github.com/claylo/ah-ah-ah/actions/workflows/ci.yml/badge.svg)](https://github.com/claylo/ah-ah-ah/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ah-ah-ah.svg)](https://crates.io/crates/ah-ah-ah)
[![docs.rs](https://docs.rs/ah-ah-ah/badge.svg)](https://docs.rs/ah-ah-ah)
[![MSRV](https://img.shields.io/badge/MSRV-1.89.0-blue.svg)](https://github.com/claylo/ah-ah-ah)


## Development

Single library crate — source lives in `src/`.

### Prerequisites

- Rust 1.89.0+ (2024 edition)
- [just](https://github.com/casey/just) (task runner)
- [cargo-nextest](https://nexte.st/) (test runner)

### Quick Start

```bash
# List available tasks
just --list

# Run full check suite (format, lint, test)
just check

# Run tests only
just test

# Run with coverage
just cov
```

### Build Tasks

| Command | Description |
|---------|-------------|
| `just check` | Format, lint, and test |
| `just fmt` | Format code with rustfmt |
| `just clippy` | Run clippy lints |
| `just test` | Run tests with nextest |
| `just doc-test` | Run documentation tests |
| `just cov` | Generate coverage report |


## Architecture

### Crate Organization

- **ah-ah-ah** - Token counting library with pluggable backends.

### Error Handling

- Uses `thiserror` for structured error types
- All errors include context for debugging


## CI/CD

This project uses GitHub Actions for continuous integration:

- **Build & Test** - Runs on every push and PR
- **MSRV Check** - Verifies minimum supported Rust version
- **Clippy** - Enforces lint rules
- **Coverage** - Tracks test coverage

### Dependabot

This project uses Dependabot for security monitoring, but **not** for automatic pull requests. Instead:

1. Dependabot scans for vulnerabilities in dependencies
2. A weekly GitHub Actions workflow converts alerts into **issues**
3. Maintainers review and address updates manually

This approach provides:
- Full control over when and how dependencies are updated
- Opportunity to batch related updates together
- Time to test updates before merging
- Cleaner git history without automated PR noise

Security alerts appear as issues labeled `dependabot-alert`.

## Contributing

Contributions welcome! Please see [AGENTS.md](AGENTS.md) for development conventions.

### Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

### Code Style

- Rust 2024 edition
- `#![deny(unsafe_code)]` - Safe Rust only
- Follow `rustfmt` defaults
- Keep clippy clean

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

