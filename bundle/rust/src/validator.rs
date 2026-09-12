use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::archive::reader::{ArchiveEnvelope, read_entry};
use crate::limits::ResourceLimits;
use crate::manifest::{
    BundleManifest, COMPILED_PATH, Compression, EntryType, MANIFEST_PATH, SIGNATURE_PATH,
};
use crate::proto::storybundle::v1 as pb;
use crate::{BundleError, COMPILER_VERSION, FORMAT_VERSION, Result, schema};

pub fn parse_manifest(
    archive: &[u8],
    envelope: &ArchiveEnvelope,
    limits: ResourceLimits,
) -> Result<(BundleManifest, Vec<u8>)> {
    let entry = envelope
        .entry(MANIFEST_PATH)
        .ok_or_else(|| BundleError::Manifest("manifest entry is missing".to_string()))?;
    let bytes = read_entry(archive, entry, limits.max_manifest_bytes)?;
    let manifest: BundleManifest =
        serde_json::from_slice(&bytes).map_err(|error| BundleError::Manifest(error.to_string()))?;
    if manifest.canonical_bytes()? != bytes {
        return Err(BundleError::Manifest(
            "manifest is not in canonical JSON form".to_string(),
        ));
    }
    Ok((manifest, bytes))
}

pub fn validate_manifest_contract(
    manifest: &BundleManifest,
    envelope: &ArchiveEnvelope,
    limits: ResourceLimits,
    signature_present: bool,
) -> Result<()> {
    validate_declared_limits(manifest)?;
    if envelope.archive_size > limits.max_archive_bytes {
        return Err(BundleError::Limit(
            "archive exceeds the signed resource profile".to_string(),
        ));
    }
    if envelope.entries.len() > limits.max_entries {
        return Err(BundleError::Limit(
            "entry count exceeds the signed resource profile".to_string(),
        ));
    }
    let manifest_entry = envelope
        .entry(MANIFEST_PATH)
        .ok_or_else(|| BundleError::Manifest("manifest entry is missing".to_string()))?;
    if manifest_entry.uncompressed_size > limits.max_manifest_bytes {
        return Err(BundleError::Limit(
            "manifest exceeds the signed resource profile".to_string(),
        ));
    }
    let total = envelope.entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.uncompressed_size)
            .ok_or_else(|| BundleError::Limit("declared size overflow".to_string()))
    })?;
    if total > limits.max_total_uncompressed_bytes {
        return Err(BundleError::Limit(
            "total payload exceeds the signed resource profile".to_string(),
        ));
    }
    if manifest.entries.len() + 1 + usize::from(signature_present) != envelope.entries.len() {
        return Err(BundleError::Manifest(
            "manifest does not cover the complete archive".to_string(),
        ));
    }

    let mut listed = HashSet::new();
    let mut previous = None;
    let mut compiled_count = 0;
    for entry in &manifest.entries {
        if previous.is_some_and(|value: &str| value >= entry.path.as_str()) {
            return Err(BundleError::Manifest(
                "manifest entries must be strictly path-sorted".to_string(),
            ));
        }
        previous = Some(&entry.path);
        if !listed.insert(entry.path.as_str()) {
            return Err(BundleError::PathCollision(entry.path.clone()));
        }
        if entry.compression != Compression::Stored {
            return Err(BundleError::UnsupportedFormat(format!(
                "entry '{}' uses unsupported compression",
                entry.path
            )));
        }
        match entry.entry_type {
            EntryType::CompiledStory if entry.path == COMPILED_PATH => compiled_count += 1,
            EntryType::Asset if entry.path.starts_with("assets/") => {}
            _ => {
                return Err(BundleError::Manifest(format!(
                    "entry '{}' has an invalid type/path pairing",
                    entry.path
                )));
            }
        }
        let actual = envelope.entry(&entry.path).ok_or_else(|| {
            BundleError::Manifest(format!("listed entry '{}' is missing", entry.path))
        })?;
        if entry.path.len() > limits.max_path_bytes {
            return Err(BundleError::Limit(format!(
                "entry '{}' exceeds the signed path limit",
                entry.path
            )));
        }
        if actual.compressed_size != entry.compressed_size
            || actual.uncompressed_size != entry.uncompressed_size
            || entry.compressed_size != entry.uncompressed_size
        {
            return Err(BundleError::Manifest(format!(
                "entry '{}' size/compression declaration differs from ZIP",
                entry.path
            )));
        }
        if entry.path == COMPILED_PATH && entry.uncompressed_size > limits.max_compiled_ir_bytes {
            return Err(BundleError::Limit("compiled IR exceeds limit".to_string()));
        }
        if entry.path != COMPILED_PATH && entry.uncompressed_size > limits.max_entry_bytes {
            return Err(BundleError::Limit(format!(
                "entry '{}' exceeds limit",
                entry.path
            )));
        }
        if entry.sha256.len() != 64
            || !entry
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(BundleError::Manifest(format!(
                "entry '{}' has an invalid SHA-256",
                entry.path
            )));
        }
    }
    if compiled_count != 1 {
        return Err(BundleError::Manifest(
            "manifest must contain exactly one compiled payload".to_string(),
        ));
    }

    let signer_id_valid = manifest.signer_key_id.len() == 64
        && manifest
            .signer_key_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !signer_id_valid && (signature_present || !manifest.signer_key_id.is_empty()) {
        return Err(BundleError::Manifest(
            "signer key ID must be lowercase SHA-256 hex".to_string(),
        ));
    }

    let allowed = manifest
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .chain([MANIFEST_PATH])
        .chain(signature_present.then_some(SIGNATURE_PATH))
        .collect::<HashSet<_>>();
    for entry in &envelope.entries {
        if !allowed.contains(entry.path.as_str()) {
            return Err(BundleError::Manifest(format!(
                "unknown or unlisted archive entry '{}'",
                entry.path
            )));
        }
    }
    Ok(())
}

