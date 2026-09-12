use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use storyscript_bundle::exporter;
use storyscript_bundle::manifest::{BundleManifest, MANIFEST_PATH, SIGNATURE_PATH};
use storyscript_bundle::signing::Ed25519Signer;
use zip::ZipArchive;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

fn signer() -> Ed25519Signer {
    let pem = SigningKey::generate(&mut OsRng)
        .to_pkcs8_pem(LineEnding::LF)
        .expect("PKCS#8");
    Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer")
}

#[test]
fn identical_inputs_and_key_produce_byte_identical_archives() {
    let signer = signer();
    let first = exporter::export(&fixture(), &signer).expect("first export");
    let second = exporter::export(&fixture(), &signer).expect("second export");
    assert_eq!(first, second);
}

#[test]
fn archive_has_fixed_metadata_complete_manifest_and_no_source() {
    let bytes = exporter::export(&fixture(), &signer()).expect("export");
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("ZIP");
    let mut names = Vec::new();
    let mut payload_names = Vec::new();
    let mut manifest_bytes = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().to_string();
        assert_eq!(entry.last_modified().unwrap().year(), 1980);
        assert_eq!(entry.unix_mode(), Some(0o100644));
        if name == MANIFEST_PATH {
            entry.read_to_end(&mut manifest_bytes).expect("manifest");
        } else if name != SIGNATURE_PATH {
            payload_names.push(name.clone());
        }
        names.push(name);
    }

    assert!(names.contains(&MANIFEST_PATH.to_string()));
    assert!(names.contains(&SIGNATURE_PATH.to_string()));
    assert!(
        names
            .iter()
            .all(|name| !name.to_lowercase().ends_with(".storyscript"))
    );
    let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    let indexed = manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    payload_names.sort();
    assert_eq!(indexed, payload_names);
    assert!(!String::from_utf8_lossy(&manifest_bytes).contains(env!("CARGO_MANIFEST_DIR")));
}
