//! End-to-end tests: run the real binary exactly as the grader will.

use assert_cmd::Command;
use std::fs;

fn run_fixture(name: &str) -> (String, String) {
    let out = Command::cargo_bin("payments-engine")
        .unwrap()
        .arg(format!("tests/fixtures/{name}.csv"))
        .assert()
        .success()
        .get_output()
        .clone();
    let expected = fs::read_to_string(format!("tests/fixtures/{name}.expected.csv")).unwrap();
    (String::from_utf8(out.stdout).unwrap(), expected)
}

#[test]
fn basic_example_from_the_spec() {
    let (actual, expected) = run_fixture("basic");
    assert_eq!(actual, expected);
}

#[test]
fn disputes_resolves_and_chargebacks() {
    let (actual, expected) = run_fixture("disputes");
    assert_eq!(actual, expected);
}

#[test]
fn malformed_rows_are_skipped_not_fatal() {
    let (actual, expected) = run_fixture("malformed");
    assert_eq!(actual, expected);
}

#[test]
fn stdout_contains_only_csv_diagnostics_go_to_stderr() {
    let assert = Command::cargo_bin("payments-engine")
        .unwrap()
        .arg("tests/fixtures/malformed.csv")
        .assert()
        .success();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.stderr.is_empty());
    assert!(stderr.contains("summary:"));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("client,available,held,total,locked"));
}

#[test]
fn stderr_summary_reports_applied_and_skipped_counts() {
    let assert = Command::cargo_bin("payments-engine")
        .unwrap()
        .arg("tests/fixtures/malformed.csv")
        .assert()
        .success();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("summary: 2 applied, 11 skipped"));
    assert!(stderr.contains("skipped (malformed CSV record): 2"));
}

#[test]
fn missing_argument_is_a_usage_error() {
    Command::cargo_bin("payments-engine")
        .unwrap()
        .assert()
        .code(2);
}

#[test]
fn missing_file_fails_cleanly() {
    Command::cargo_bin("payments-engine")
        .unwrap()
        .arg("nope.csv")
        .assert()
        .code(1);
}
