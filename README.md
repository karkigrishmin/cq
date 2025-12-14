# cq - CBOR Query Tool for Cardano

[![CI](https://github.com/karkigrishmin/cq/actions/workflows/ci.yml/badge.svg)](https://github.com/karkigrishmin/cq/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/cq.svg)](https://crates.io/crates/cq)

**Think "jq for Cardano CBOR"** - A fast CLI tool to inspect and query Cardano transactions encoded in CBOR format.

Born from a real pain point in the [2025 Cardano Developer Survey](https://cardano.org): developers lack good CLI tooling for debugging CBOR transactions. `cq` fills that gap.

## Features

- **Parse transactions** from file, hex string, or stdin
- **Query with dot-notation** - `cq outputs.0.address tx.cbor`
- **Wildcard support** - `cq outputs.*.value tx.cbor`
- **Bech32 addresses** - Auto-formatted for readability
- **Pretty terminal output** - Colors, tables, smart truncation
- **JSON output** - Perfect for piping to `jq`
- **Babbage + Conway eras** - Full support including governance
- **Blazing fast** - Native Rust, no WASM overhead

## Installation

### From source (recommended)

```bash
cargo install --git https://github.com/karkigrishmin/cq
```

### From crates.io

```bash
cargo install cq
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/karkigrishmin/cq/releases).

## Quick Start

```bash
# Show full transaction (pretty output)
cq transaction.cbor

# Query specific fields
cq fee tx.cbor                    # Transaction fee
cq fee tx.cbor --ada              # Fee in ADA (not lovelace)
cq hash tx.cbor                   # Transaction hash

# Nested queries
cq outputs.0.address tx.cbor      # First output address
cq outputs.0.value.coin tx.cbor   # First output ADA amount
cq inputs.0.transaction_id tx.cbor

# Wildcard queries
cq outputs.*.address tx.cbor      # All output addresses
cq outputs.*.value tx.cbor        # All output values

# Different output formats
cq tx.cbor --json                 # JSON output
cq tx.cbor --raw                  # CBOR diagnostic notation

# Validation mode
cq tx.cbor --check && echo "Valid!"

# Read from stdin
cat tx.cbor | cq
cat tx.cbor | cq fee --ada

# Hex input (with or without 0x prefix)
cq 84a400818258203b40265111d8bb3c3c...
cq 0x84a400818258203b40265111d8bb3c3c...
```

## Query Shortcuts

| Shortcut | Expands To | Description |
|----------|------------|-------------|
| `fee` | `body.fee` | Transaction fee |
| `inputs` | `body.inputs` | Input UTxOs |
| `outputs` | `body.outputs` | Output UTxOs |
| `hash` | *(computed)* | Transaction hash |
| `metadata` | `auxiliary_data.metadata` | Transaction metadata |
| `witnesses` | `witness_set` | Signatures & scripts |
| `ttl` | `body.ttl` | Time to live |
| `mint` | `body.mint` | Minted assets |
| `certs` | `body.certs` | Certificates |
| `withdrawals` | `body.withdrawals` | Stake withdrawals |
| `collateral` | `body.collateral_inputs` | Collateral inputs |
| `required_signers` | `body.required_signers` | Required signers |
| `network_id` | `body.network_id` | Network ID |
| `validity_start` | `body.validity_interval_start` | Valid from slot |
| `script_data_hash` | `body.script_data_hash` | Plutus script data hash |
| `collateral_return` | `body.collateral_return` | Collateral return output |
| `total_collateral` | `body.total_collateral` | Total collateral amount |

## Example Output

```
$ cq transaction.cbor

Transaction
  Hash: 0edb4eac0b992ac4af71a2a52f41ab63c806e0ef4e5c5d9c7348ea03cf9a9e4e
  Valid: true

Body
  Fee: 171,617 lovelace

Inputs (1)
┌───┬─────────────────┬───────┐
│ # ┆ Transaction ID  ┆ Index │
╞═══╪═════════════════╪═══════╡
│ 0 ┆ 852ec7...73fa31 ┆ 0     │
└───┴─────────────────┴───────┘

Outputs (2)
┌───┬──────────────────────────┬────────────────────────┬───────┐
│ # ┆ Address                  ┆ Value                  ┆ Datum │
╞═══╪══════════════════════════╪════════════════════════╪═══════╡
│ 0 ┆ addr_test1vp9...jg52l8g8 ┆ 9,594,993,891 lovelace ┆ -     │
│ 1 ┆ addr_test1qz8...h7q3xhdsk│ 1,500,000 lovelace     ┆ -     │
└───┴──────────────────────────┴────────────────────────┴───────┘

Witnesses
  VKey signatures: 2
```

## Fetching Real Transactions

Use [Koios API](https://koios.rest) to fetch CBOR from mainnet/testnet:

```bash
# Mainnet
curl -s -X POST 'https://api.koios.rest/api/v1/tx_cbor' \
  -H 'Content-Type: application/json' \
  -d '{"_tx_hashes": ["YOUR_TX_HASH"]}' | jq -r '.[0].cbor' | xxd -r -p | cq

# Preprod testnet
curl -s -X POST 'https://preprod.koios.rest/api/v1/tx_cbor' \
  -H 'Content-Type: application/json' \
  -d '{"_tx_hashes": ["YOUR_TX_HASH"]}' | jq -r '.[0].cbor' | xxd -r -p | cq
```

**Pro tip** - Add this to your `.bashrc`:

```bash
cqtx() {
  curl -s -X POST 'https://api.koios.rest/api/v1/tx_cbor' \
    -H 'Content-Type: application/json' \
    -d "{\"_tx_hashes\": [\"$1\"]}" | jq -r '.[0].cbor' | xxd -r -p | cq "${@:2}"
}

# Usage: cqtx <tx_hash> [query] [flags]
# cqtx 31ed9234a830667a0152fbfe4a244f896f5aad459831a5620571465283ec5f0c
# cqtx 31ed9234... fee --ada
```

## Supported Certificate Types

### Babbage Era
- `StakeRegistration`, `StakeDeregistration`, `StakeDelegation`
- `PoolRegistration`, `PoolRetirement`

### Conway Era (Governance)
- `RegCert`, `UnregCert`
- `VoteDelegCert`, `StakeVoteDelegCert`
- `StakeRegDelegCert`, `VoteRegDelegCert`, `StakeVoteRegDelegCert`
- `AuthCommitteeHotCert`, `ResignCommitteeColdCert`
- `RegDrepCert`, `UnregDrepCert`, `UpdateDrepCert`

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation failed (invalid CBOR/transaction) |
| 2 | Parse error |
| 3 | I/O error (file not found, etc.) |
| 4 | Query error (field not found, index out of bounds) |

## Comparison with Alternatives

| Feature | cq | cardano-cli | CQUISITOR | cbor.me |
|---------|-----|-------------|-----------|---------|
| Offline | Yes | Yes | No | No |
| Query syntax | Dot notation | Flags | GUI | N/A |
| JSON output | Yes | Yes | No | No |
| Scriptable | Yes | Yes | No | No |
| Pretty output | Yes | Limited | Yes | Yes |
| Install | `cargo install` | Heavy | Browser | Browser |

## Building from Source

```bash
git clone https://github.com/karkigrishmin/cq
cd cq
cargo build --release
./target/release/cq --help
```

### Requirements

- Rust 1.85+ (edition 2024)
- No system dependencies

### Running Tests

```bash
cargo test              # All tests (79 total)
cargo test --test cli   # Integration tests only
cargo test --lib        # Unit tests only
```

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `cargo test` and `cargo clippy`
4. Submit a PR

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [CML (Cardano Multiplatform Lib)](https://github.com/dcSpark/cardano-multiplatform-lib) for Cardano type parsing
- Inspired by the excellent [jq](https://stedolan.github.io/jq/) tool
