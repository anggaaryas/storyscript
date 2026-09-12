use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

fn write_key(directory: &Path) -> PathBuf {
    let path = directory.join("signing-key.pem");
    let pem = SigningKey::generate(&mut OsRng)
        .to_pkcs8_pem(LineEnding::LF)
        .expect("PKCS#8");
    fs::write(&path, pem.as_bytes()).expect("key file");
    path
}

#[test]
fn export_cli_writes_storybundle_and_json_success() {
    let temp = tempfile::tempdir().expect("temporary output");
    let key = write_key(temp.path());
    let output = temp.path().join("demo.storybundle");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "export",
            "--project",
            fixture().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--signing-key",
            key.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("run CLI");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.is_file());
    assert!(String::from_utf8_lossy(&result.stdout).contains(r#""status":"ok""#));
}

#[test]
fn export_cli_rejects_wrong_extension_with_machine_error() {
    let temp = tempfile::tempdir().expect("temporary output");
    let key = write_key(temp.path());
    let output = temp.path().join("demo.zip");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "export",
            "--project",
            fixture().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--signing-key",
            key.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("run CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_OUTPUT_INVALID"));
    assert!(!output.exists());
}

#[test]
fn export_cli_refuses_project_local_private_key() {
    let project = tempfile::tempdir().expect("temporary project");
    let key = write_key(project.path());
    let output = tempfile::tempdir()
        .expect("output directory")
        .path()
        .join("demo.storybundle");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "export",
            "--project",
            project.path().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--signing-key",
            key.to_str().unwrap(),
        ])
        .output()
        .expect("run CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_SIGNING_KEY_INVALID"));
}