pub fn validate_compatibility(manifest: &BundleManifest) -> Result<()> {
    if manifest.format_version != FORMAT_VERSION {
        return Err(BundleError::UnsupportedFormat(format!(
            "expected format {}, found {}",
            FORMAT_VERSION, manifest.format_version
        )));
    }
    if manifest.compiler_version != COMPILER_VERSION {
        return Err(BundleError::CompilerMismatch {
            expected: COMPILER_VERSION.to_string(),
            found: manifest.compiler_version.clone(),
        });
    }
    semver::Version::parse(&manifest.project.version).map_err(|error| {
        BundleError::Manifest(format!("project version is not SemVer: {error}"))
    })?;
    let expected_schema = schema::descriptor_sha256();
    if manifest.schema_sha256 != expected_schema {
        return Err(BundleError::SchemaMismatch {
            expected: expected_schema,
            found: manifest.schema_sha256.clone(),
        });
    }
    Ok(())
}

pub fn read_and_verify_entries(
    archive: &[u8],
    envelope: &ArchiveEnvelope,
    manifest: &BundleManifest,
    limits: ResourceLimits,
) -> Result<Vec<u8>> {
    let mut compiled = None;
    for listed in &manifest.entries {
        let envelope_entry = envelope.entry(&listed.path).ok_or_else(|| {
            BundleError::Manifest(format!("listed entry '{}' is missing", listed.path))
        })?;
        let maximum = if listed.path == COMPILED_PATH {
            limits.max_compiled_ir_bytes
        } else {
            limits.max_entry_bytes
        };
        let bytes = read_entry(archive, envelope_entry, maximum)?;
        let digest = hex::encode(Sha256::digest(&bytes));
        if digest != listed.sha256 {
            return Err(BundleError::DigestMismatch(listed.path.clone()));
        }
        if listed.path == COMPILED_PATH {
            compiled = Some(bytes);
        }
    }
    compiled.ok_or_else(|| BundleError::Manifest("compiled payload is missing".to_string()))
}

pub fn validate_semantic_depth(story: &pb::CompiledStory, maximum: usize) -> Result<()> {
    if let Some(initialization) = &story.initialization {
        for variable in &initialization.variables {
            if let Some(expression) = &variable.value {
                expression_depth(expression, 1, maximum)?;
            }
        }
    }
    for logic in &story.logic_blocks {
        prep_depth(&logic.body, 1, maximum)?;
    }
    for scene in &story.scenes {
        if let Some(prep) = &scene.prep {
            prep_depth(&prep.statements, 1, maximum)?;
        }
        if let Some(story) = &scene.story {
            story_depth(&story.statements, 1, maximum)?;
        }
    }
    Ok(())
}

