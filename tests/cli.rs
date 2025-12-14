//! Integration tests for the cq CLI.

#![allow(deprecated)] // cargo_bin deprecation doesn't affect standard builds

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get the path to the test fixture.
fn fixture_path() -> &'static str {
    "tests/fixtures/babbage_simple.cbor"
}

/// Get the hex string of the test fixture.
fn fixture_hex() -> String {
    let bytes = fs::read(fixture_path()).expect("Failed to read fixture");
    hex::encode(bytes)
}

#[test]
fn test_show_help() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CBOR Query Tool for Cardano"));
}

#[test]
fn test_show_version() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cq"));
}

#[test]
fn test_full_transaction_from_file() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg(fixture_path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"))
        .stdout(predicate::str::contains("Hash:"))
        .stdout(predicate::str::contains("Fee:"))
        .stdout(predicate::str::contains("Inputs"))
        .stdout(predicate::str::contains("Outputs"));
}

#[test]
fn test_full_transaction_from_hex() {
    let hex = fixture_hex();
    Command::cargo_bin("cq")
        .unwrap()
        .arg(&hex)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"))
        .stdout(predicate::str::contains("Hash:"));
}

#[test]
fn test_full_transaction_from_hex_with_prefix() {
    let hex = format!("0x{}", fixture_hex());
    Command::cargo_bin("cq")
        .unwrap()
        .arg(&hex)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"));
}

#[test]
fn test_stdin_binary() {
    let bytes = fs::read(fixture_path()).expect("Failed to read fixture");
    Command::cargo_bin("cq")
        .unwrap()
        .write_stdin(bytes)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"));
}

#[test]
fn test_stdin_hex() {
    let hex = fixture_hex();
    Command::cargo_bin("cq")
        .unwrap()
        .write_stdin(hex)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"));
}

#[test]
fn test_query_fee() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["fee", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("171,617"));
}

#[test]
fn test_query_fee_ada() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["fee", fixture_path(), "--ada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.171617 ADA"));
}

#[test]
fn test_query_hash() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["hash", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("0edb4eac0b"));
}

#[test]
fn test_query_inputs() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["inputs", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("transaction_id"))
        .stdout(predicate::str::contains("index"));
}

#[test]
fn test_query_outputs() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["outputs", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("address"))
        .stdout(predicate::str::contains("value"));
}

#[test]
fn test_query_nested_outputs_address() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["outputs.0.address", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "addr_test1vp9s80tz7l3dxmg4wcsd6fwnjcxuqul6wy6x5pwt98hmhjg52l8g8",
        ));
}

#[test]
fn test_query_nested_outputs_value() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["outputs.0.value.coin", fixture_path(), "--ada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("9594.993891 ADA"));
}

#[test]
fn test_query_nested_inputs_txid() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["inputs.0.transaction_id", fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("852ec7f7da"));
}

#[test]
fn test_json_output() {
    Command::cargo_bin("cq")
        .unwrap()
        .args([fixture_path(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hash\":"))
        .stdout(predicate::str::contains("\"body\":"))
        .stdout(predicate::str::contains("\"is_valid\":"));
}

#[test]
fn test_json_output_query() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["body.fee", fixture_path(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("171617"));
}

#[test]
fn test_raw_output() {
    Command::cargo_bin("cq")
        .unwrap()
        .args([fixture_path(), "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hash\":"));
}

#[test]
fn test_check_mode_valid() {
    Command::cargo_bin("cq")
        .unwrap()
        .args([fixture_path(), "--check"])
        .assert()
        .success();
}

#[test]
fn test_check_mode_invalid() {
    // Create a temp file with invalid CBOR
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().join("invalid.cbor");
    fs::write(&temp_path, b"not valid cbor").unwrap();

    Command::cargo_bin("cq")
        .unwrap()
        .args([temp_path.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .code(1); // Validation fail exit code (DecodeFailed)
}

#[test]
fn test_no_color_flag() {
    Command::cargo_bin("cq")
        .unwrap()
        .args([fixture_path(), "--no-color"])
        .assert()
        .success()
        // Should not contain ANSI escape codes
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_file_not_found() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("nonexistent_file.cbor")
        .assert()
        .failure()
        .code(3) // IO error exit code
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_query_field_not_found() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["nonexistent_field", fixture_path()])
        .assert()
        .failure()
        .code(4) // Query error exit code
        .stderr(predicate::str::contains("Field not found"));
}

#[test]
fn test_query_index_out_of_bounds() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["outputs.99", fixture_path()])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("out of bounds"));
}

#[test]
fn test_stdin_with_query() {
    let bytes = fs::read(fixture_path()).expect("Failed to read fixture");
    Command::cargo_bin("cq")
        .unwrap()
        .arg("fee")
        .write_stdin(bytes)
        .assert()
        .success()
        .stdout(predicate::str::contains("171,617"));
}

// ===== Tests for new fixtures and features =====

#[test]
fn test_plutus_transaction_multi_assets() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("tests/fixtures/preprod_plutus.cbor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction"))
        .stdout(predicate::str::contains("Collateral"))
        .stdout(predicate::str::contains("asset(s)"));
}

#[test]
fn test_plutus_transaction_inline_datum() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("tests/fixtures/preprod_plutus.cbor")
        .assert()
        .success()
        .stdout(predicate::str::contains("<inline:"));
}

#[test]
fn test_plutus_transaction_validity_start() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("tests/fixtures/preprod_plutus.cbor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid from:"));
}

#[test]
fn test_pool_registration_certificate() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("tests/fixtures/pool_registration.cbor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Certificates"))
        .stdout(predicate::str::contains("Pool Registration"))
        .stdout(predicate::str::contains("margin"));
}

#[test]
fn test_query_certs_pool_registration() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["certs", "tests/fixtures/pool_registration.cbor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pool_registration"));
}

#[test]
fn test_drep_registration_certificate() {
    Command::cargo_bin("cq")
        .unwrap()
        .arg("tests/fixtures/drep_registration.cbor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Certificates"))
        .stdout(predicate::str::contains("Register DRep"))
        .stdout(predicate::str::contains("deposit"));
}

#[test]
fn test_query_certs_drep() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["certs", "tests/fixtures/drep_registration.cbor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reg_drep_cert"));
}

#[test]
fn test_plutus_collateral_query() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["collateral", "tests/fixtures/preprod_plutus.cbor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("transaction_id"))
        .stdout(predicate::str::contains("index"));
}

#[test]
fn test_multi_asset_output_json() {
    Command::cargo_bin("cq")
        .unwrap()
        .args([
            "outputs.0.value",
            "tests/fixtures/preprod_plutus.cbor",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("multi_assets"))
        .stdout(predicate::str::contains("policy_id"));
}

#[test]
fn test_pool_registration_json_output() {
    Command::cargo_bin("cq")
        .unwrap()
        .args(["tests/fixtures/pool_registration.cbor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"certs\":"))
        .stdout(predicate::str::contains("pool_registration"));
}
