#![allow(dead_code)]

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use ed25519_dalek::{SigningKey, VerifyingKey};
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use storyscript_bundle::exporter;
use storyscript_bundle::manifest::{BundleManifest, MANIFEST_PATH, SIGNATURE_PATH};
use storyscript_bundle::signing::{BundleSigner, Ed25519Signer};
use storyscript_bundle::trust::TrustStore;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

pub struct TestBundle {
    pub bytes: Vec<u8>,
    pub signing_key: SigningKey,
}

impl TestBundle {
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pem = signing_key.to_pkcs8_pem(LineEnding::LF).expect("PKCS#8");
        let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
        let bytes = exporter::export(&fixture(), &signer).expect("export");
        Self { bytes, signing_key }
    }

    pub fn trust_store(&self) -> TrustStore {
        let mut trust = TrustStore::new();
        trust.insert_key(self.signing_key.verifying_key());
        trust
    }

    pub fn public_key_pem(&self) -> String {
        self.signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("public key PEM")
    }

    pub fn rewrite<F>(&self, mut rewrite: F) -> Vec<u8>
    where
        F: FnMut(&str, &mut Vec<u8>) -> bool,
    {
        let mut entries = read_entries(&self.bytes);
        entries.retain_mut(|(path, bytes)| rewrite(path, bytes));
        write_entries(&entries)
    }

    pub fn rewrite_manifest<F>(&self, rewrite: F, resign: bool) -> Vec<u8>
    where
        F: FnOnce(&mut BundleManifest),
    {
        let mut entries = read_entries(&self.bytes);
        let manifest_entry = entries
            .iter_mut()
            .find(|(path, _)| path == MANIFEST_PATH)
            .expect("manifest");
        let mut manifest: BundleManifest =
            serde_json::from_slice(&manifest_entry.1).expect("manifest JSON");
        rewrite(&mut manifest);
        manifest_entry.1 = manifest.canonical_bytes().expect("canonical manifest");
        if resign {
            let pem = self
                .signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .expect("PKCS#8");
            let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
            let signature = signer
                .sign_manifest(&manifest_entry.1)
                .expect("signature")
                .to_vec();
            entries
                .iter_mut()
                .find(|(path, _)| path == SIGNATURE_PATH)
                .expect("signature entry")
                .1 = signature;
        }
        write_entries(&entries)
    }

    pub fn rewrite_payload(&self, path: &str, replacement: Vec<u8>) -> Vec<u8> {
        let mut entries = read_entries(&self.bytes);
        entries
            .iter_mut()
            .find(|(entry_path, _)| entry_path == path)
            .expect("payload entry")
            .1 = replacement;

        let payloads = entries
            .iter()
            .filter(|(entry_path, _)| entry_path != MANIFEST_PATH && entry_path != SIGNATURE_PATH)
            .cloned()
            .collect::<std::collections::HashMap<_, _>>();
        let manifest_entry = entries
            .iter_mut()
            .find(|(entry_path, _)| entry_path == MANIFEST_PATH)
            .expect("manifest");
        let mut manifest: BundleManifest =
            serde_json::from_slice(&manifest_entry.1).expect("manifest JSON");
        for listed in &mut manifest.entries {
            let bytes = payloads.get(&listed.path).expect("listed payload");
            listed.compressed_size = bytes.len() as u64;
            listed.uncompressed_size = bytes.len() as u64;
            listed.sha256 = hex::encode(Sha256::digest(bytes));
        }
        manifest_entry.1 = manifest.canonical_bytes().expect("canonical manifest");
        let pem = self
            .signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("PKCS#8");
        let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
        let signature = signer
            .sign_manifest(&manifest_entry.1)
            .expect("signature")
            .to_vec();
        entries
            .iter_mut()
            .find(|(entry_path, _)| entry_path == SIGNATURE_PATH)
            .expect("signature entry")
            .1 = signature;
        write_entries(&entries)
    }
}

pub fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

pub fn unrelated_key() -> VerifyingKey {
    SigningKey::generate(&mut OsRng).verifying_key()
}

pub fn read_entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("ZIP");
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).expect("entry");
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("entry bytes");
            (entry.name().to_string(), bytes)
        })
        .collect()
}

pub fn write_entries(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(DateTime::default())
            .unix_permissions(0o644);
        for (path, bytes) in entries {
            archive.start_file(path, options).expect("start entry");
            archive.write_all(bytes).expect("write entry");
        }
        archive.finish().expect("finish ZIP");
    }
    output.into_inner()
}
