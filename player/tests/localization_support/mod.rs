#![allow(dead_code)]
use std::{fs, path::Path};
pub fn project() -> tempfile::TempDir {
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let root = tempfile::tempdir().unwrap();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../bundle/rust/tests/fixtures/localized_project"),
        root.path(),
    );
    root
}
pub fn replace(root: &Path, path: &str, from: &str, to: &str) {
    let path = root.join(path);
    let source = fs::read_to_string(&path).unwrap();
    assert!(source.contains(from));
    fs::write(path, source.replace(from, to)).unwrap();
}
pub fn append(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    let source = fs::read_to_string(&path).unwrap();
    fs::write(path, format!("{source}\n{text}\n")).unwrap();
}
pub fn prefs(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|s| s.to_string()).collect()
}

pub fn signed(root: &Path) -> (Vec<u8>, storyscript_bundle::trust::TrustStore) {
    use ed25519_dalek::{Signer, SigningKey};
    struct TestSigner(SigningKey);
    impl storyscript_bundle::signing::BundleSigner for TestSigner {
        fn key_id(&self) -> String {
            storyscript_bundle::trust::key_id_for_bytes(&self.0.verifying_key().to_bytes())
        }
        fn sign_manifest(&self, manifest: &[u8]) -> storyscript_bundle::Result<[u8; 64]> {
            Ok(self
                .0
                .sign(&storyscript_bundle::signing::signing_digest(manifest))
                .to_bytes())
        }
    }
    // Deterministic test-only in-memory key, never used for shipped fixtures.
    let signer = TestSigner(SigningKey::from_bytes(&[19; 32]));
    let mut trust = storyscript_bundle::trust::TrustStore::new();
    trust.insert_key(signer.0.verifying_key());
    (
        storyscript_bundle::exporter::export(root, &signer).unwrap(),
        trust,
    )
}
