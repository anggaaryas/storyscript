use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Project;
use crate::limits::ResourceLimits;
use crate::{BundleError, COMPILER_VERSION, FORMAT_VERSION, Result, schema};

pub const MANIFEST_PATH: &str = "META-INF/storybundle/manifest.json";
pub const SIGNATURE_PATH: &str = "META-INF/storybundle/signature.ed25519";
pub const COMPILED_PATH: &str = "compiled/story.pb";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub format_version: u32,
    pub compiler_version: String,
    pub schema_sha256: String,
    pub project: ManifestProject,
    pub signer_key_id: String,
    pub resource_limits: ManifestLimits,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestProject {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestLimits {
    pub max_archive_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_compiled_ir_bytes: u64,
    pub max_manifest_bytes: u64,
    pub max_entries: usize,
    pub max_path_bytes: usize,
    pub max_semantic_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub path: String,
    pub entry_type: EntryType,
    pub compression: Compression,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    CompiledStory,
    Asset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
    Stored,
}

impl BundleManifest {
    pub fn new(
        project: &Project,
        signer_key_id: String,
        compiled: &[u8],
        assets: &[(String, Vec<u8>)],
        limits: ResourceLimits,
    ) -> Self {
        let mut entries = Vec::with_capacity(assets.len() + 1);
        entries.push(ManifestEntry::new(
            COMPILED_PATH.to_string(),
            EntryType::CompiledStory,
            compiled,
        ));
        entries.extend(assets.iter().map(|(path, bytes)| {
            ManifestEntry::new(format!("assets/{path}"), EntryType::Asset, bytes)
        }));
        entries.sort_by(|left, right| left.path.cmp(&right.path));

        Self {
            format_version: FORMAT_VERSION,
            compiler_version: COMPILER_VERSION.to_string(),
            schema_sha256: schema::descriptor_sha256(),
            project: ManifestProject {
                id: project.id.clone(),
                name: project.name.clone(),
                version: project.version.clone(),
            },
            signer_key_id,
            resource_limits: limits.into(),
            entries,
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec(self)
            .map_err(|error| BundleError::Archive(format!("manifest JSON: {error}")))?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

impl ManifestEntry {
    fn new(path: String, entry_type: EntryType, bytes: &[u8]) -> Self {
        let size = bytes.len() as u64;
        Self {
            path,
            entry_type,
            compression: Compression::Stored,
            compressed_size: size,
            uncompressed_size: size,
            sha256: hex::encode(Sha256::digest(bytes)),
        }
    }
}

impl From<ResourceLimits> for ManifestLimits {
    fn from(value: ResourceLimits) -> Self {
        Self {
            max_archive_bytes: value.max_archive_bytes,
            max_total_uncompressed_bytes: value.max_total_uncompressed_bytes,
            max_entry_bytes: value.max_entry_bytes,
            max_compiled_ir_bytes: value.max_compiled_ir_bytes,
            max_manifest_bytes: value.max_manifest_bytes,
            max_entries: value.max_entries,
            max_path_bytes: value.max_path_bytes,
            max_semantic_depth: value.max_semantic_depth,
        }
    }
}

impl From<&ManifestLimits> for ResourceLimits {
    fn from(value: &ManifestLimits) -> Self {
        Self {
            max_archive_bytes: value.max_archive_bytes,
            max_total_uncompressed_bytes: value.max_total_uncompressed_bytes,
            max_entry_bytes: value.max_entry_bytes,
            max_compiled_ir_bytes: value.max_compiled_ir_bytes,
            max_manifest_bytes: value.max_manifest_bytes,
            max_entries: value.max_entries,
            max_path_bytes: value.max_path_bytes,
            max_semantic_depth: value.max_semantic_depth,
        }
    }
}