fn expression_depth(value: &pb::Expression, depth: usize, maximum: usize) -> Result<()> {
    check_semantic_depth(depth, maximum)?;
    use pb::expression::Value;
    match value.value.as_ref() {
        Some(Value::Binary(binary)) => {
            if let Some(left) = &binary.left {
                expression_depth(left, depth + 1, maximum)?;
            }
            if let Some(right) = &binary.right {
                expression_depth(right, depth + 1, maximum)?;
            }
        }
        Some(Value::Call(call)) => {
            for argument in &call.arguments {
                expression_depth(argument, depth + 1, maximum)?;
            }
        }
        Some(Value::List(list)) => {
            for item in &list.items {
                expression_depth(item, depth + 1, maximum)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn prep_depth(values: &[pb::PrepStatement], depth: usize, maximum: usize) -> Result<()> {
    check_semantic_depth(depth, maximum)?;
    use pb::prep_statement::Value;
    for statement in values {
        match statement.value.as_ref() {
            Some(Value::VariableDefinition(value)) => {
                if let Some(expression) = &value.value {
                    expression_depth(expression, depth + 1, maximum)?;
                }
            }
            Some(Value::VariableAssignment(value)) => {
                if let Some(expression) = &value.value {
                    expression_depth(expression, depth + 1, maximum)?;
                }
            }
            Some(Value::Call(call)) => {
                for argument in &call.arguments {
                    expression_depth(argument, depth + 1, maximum)?;
                }
            }
            Some(Value::IfElse(value)) => {
                if let Some(condition) = &value.condition {
                    expression_depth(condition, depth + 1, maximum)?;
                }
                if let Some(branch) = &value.then_branch {
                    prep_depth(&branch.statements, depth + 1, maximum)?;
                }
                if let Some(branch) = &value.else_branch {
                    prep_depth(&branch.statements, depth + 1, maximum)?;
                }
            }
            Some(Value::ForSnapshot(value)) => prep_depth(&value.body, depth + 1, maximum)?,
            Some(Value::Repeat(value)) => prep_depth(&value.body, depth + 1, maximum)?,
            Some(Value::ReturnStatement(value)) => {
                if let Some(expression) = &value.value {
                    expression_depth(expression, depth + 1, maximum)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn story_depth(values: &[pb::StoryStatement], depth: usize, maximum: usize) -> Result<()> {
    check_semantic_depth(depth, maximum)?;
    use pb::story_statement::Value;
    for statement in values {
        match statement.value.as_ref() {
            Some(Value::IfElse(value)) => {
                if let Some(condition) = &value.condition {
                    expression_depth(condition, depth + 1, maximum)?;
                }
                if let Some(branch) = &value.then_branch {
                    story_depth(&branch.statements, depth + 1, maximum)?;
                }
                if let Some(branch) = &value.else_branch {
                    story_depth(&branch.statements, depth + 1, maximum)?;
                }
            }
            Some(Value::Choice(value)) => choice_depth(&value.entries, depth + 1, maximum)?,
            Some(Value::ForSnapshot(value)) => story_depth(&value.body, depth + 1, maximum)?,
            Some(Value::Repeat(value)) => story_depth(&value.body, depth + 1, maximum)?,
            _ => {}
        }
    }
    Ok(())
}

fn choice_depth(values: &[pb::ChoiceEntry], depth: usize, maximum: usize) -> Result<()> {
    check_semantic_depth(depth, maximum)?;
    use pb::choice_entry::Value;
    for entry in values {
        match entry.value.as_ref() {
            Some(Value::IfEntry(value)) => {
                if let Some(condition) = &value.condition {
                    expression_depth(condition, depth + 1, maximum)?;
                }
                if let Some(body) = &value.body {
                    choice_depth(&body.entries, depth + 1, maximum)?;
                }
            }
            Some(Value::Repeat(value)) => {
                if let Some(body) = &value.body {
                    choice_depth(&body.entries, depth + 1, maximum)?;
                }
            }
            Some(Value::ForSnapshot(value)) => {
                if let Some(body) = &value.body {
                    choice_depth(&body.entries, depth + 1, maximum)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_semantic_depth(depth: usize, maximum: usize) -> Result<()> {
    if depth > maximum {
        return Err(BundleError::Limit(format!(
            "semantic nesting exceeds {maximum}"
        )));
    }
    Ok(())
}

fn validate_declared_limits(manifest: &BundleManifest) -> Result<()> {
    let declared = &manifest.resource_limits;
    let hard = ResourceLimits::HARD;
    if declared.max_archive_bytes > hard.max_archive_bytes
        || declared.max_total_uncompressed_bytes > hard.max_total_uncompressed_bytes
        || declared.max_entry_bytes > hard.max_entry_bytes
        || declared.max_compiled_ir_bytes > hard.max_compiled_ir_bytes
        || declared.max_manifest_bytes > hard.max_manifest_bytes
        || declared.max_entries > hard.max_entries
        || declared.max_path_bytes > hard.max_path_bytes
        || declared.max_semantic_depth > hard.max_semantic_depth
    {
        return Err(BundleError::Limit(
            "manifest resource profile exceeds hard limits".to_string(),
        ));
    }
    Ok(())
}
