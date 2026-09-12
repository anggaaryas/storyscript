use std::fs;
use std::path::{Path, PathBuf};

use prost::Message;
use zeroize::Zeroize;

use crate::archive::writer::{self, ArchiveEntry};
use crate::limits::ResourceLimits;
use crate::manifest::{BundleManifest, COMPILED_PATH};
use crate::project;
use crate::signing::{BundleSigner, Ed25519Signer};
use crate::{BundleError, Result};

pub fn export(project_root: &Path, signer: &dyn BundleSigner) -> Result<Vec<u8>> {
    let compiled = project::compile(project_root)?;
    let limits = ResourceLimits::HARD;
    let story_bytes = compiled.story.encode_to_vec();
    enforce_payload_limits(&story_bytes, &compiled.assets, limits)?;

    let assets = compiled
        .assets
        .iter()
        .map(|asset| (asset.logical_path.clone(), asset.bytes.clone()))
        .collect::<Vec<_>>();
    let manifest = BundleManifest::new(
        &compiled.config.project,
        signer.key_id(),
        &story_bytes,
        &assets,
        limits,
    );
    let manifest_bytes = manifest.canonical_bytes()?;
    if manifest_bytes.len() as u64 > limits.max_manifest_bytes {
        return Err(BundleError::Limit("manifest exceeds 1 MiB".to_string()));
    }
    let signature = signer.sign_manifest(&manifest_bytes)?;

    let mut owned_entries = Vec::with_capacity(assets.len() + 1);
    owned_entries.push((COMPILED_PATH.to_string(), story_bytes.as_slice()));
    owned_entries.extend(
        assets
            .iter()
            .map(|(path, bytes)| (format!("assets/{path}"), bytes.as_slice())),
    );
    let mut archive_entries = owned_entries
        .iter()
        .map(|(path, bytes)| ArchiveEntry {
            path: path.as_str(),
            bytes,
        })
        .collect::<Vec<_>>();
    archive_entries.sort_by(|left, right| left.path.cmp(right.path));
    let bytes = writer::write(&manifest_bytes, &signature, &archive_entries)?;
    if bytes.len() as u64 > limits.max_archive_bytes {
        return Err(BundleError::Limit("archive exceeds 100 MiB".to_string()));
    }
    Ok(bytes)
}

pub fn export_with_pkcs8_file(
    project_root: &Path,
    output: &Path,
    signing_key: &Path,
) -> Result<()> {
    require_extension(output)?;
    reject_project_local_key(project_root, signing_key)?;
    let mut pem = fs::read(signing_key)?;
    let pem_text = std::str::from_utf8(&pem)
        .map_err(|_| BundleError::SigningKey("PKCS#8 PEM is not UTF-8".to_string()))?;
    let signer = Ed25519Signer::from_pkcs8_pem(pem_text);
    pem.zeroize();
    let bytes = export(project_root, &signer?)?;
    fs::write(output, bytes)?;
    Ok(())
}

fn enforce_payload_limits(
    compiled: &[u8],
    assets: &[crate::assets::Asset],
    limits: ResourceLimits,
) -> Result<()> {
    if compiled.len() as u64 > limits.max_compiled_ir_bytes {
        return Err(BundleError::Limit("compiled IR exceeds 16 MiB".to_string()));
    }
    if assets.len() + 3 > limits.max_entries {
        return Err(BundleError::Limit(
            "archive has too many entries".to_string(),
        ));
    }
    let mut total = compiled.len() as u64;
    for asset in assets {
        if asset.logical_path.len() > limits.max_path_bytes {
            return Err(BundleError::Limit(format!(
                "asset path '{}' exceeds path limit",
                asset.logical_path
            )));
        }
        if asset.bytes.len() as u64 > limits.max_entry_bytes {
            return Err(BundleError::Limit(format!(
                "asset '{}' exceeds 64 MiB",
                asset.logical_path
            )));
        }
        total = total
            .checked_add(asset.bytes.len() as u64)
            .ok_or_else(|| BundleError::Limit("uncompressed size overflow".to_string()))?;
    }
    if total > limits.max_total_uncompressed_bytes {
        return Err(BundleError::Limit(
            "total uncompressed payload exceeds 100 MiB".to_string(),
        ));
    }
    Ok(())
}

fn require_extension(output: &Path) -> Result<()> {
    if output.extension().and_then(|value| value.to_str()) != Some("storybundle") {
        return Err(BundleError::Output(
            "output path must use the .storybundle extension".to_string(),
        ));
    }
    Ok(())
}

fn reject_project_local_key(project_root: &Path, key: &Path) -> Result<()> {
    let project = project_root.canonicalize()?;
    let key = key.canonicalize()?;
    let repository = repository_root(&project);
    if key.starts_with(&project) || repository.is_some_and(|root| key.starts_with(root)) {
        return Err(BundleError::SigningKey(
            "signing key must be outside the project and repository root".to_string(),
        ));
    }
    Ok(())
}

fn repository_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| ancestor.join(".git").exists())
        .map(Path::to_path_buf)
}
