use sha2::{Digest, Sha256};

use crate::{BundleError, Result};

pub const DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/storybundle_descriptor.bin"));
pub const EXPECTED_SHA256: &str = include_str!("../../proto/storybundle/v1/schema.sha256");

pub fn descriptor_sha256() -> String {
    hex::encode(Sha256::digest(DESCRIPTOR_SET))
}

pub fn check() -> Result<String> {
    let actual = descriptor_sha256();
    let expected = EXPECTED_SHA256.trim();
    if actual != expected {
        return Err(BundleError::SchemaMismatch {
            expected: expected.to_string(),
            found: actual,
        });
    }
    Ok(actual)
}
