use ed25519_dalek::{Signature, Verifier};
use prost::Message;

use crate::archive::reader::{self, ArchiveEnvelope};
use crate::assets;
use crate::contract;
use crate::limits::ResourceLimits;
use crate::manifest::{BundleManifest, EntryType};
use crate::proto::storybundle::v1::CompiledStory;
use crate::signing::signing_digest;
use crate::trust::TrustStore;
use crate::validator;
use crate::{BundleError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerificationPolicy {
    #[default]
    Strict,
    UnsignedDevelopment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationStatus {
    pub signer_key_id: Option<String>,
    pub is_unsigned_development: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDescriptor {
    pub logical_path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug)]
pub struct LoadedBundle {
    archive: Vec<u8>,
    envelope: ArchiveEnvelope,
    manifest: BundleManifest,
    story: CompiledStory,
    status: VerificationStatus,
    assets: Vec<AssetDescriptor>,
    catalogs: Vec<CatalogDescriptor>,
    limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub struct UntrustedInspection {
    pub archive_size: u64,
    pub entry_paths: Vec<String>,
    pub format_version: Option<u32>,
    pub project_id: Option<String>,
    pub signer_key_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDescriptor {
    pub locale: String,
    pub size: u64,
    pub sha256: String,
}

pub fn load(
    bytes: &[u8],
    trust_store: &TrustStore,
    policy: VerificationPolicy,
    requested_limits: ResourceLimits,
) -> Result<LoadedBundle> {
    let mut limits = ResourceLimits::HARD.lowered(requested_limits);
    let envelope = reader::preflight(bytes, limits)?;
    let (manifest, manifest_bytes) = validator::parse_manifest(bytes, &envelope, limits)?;
    limits = limits.lowered((&manifest.resource_limits).into());
    let signature_entry = reader::signature_entry(&envelope);
    validator::validate_manifest_contract(&manifest, &envelope, limits, signature_entry.is_some())?;
    let status = verify_signature(
        bytes,
        &envelope,
        &manifest,
        &manifest_bytes,
        trust_store,
        policy,
    )?;
    validator::validate_compatibility(&manifest)?;
    let compiled = validator::read_and_verify_entries(bytes, &envelope, &manifest, limits)?;
    let story = CompiledStory::decode(compiled.as_slice())
        .map_err(|error| BundleError::ProtobufDecode(error.to_string()))?;
    validator::validate_semantic_depth(&story, limits.max_semantic_depth)?;
    contract::validate_story(&story)
        .map_err(|error| BundleError::SemanticViolation(error.to_string()))?;
    let story_project = story
        .project
        .as_ref()
        .ok_or_else(|| BundleError::SemanticViolation("project is missing".to_string()))?;
    if story.format_version != manifest.format_version
        || story_project.id != manifest.project.id
        || story_project.name != manifest.project.name
        || story_project.version != manifest.project.version
    {
        return Err(BundleError::SemanticViolation(
            "compiled project metadata differs from the signed manifest".to_string(),
        ));
    }
    let assets = manifest
        .entries
        .iter()
        .filter(|entry| entry.entry_type == EntryType::Asset)
        .map(|entry| AssetDescriptor {
            logical_path: entry.path.trim_start_matches("assets/").to_string(),
            size: entry.uncompressed_size,
            sha256: entry.sha256.clone(),
        })
        .collect::<Vec<_>>();
    assets::validate_closure(
        &story,
        assets.iter().map(|asset| asset.logical_path.as_str()),
    )?;
    let contracts = crate::localization::message_contracts(&story)?;
    let catalog_entries: Vec<_> = manifest
        .entries
        .iter()
        .filter(|e| e.entry_type == EntryType::Catalog)
        .collect();
    let expected: std::collections::BTreeSet<_> = story
        .localization
        .as_ref()
        .map(|v| v.supported_locales.iter().cloned().collect())
        .unwrap_or_default();
    if catalog_entries.len() != expected.len() || catalog_entries.len() > crate::limits::MAX_LOCALES
    {
        return Err(BundleError::SemanticViolation(
            "catalog entries must exactly cover supported locales".into(),
        ));
    }
    let mut catalogs = Vec::new();
    for listed in catalog_entries {
        let locale = crate::localization::catalog_locale(&listed.path)?;
        if !expected.contains(&locale) {
            return Err(BundleError::SemanticViolation(
                "unknown catalog locale".into(),
            ));
        }
        let entry = envelope
            .entry(&listed.path)
            .expect("validated listed entry");
        let data = reader::read_entry(
            bytes,
            entry,
            crate::limits::MAX_CATALOG_BYTES.min(limits.max_entry_bytes),
        )?;
        let source = std::str::from_utf8(&data)
            .map_err(|_| BundleError::Contract(format!("{}: catalog is not UTF-8", listed.path)))?;
        let catalog = crate::localization::parse_catalog(&locale, source, &listed.path)?;
        if catalog.canonical != source {
            return Err(BundleError::Contract(format!(
                "{}: catalog is not canonical semantic FTL",
                listed.path
            )));
        }
        crate::localization::validate_catalog_contract(&catalog, &contracts, &listed.path)?;
        catalogs.push(CatalogDescriptor {
            locale,
            size: listed.uncompressed_size,
            sha256: listed.sha256.clone(),
        });
    }

    Ok(LoadedBundle {
        archive: bytes.to_vec(),
        envelope,
        manifest,
        story,
        status,
        assets,
        catalogs,
        limits,
    })
}

pub fn inspect(bytes: &[u8], requested_limits: ResourceLimits) -> Result<UntrustedInspection> {
    let limits = ResourceLimits::HARD.lowered(requested_limits);
    let envelope = reader::preflight(bytes, limits)?;
    let manifest = validator::parse_manifest(bytes, &envelope, limits)
        .map(|(manifest, _)| manifest)
        .ok();
    Ok(UntrustedInspection {
        archive_size: envelope.archive_size,
        entry_paths: envelope
            .entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect(),
        format_version: manifest.as_ref().map(|value| value.format_version),
        project_id: manifest.as_ref().map(|value| value.project.id.clone()),
        signer_key_id: manifest.map(|value| value.signer_key_id),
    })
}

impl LoadedBundle {
    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    pub fn story(&self) -> &CompiledStory {
        &self.story
    }

    pub fn verification_status(&self) -> &VerificationStatus {
        &self.status
    }

    /// Digest authenticated by the manifest for the validated compiled model.
    pub fn compiled_entry_sha256(&self) -> &str {
        self.manifest
            .entries
            .iter()
            .find(|entry| entry.path == crate::manifest::COMPILED_PATH)
            .map(|entry| entry.sha256.as_str())
            .expect("validated bundle always has compiled/story.pb")
    }

    pub fn assets(&self) -> &[AssetDescriptor] {
        &self.assets
    }

    pub fn catalogs(&self) -> &[CatalogDescriptor] {
        &self.catalogs
    }

    /// Only available through the completely verified bundle capability. Returns
    /// one bounded catalog; generic model loading never copies all bodies to Dart.
    pub fn read_catalog(&self, locale: &str, maximum_bytes: u64) -> Result<String> {
        let descriptor = self
            .catalogs
            .iter()
            .find(|v| v.locale == locale)
            .ok_or_else(|| BundleError::Contract("unsupported catalog locale".into()))?;
        let maximum = maximum_bytes
            .min(crate::limits::MAX_CATALOG_BYTES)
            .min(self.limits.max_entry_bytes);
        if descriptor.size > maximum {
            return Err(BundleError::Limit(
                "catalog exceeds requested read limit".into(),
            ));
        }
        let path = format!("{}{locale}.ftl", crate::manifest::CATALOG_PREFIX);
        let bytes = reader::read_entry(
            &self.archive,
            self.envelope.entry(&path).expect("verified catalog entry"),
            maximum,
        )?;
        use sha2::{Digest, Sha256};
        if hex::encode(Sha256::digest(&bytes)) != descriptor.sha256 {
            return Err(BundleError::DigestMismatch(path));
        }
        String::from_utf8(bytes).map_err(|_| BundleError::Contract("catalog is not UTF-8".into()))
    }

    pub fn read_asset(&self, logical_path: &str, maximum_bytes: u64) -> Result<Vec<u8>> {
        let descriptor = self
            .assets
            .iter()
            .find(|asset| asset.logical_path == logical_path)
            .ok_or_else(|| BundleError::MissingAsset(logical_path.to_string()))?;
        let maximum = maximum_bytes.min(self.limits.max_entry_bytes);
        if descriptor.size > maximum {
            return Err(BundleError::Limit(format!(
                "asset '{logical_path}' exceeds requested read limit"
            )));
        }
        let path = format!("assets/{logical_path}");
        let entry = self
            .envelope
            .entry(&path)
            .ok_or_else(|| BundleError::MissingAsset(logical_path.to_string()))?;
        let bytes = reader::read_entry(&self.archive, entry, maximum)?;
        let expected = &descriptor.sha256;
        let actual = {
            use sha2::{Digest, Sha256};
            hex::encode(Sha256::digest(&bytes))
        };
        if actual != *expected {
            return Err(BundleError::DigestMismatch(path));
        }
        Ok(bytes)
    }
}

fn verify_signature(
    archive: &[u8],
    envelope: &ArchiveEnvelope,
    manifest: &BundleManifest,
    manifest_bytes: &[u8],
    trust_store: &TrustStore,
    policy: VerificationPolicy,
) -> Result<VerificationStatus> {
    let signature_entry = reader::signature_entry(envelope);
    if signature_entry.is_none() {
        return match policy {
            VerificationPolicy::Strict => Err(BundleError::UnsignedDisallowed(
                "signature entry is missing".to_string(),
            )),
            VerificationPolicy::UnsignedDevelopment => Ok(VerificationStatus {
                signer_key_id: None,
                is_unsigned_development: true,
            }),
        };
    }
    if manifest.signer_key_id.is_empty() {
        return Err(BundleError::BadSignature(
            "signed manifest has no signer key ID".to_string(),
        ));
    }
    let key = trust_store
        .get(&manifest.signer_key_id)
        .ok_or_else(|| BundleError::UnknownSigner(manifest.signer_key_id.clone()))?;
    let signature_bytes = reader::read_entry(archive, signature_entry.unwrap(), 64)?;
    let signature_array: [u8; 64] = signature_bytes.try_into().map_err(|_| {
        BundleError::BadSignature("signature entry must contain exactly 64 bytes".to_string())
    })?;
    key.verify(
        &signing_digest(manifest_bytes),
        &Signature::from_bytes(&signature_array),
    )
    .map_err(|_| BundleError::BadSignature("Ed25519 verification failed".to_string()))?;
    Ok(VerificationStatus {
        signer_key_id: Some(manifest.signer_key_id.clone()),
        is_unsigned_development: false,
    })
}
