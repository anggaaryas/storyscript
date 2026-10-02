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
mod localization_support;

#[test]
fn localize_extract_sync_check_are_deterministic_and_preserve_translator_content() {
    let root = localization_support::project();
    let run = |action: &str| {
        std::process::Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
            .args([
                "localize",
                action,
                "--project",
                root.path().to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap()
    };
    let first = run("extract");
    assert!(first.status.success());
    assert_eq!(first.stdout, run("extract").stdout);
    assert!(!String::from_utf8_lossy(&first.stdout).contains(root.path().to_str().unwrap()));
    let path = root.path().join("localization/id.ftl");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(run("check").status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), old);
    localization_support::replace(
        root.path(),
        "story/main.StoryScript",
        "@\"greeting\";",
        "@\"greeting\"; @\"new-line\";",
    );
    assert!(!run("check").status.success());
    assert!(run("sync").status.success());
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.starts_with(&old));
    assert!(updated.contains("new-line = new-line"));
    assert!(run("check").status.success());
    assert!(run("sync").status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), updated);
}

#[test]
fn sync_reports_contract_conflicts_and_obsolete_records_without_overwriting_them() {
    let root = localization_support::project();
    localization_support::replace(
        root.path(),
        "localization/id.ftl",
        "{ $name }",
        "{ $count }",
    );
    let path = root.path().join("localization/id.ftl");
    let before = std::fs::read_to_string(&path).unwrap();
    let report = storyscript_bundle::localization::author_command(root.path(), "sync").unwrap();
    assert!(
        report["conflicts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "greeting")
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    let path = root.path().join("localization/en.ftl");
    let before = format!(
        "{}\nobsolete = Retain me\n",
        std::fs::read_to_string(&path).unwrap()
    );
    std::fs::write(&path, &before).unwrap();
    let report = storyscript_bundle::localization::author_command(root.path(), "sync").unwrap();
    assert!(
        report["catalogs"][0]["obsolete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "obsolete")
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    assert!(
        !report["catalogs"][0]["obsolete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "-brand")
    );
}
