# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-15

### Added

- Initial release of cq - CBOR Query Tool for Cardano
- Parse Cardano transactions from file, hex string, or stdin
- Query with dot-notation paths (e.g., `outputs.0.address`)
- Wildcard support (e.g., `outputs.*.value`)
- 17 query shortcuts (`fee`, `inputs`, `outputs`, `hash`, `certs`, etc.)
- Pretty terminal output with colors and tables
- JSON output mode (`--json`)
- Raw CBOR diagnostic notation (`--raw`)
- ADA display mode (`--ada`) for human-readable amounts
- Validation mode (`--check`)
- Babbage era support (all transaction types)
- Conway era governance support:
  - DRep registration/deregistration/update
  - Vote delegation certificates
  - Committee certificates
- Certificate extraction and display:
  - Stake registration/deregistration/delegation
  - Pool registration/retirement
  - All Conway governance certificates
- Withdrawal extraction with bech32 reward addresses
- Plutus transaction support:
  - Script data hash display
  - Collateral inputs/return/total
  - Inline datums
  - Redeemer detection
- Multi-asset support with policy IDs and token names
- Bech32 address formatting (auto-detect network)
- 79 tests (44 unit + 35 integration)

### Technical

- Built with CML (Cardano Multiplatform Lib) 6.0
- Native Rust implementation (no WASM)
- Supports Rust 1.85+ (edition 2024)

[Unreleased]: https://github.com/karkigrishmin/cq/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/karkigrishmin/cq/releases/tag/v0.1.0
