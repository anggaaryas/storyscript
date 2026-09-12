mod common;

use std::fs;
use std::process::Command;

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePublicKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;

use common::TestBundle;

#[test]
fn inspect_is_explicitly_untrusted_and_does_not_print_model_text() {
    let bundle = TestBundle::new();
    let temp = tempfile::tempdir().expect("temporary files");
    let path = temp.path().join("demo.storybundle");
    fs::write(&path, &bundle.bytes).expect("bundle");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args(["inspect", path.to_str().unwrap(), "--json"])
        .output()
        .expect("inspect CLI");

    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!stdout.contains(r#""status""#));
    assert!(stdout.contains(r#""archive_size""#));
    assert!(stdout.contains(r#""entry_paths""#));
    assert!(stdout.contains("untrusted"));
    assert!(!stdout.contains("Welcome"));
}

#[test]
fn verify_requires_the_trusted_public_key_and_returns_json_status() {
    let bundle = TestBundle::new();
    let temp = tempfile::tempdir().expect("temporary files");
    let bundle_path = temp.path().join("demo.storybundle");
    let key_path = temp.path().join("public.pem");
    fs::write(&bundle_path, &bundle.bytes).expect("bundle");
    fs::write(&key_path, bundle.public_key_pem()).expect("public key");

    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "verify",
            bundle_path.to_str().unwrap(),
            "--public-key",
            key_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("verify CLI");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains(r#""status":"verified""#));
}

#[test]
fn verify_wrong_key_has_stable_nonzero_json_error() {
    let bundle = TestBundle::new();
    let temp = tempfile::tempdir().expect("temporary files");
    let bundle_path = temp.path().join("demo.storybundle");
    let key_path = temp.path().join("wrong-public.pem");
    fs::write(&bundle_path, &bundle.bytes).expect("bundle");
    let wrong = SigningKey::generate(&mut OsRng)
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .expect("public key");
    fs::write(&key_path, wrong).expect("public key");

    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "verify",
            bundle_path.to_str().unwrap(),
            "--public-key",
            key_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("verify CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_UNKNOWN_SIGNER"));
}
