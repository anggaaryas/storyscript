use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::{Signer as _, SigningKey};
use sha2::{Digest, Sha256};

use crate::{BundleError, Result};

const SIGNING_DOMAIN: &[u8] = b"StoryBundle-v1\0";

pub trait BundleSigner {
    fn key_id(&self) -> String;
    fn sign_manifest(&self, manifest: &[u8]) -> Result<[u8; 64]>;
}

pub struct Ed25519Signer {
    key: SigningKey,
}

impl Ed25519Signer {
    pub fn from_pkcs8_pem(pem: &str) -> Result<Self> {
        let key = SigningKey::from_pkcs8_pem(pem)
            .map_err(|error| BundleError::SigningKey(error.to_string()))?;
        Ok(Self { key })
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.key.verifying_key().to_bytes()
    }
}

impl BundleSigner for Ed25519Signer {
    fn key_id(&self) -> String {
        hex::encode(Sha256::digest(self.public_key_bytes()))
    }

    fn sign_manifest(&self, manifest: &[u8]) -> Result<[u8; 64]> {
        Ok(self.key.sign(&signing_digest(manifest)).to_bytes())
    }
}

pub fn signing_digest(manifest: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SIGNING_DOMAIN);
    hasher.update(manifest);
    hasher.finalize().into()
}
