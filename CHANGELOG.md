# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-15

### Added

- **Asset name decoding**: Token names now display as UTF-8 when valid (e.g., "NIGHT" instead of "4e49474854")
- **CIP metadata standard support**:
  - CIP-20 (label 674): Transaction messages
  - CIP-25 (label 721): NFT metadata
  - CIP-68 (labels 100, 222, 333, 444): Datum metadata standard
- **Address parsing**: Output addresses now include detailed components:
  - Address type (base, enterprise, reward, pointer, byron)
  - Network (mainnet/testnet per CIP-19)
  - Payment and stake credentials with hashes
- **Standalone address command**: `cq addr <bech32>` decodes any Cardano address
- **Query filtering**: Filter arrays with bracket syntax:
  - `outputs[value.coin > 1000000]` - numeric comparisons
  - `outputs[address.address ~ "addr1"]` - string contains
  - `outputs[datum != null]` - existence checks
  - Operators: `>`, `<`, `>=`, `<=`, `==`, `!=`, `~`

### Changed

- Address output is now a JSON object with `address`, `type`, `network`, and credential fields
- Shortcuts now work with filter syntax (e.g., `outputs[value.coin > 1000000]`)

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

[Unreleased]: https://github.com/karkigrishmin/cq/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/karkigrishmin/cq/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/karkigrishmin/cq/releases/tag/v0.1.0
