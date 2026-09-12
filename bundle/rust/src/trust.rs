use std::collections::HashMap;

use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

use crate::{BundleError, Result};

#[derive(Debug, Default, Clone)]
pub struct TrustStore {
    keys: HashMap<String, VerifyingKey>,
}

impl TrustStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key_id: impl Into<String>, bytes: [u8; 32]) -> Result<()> {
        let key_id = key_id.into();
        let expected = key_id_for_bytes(&bytes);
        if key_id != expected {
            return Err(BundleError::SigningKey(format!(
                "key ID '{key_id}' does not match public key digest '{expected}'"
            )));
        }
        let key = VerifyingKey::from_bytes(&bytes)
            .map_err(|error| BundleError::SigningKey(error.to_string()))?;
        self.keys.insert(key_id, key);
        Ok(())
    }

    pub fn insert_key(&mut self, key: VerifyingKey) -> String {
        let key_id = key_id_for_bytes(&key.to_bytes());
        self.keys.insert(key_id.clone(), key);
        key_id
    }

    pub fn get(&self, key_id: &str) -> Option<&VerifyingKey> {
        self.keys.get(key_id)
    }
}

pub fn key_id_for_bytes(bytes: &[u8; 32]) -> String {
    hex::encode(Sha256::digest(bytes))
}
