# Contributing to cq

Thank you for your interest in contributing to cq! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We're all here to make Cardano development better.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/karkigrishmin/cq/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Transaction CBOR (if applicable, use preprod/testnet data)
   - cq version (`cq --version`)

### Suggesting Features

1. Check existing issues for similar suggestions
2. Create a new issue with:
   - Clear use case description
   - Example of how it would work
   - Why it's useful for Cardano developers

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run the checks:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Security audit
cargo audit
```

5. Commit with a clear message
6. Push to your fork
7. Open a Pull Request

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cq
cd cq

# Build
cargo build

# Run tests
cargo test

# Run with a fixture
cargo run -- tests/fixtures/babbage_simple.cbor
```

## Project Structure

```
cq/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # Argument parsing
│   ├── error.rs         # Error types
│   ├── input/           # Input handling (file, hex, stdin)
│   ├── decode/          # CBOR decoding with CML
│   ├── query/           # Query engine and shortcuts
│   └── format/          # Output formatting (pretty, JSON, raw)
├── tests/
│   ├── cli.rs           # Integration tests
│   └── fixtures/        # Test CBOR files
```

## Adding New Features

### Adding a Query Shortcut

1. Edit `src/query/shortcuts.rs`:

```rust
fn shortcut_expansion(shortcut: &str) -> Option<&'static str> {
    match shortcut {
        // ... existing shortcuts ...
        "your_shortcut" => Some("body.your.path"),
        _ => None,
    }
}
```

2. Add tests in `tests/cli.rs`
3. Update README.md with the new shortcut

### Adding Certificate Support

1. Edit `src/query/engine.rs`:
   - Add match arm in `certificate_to_json()`
2. Edit `src/format/pretty.rs`:
   - Add display logic in `format_cert_type()`
3. Add test fixture and integration test

## Testing

### Getting Test Fixtures

Use Koios API to fetch real transactions:

```bash
# Preprod
curl -s -X POST 'https://preprod.koios.rest/api/v1/tx_cbor' \
  -H 'Content-Type: application/json' \
  -d '{"_tx_hashes": ["TX_HASH"]}' | jq -r '.[0].cbor' | xxd -r -p > tests/fixtures/new_fixture.cbor
```

### Running Specific Tests

```bash
cargo test test_name           # Single test
cargo test --test cli          # Integration tests only
cargo test --lib               # Unit tests only
```

## Style Guidelines

- Follow Rust standard formatting (`cargo fmt`)
- No clippy warnings (`cargo clippy -- -D warnings`)
- Write tests for new functionality
- Keep commits focused and atomic
- Use conventional commit messages when possible

## CML (Cardano Multiplatform Lib) Notes

When working with CML 6.0, be aware of these gotchas:

- `RawBytesEncoding` trait must be imported for `.to_raw_bytes()`
- `UnitInterval` uses `.start`/`.end` not `.numerator`/`.denominator`
- Conway cert types use `deposit` not `coin` field
- `RewardAddress.to_address()` takes ownership - clone first
- `DRep::AlwaysAbstain`/`AlwaysNoConfidence` are struct variants

## Questions?

Open an issue or discussion on GitHub. We're happy to help!
