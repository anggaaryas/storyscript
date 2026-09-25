use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use std::fs;

use prost::Message;
use storyscript_bundle_core::limits::ResourceLimits;
use storyscript_bundle_core::loader::{self, LoadedBundle, VerificationPolicy};
use storyscript_bundle_core::manifest::{BundleManifest, Compression, EntryType};
use storyscript_bundle_core::trust::TrustStore;
use storyscript_bundle_core::BundleError;

/// Host-provided Ed25519 trust material. The key must contain exactly 32 raw
/// public-key bytes. When present, `expected_key_id` must match its SHA-256 ID.
#[derive(Debug, Clone)]
pub struct BridgeTrustKey {
    pub public_key: Vec<u8>,
    pub expected_key_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeVerificationPolicy {
    Strict,
    UnsignedDevelopment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeLimits {
    pub max_archive_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_compiled_ir_bytes: u64,
    pub max_manifest_bytes: u64,
    pub max_entries: u32,
    pub max_path_bytes: u32,
    pub max_semantic_depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeProjectMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeManifestEntry {
    pub path: String,
    pub entry_type: String,
    pub compression: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeManifest {
    pub format_version: u32,
    pub compiler_version: String,
    pub schema_sha256: String,
    pub project: BridgeProjectMetadata,
    pub signer_key_id: String,
    pub resource_limits: BridgeLimits,
    pub entries: Vec<BridgeManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeVerificationStatus {
    pub signer_key_id: Option<String>,
    pub is_unsigned_development: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAssetDescriptor {
    pub logical_path: String,
    pub size: u64,
    pub sha256: String,
}

/// Opaque owner of verified archive bytes. The mutex serializes reads and
/// makes explicit disposal deterministic and idempotent.
#[derive(Clone)]
pub struct BundleResource {
    loaded: Arc<LoadedBundle>,
    active: Arc<AtomicBool>,
}

pub struct BridgeOpenedBundle {
    pub resource: BundleResource,
    pub manifest: BridgeManifest,
    pub verification: BridgeVerificationStatus,
    pub assets: Vec<BridgeAssetDescriptor>,
    pub compiled_story: Vec<u8>,
}

pub struct BridgeOpenResult {
    pub opened: Option<BridgeOpenedBundle>,
    pub error: Option<BridgeError>,
}

pub struct BridgeAssetReadResult {
    pub bytes: Option<Vec<u8>>,
    pub error: Option<BridgeError>,
}

pub struct BridgeDisposeResult {
    pub released: bool,
    pub error: Option<BridgeError>,
}

/// Opens untrusted bytes on FRB's asynchronous worker pool. No model or asset
/// data is returned unless the complete Rust verification pipeline succeeds.
pub fn bundle_open_bytes(
    bytes: Vec<u8>,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    limits: BridgeLimits,
) -> BridgeOpenResult {
    open_bytes_impl(bytes, trust_keys, policy, limits)
}

/// Native convenience API. Flutter Web callers must use `bundle_open_bytes`.
pub fn bundle_open_path(
    path: String,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    limits: BridgeLimits,
) -> BridgeOpenResult {
    #[cfg(target_family = "wasm")]
    {
        let _ = (path, trust_keys, policy, limits);
        return BridgeOpenResult::failure(BridgeError::new(
            "B_PATH_UNAVAILABLE",
            "filesystem paths are unavailable on Web; open bundle bytes instead",
        ));
    }

    #[cfg(not(target_family = "wasm"))]
    match fs::read(path) {
        Ok(bytes) => open_bytes_impl(bytes, trust_keys, policy, limits),
        Err(error) => BridgeOpenResult::failure(BridgeError::new("B_IO", error.to_string())),
    }
}

pub fn bundle_read_asset(
    resource: &BundleResource,
    logical_path: String,
    maximum_bytes: u64,
) -> BridgeAssetReadResult {
    let loaded = match resource.lease() {
        Ok(loaded) => loaded,
        Err(error) => {
            return BridgeAssetReadResult::failure(BridgeError::new(error.code, error.message));
        }
    };
    match loaded.read_asset(&logical_path, maximum_bytes) {
        Ok(bytes) => BridgeAssetReadResult {
            bytes: Some(bytes),
            error: None,
        },
        Err(error) => BridgeAssetReadResult::failure(error.into()),
    }
}

pub fn bundle_dispose(resource: &BundleResource) -> BridgeDisposeResult {
    BridgeDisposeResult {
        released: resource.active.swap(false, Ordering::AcqRel),
        error: None,
    }
}

pub fn bridge_hard_limits() -> BridgeLimits {
    ResourceLimits::HARD.into()
}

pub(crate) fn open_bytes_impl(
    bytes: Vec<u8>,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    limits: BridgeLimits,
) -> BridgeOpenResult {
    let requested_limits = match ResourceLimits::try_from(limits) {
        Ok(limits) => limits,
        Err(error) => return BridgeOpenResult::failure(error),
    };
    if bytes.len() as u64
        > requested_limits
            .max_archive_bytes
            .min(ResourceLimits::HARD.max_archive_bytes)
    {
        return BridgeOpenResult::failure(BridgeError::new(
            "B_RESOURCE_LIMIT",
            "bundle exceeds the requested archive-byte limit",
        ));
    }
    let trust_store = match build_trust_store(trust_keys) {
        Ok(store) => store,
        Err(error) => return BridgeOpenResult::failure(error),
    };
    let policy = match policy {
        BridgeVerificationPolicy::Strict => VerificationPolicy::Strict,
        BridgeVerificationPolicy::UnsignedDevelopment => VerificationPolicy::UnsignedDevelopment,
    };
    match loader::load(&bytes, &trust_store, policy, requested_limits) {
        Ok(loaded) => BridgeOpenResult::success(loaded),
        Err(error) => BridgeOpenResult::failure(error.into()),
    }
}

fn build_trust_store(keys: Vec<BridgeTrustKey>) -> Result<TrustStore, BridgeError> {
    let mut store = TrustStore::new();
    for key in keys {
        let bytes: [u8; 32] = key.public_key.try_into().map_err(|value: Vec<u8>| {
            BridgeError::new(
                "B_SIGNING_KEY_INVALID",
                format!(
                    "Ed25519 public key must contain 32 bytes, found {}",
                    value.len()
                ),
            )
        })?;
        match key.expected_key_id {
            Some(key_id) => store.insert(key_id, bytes).map_err(BridgeError::from)?,
            None => {
                let verifying_key =
                    ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|error| {
                        BridgeError::new("B_SIGNING_KEY_INVALID", error.to_string())
                    })?;
                store.insert_key(verifying_key);
            }
        }
    }
    Ok(store)
}

impl BridgeOpenResult {
    fn success(loaded: LoadedBundle) -> Self {
        let manifest = BridgeManifest::from(loaded.manifest());
        let verification = BridgeVerificationStatus {
            signer_key_id: loaded.verification_status().signer_key_id.clone(),
            is_unsigned_development: loaded.verification_status().is_unsigned_development,
        };
        let assets = loaded
            .assets()
            .iter()
            .map(|asset| BridgeAssetDescriptor {
                logical_path: asset.logical_path.clone(),
                size: asset.size,
                sha256: asset.sha256.clone(),
            })
            .collect();
        let compiled_story = loaded.story().encode_to_vec();
        Self {
            opened: Some(BridgeOpenedBundle {
                resource: BundleResource {
                    loaded: Arc::new(loaded),
                    active: Arc::new(AtomicBool::new(true)),
                },
                manifest,
                verification,
                assets,
                compiled_story,
            }),
            error: None,
        }
    }

    fn failure(error: BridgeError) -> Self {
        Self {
            opened: None,
            error: Some(error),
        }
    }
}

impl BridgeAssetReadResult {
    fn failure(error: BridgeError) -> Self {
        Self {
            bytes: None,
            error: Some(error),
        }
    }
}

impl BridgeError {
    pub(crate) fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl BundleResource {
    pub(crate) fn lease(&self) -> Result<Arc<LoadedBundle>, BridgeError> {
        if self.active.load(Ordering::Acquire) {
            Ok(Arc::clone(&self.loaded))
        } else {
            Err(BridgeError::new(
                "B_RESOURCE_DISPOSED",
                "bundle resource has been disposed",
            ))
        }
    }
}

impl From<BundleError> for BridgeError {
    fn from(error: BundleError) -> Self {
        Self::new(error.code().to_string(), error.to_string())
    }
}

impl From<ResourceLimits> for BridgeLimits {
    fn from(value: ResourceLimits) -> Self {
        Self {
            max_archive_bytes: value.max_archive_bytes,
            max_total_uncompressed_bytes: value.max_total_uncompressed_bytes,
            max_entry_bytes: value.max_entry_bytes,
            max_compiled_ir_bytes: value.max_compiled_ir_bytes,
            max_manifest_bytes: value.max_manifest_bytes,
            max_entries: value.max_entries as u32,
            max_path_bytes: value.max_path_bytes as u32,
            max_semantic_depth: value.max_semantic_depth as u32,
        }
    }
}

impl TryFrom<BridgeLimits> for ResourceLimits {
    type Error = BridgeError;

    fn try_from(value: BridgeLimits) -> Result<Self, Self::Error> {
        if value.max_archive_bytes == 0
            || value.max_total_uncompressed_bytes == 0
            || value.max_entry_bytes == 0
            || value.max_compiled_ir_bytes == 0
            || value.max_manifest_bytes == 0
            || value.max_entries == 0
            || value.max_path_bytes == 0
            || value.max_semantic_depth == 0
        {
            return Err(BridgeError::new(
                "B_RESOURCE_LIMIT",
                "resource limits must all be greater than zero",
            ));
        }
        Ok(ResourceLimits::HARD.lowered(ResourceLimits {
            max_archive_bytes: value.max_archive_bytes,
            max_total_uncompressed_bytes: value.max_total_uncompressed_bytes,
            max_entry_bytes: value.max_entry_bytes,
            max_compiled_ir_bytes: value.max_compiled_ir_bytes,
            max_manifest_bytes: value.max_manifest_bytes,
            max_entries: value.max_entries as usize,
            max_path_bytes: value.max_path_bytes as usize,
            max_semantic_depth: value.max_semantic_depth as usize,
        }))
    }
}

impl From<&BundleManifest> for BridgeManifest {
    fn from(value: &BundleManifest) -> Self {
        Self {
            format_version: value.format_version,
            compiler_version: value.compiler_version.clone(),
            schema_sha256: value.schema_sha256.clone(),
            project: BridgeProjectMetadata {
                id: value.project.id.clone(),
                name: value.project.name.clone(),
                version: value.project.version.clone(),
            },
            signer_key_id: value.signer_key_id.clone(),
            resource_limits: ResourceLimits::from(&value.resource_limits).into(),
            entries: value
                .entries
                .iter()
                .map(|entry| BridgeManifestEntry {
                    path: entry.path.clone(),
                    entry_type: match entry.entry_type {
                        EntryType::CompiledStory => "compiled_story",
                        EntryType::Asset => "asset",
                    }
                    .to_string(),
                    compression: match entry.compression {
                        Compression::Stored => "stored",
                    }
                    .to_string(),
                    compressed_size: entry.compressed_size,
                    uncompressed_size: entry.uncompressed_size,
                    sha256: entry.sha256.clone(),
                })
                .collect(),
        }
    }
}
