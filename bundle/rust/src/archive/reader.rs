use std::collections::HashMap;
use std::io::{Cursor, Read};

use unicode_normalization::UnicodeNormalization;
use zip::{CompressionMethod, ZipArchive};

use crate::limits::ResourceLimits;
use crate::manifest::{MANIFEST_PATH, SIGNATURE_PATH};
use crate::{BundleError, Result};

#[derive(Debug, Clone)]
pub struct EnvelopeEntry {
    pub index: usize,
    pub path: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Debug, Clone)]
pub struct ArchiveEnvelope {
    pub archive_size: u64,
    pub entries: Vec<EnvelopeEntry>,
}

impl ArchiveEnvelope {
    pub fn entry(&self, path: &str) -> Option<&EnvelopeEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }
}

pub fn preflight(bytes: &[u8], limits: ResourceLimits) -> Result<ArchiveEnvelope> {
    if bytes.len() as u64 > limits.max_archive_bytes {
        return Err(BundleError::Limit(format!(
            "archive exceeds {} bytes",
            limits.max_archive_bytes
        )));
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| BundleError::Archive(error.to_string()))?;
    if !archive.comment().is_empty() {
        return Err(BundleError::Archive(
            "ZIP comments are forbidden".to_string(),
        ));
    }
    let declared_count = declared_entry_count(bytes)?;
    if declared_count != archive.len() {
        return Err(BundleError::PathCollision(format!(
            "ZIP declares {declared_count} entries but exposes {}; duplicate names are forbidden",
            archive.len()
        )));
    }
    if archive.len() > limits.max_entries {
        return Err(BundleError::Limit(format!(
            "archive has {} entries; maximum is {}",
            archive.len(),
            limits.max_entries
        )));
    }

    let mut entries = Vec::with_capacity(archive.len());
    let mut exact = HashMap::new();
    let mut folded = HashMap::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| BundleError::Archive(error.to_string()))?;
        let path = validate_name(entry.name_raw(), limits.max_path_bytes)?;
        if entry.encrypted() {
            return Err(BundleError::Archive(format!(
                "encrypted entry '{path}' is forbidden"
            )));
        }
        if !entry.comment().is_empty() || entry.extra_data().is_some_and(|data| !data.is_empty()) {
            return Err(BundleError::Archive(format!(
                "entry '{path}' contains forbidden comments or extra fields"
            )));
        }
        if entry.is_dir() {
            return Err(BundleError::Archive(format!(
                "directory entry '{path}' is forbidden"
            )));
        }
        if let Some(mode) = entry.unix_mode() {
            let kind = mode & 0o170000;
            if kind != 0 && kind != 0o100000 {
                return Err(BundleError::Archive(format!(
                    "special entry '{path}' is forbidden"
                )));
            }
        }
        if entry.compression() != CompressionMethod::Stored {
            return Err(BundleError::Archive(format!(
                "entry '{path}' uses unsupported compression"
            )));
        }
        if entry.size() > limits.max_entry_bytes
            && path != MANIFEST_PATH
            && path != "compiled/story.pb"
        {
            return Err(BundleError::Limit(format!("entry '{path}' is too large")));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| BundleError::Limit("declared size overflow".to_string()))?;
        if total > limits.max_total_uncompressed_bytes {
            return Err(BundleError::Limit(
                "declared total uncompressed size exceeds limit".to_string(),
            ));
        }
        if exact.insert(path.clone(), index).is_some() {
            return Err(BundleError::PathCollision(format!(
                "duplicate entry '{path}'"
            )));
        }
        let case_key = path.to_lowercase();
        if let Some(existing) = folded.insert(case_key, path.clone())
            && existing != path
        {
            return Err(BundleError::PathCollision(format!(
                "case-folded collision between '{existing}' and '{path}'"
            )));
        }
        entries.push(EnvelopeEntry {
            index,
            path,
            compressed_size: entry.compressed_size(),
            uncompressed_size: entry.size(),
        });
    }
    if entries.len() != declared_count {
        return Err(BundleError::PathCollision(format!(
            "ZIP directory collapsed from {declared_count} to {} entries; duplicate names are forbidden",
            entries.len()
        )));
    }
    if !exact.contains_key(MANIFEST_PATH) {
        return Err(BundleError::Manifest(
            "manifest entry is missing".to_string(),
        ));
    }
    Ok(ArchiveEnvelope {
        archive_size: bytes.len() as u64,
        entries,
    })
}

fn declared_entry_count(bytes: &[u8]) -> Result<usize> {
    const EOCD_SIGNATURE: &[u8; 4] = b"PK\x05\x06";
    let offset = bytes
        .windows(EOCD_SIGNATURE.len())
        .rposition(|window| window == EOCD_SIGNATURE)
        .ok_or_else(|| {
            BundleError::Archive("ZIP end-of-directory record is missing".to_string())
        })?;
    let count_bytes = bytes.get(offset + 10..offset + 12).ok_or_else(|| {
        BundleError::Archive("ZIP end-of-directory record is truncated".to_string())
    })?;
    let count = u16::from_le_bytes([count_bytes[0], count_bytes[1]]);
    if count == u16::MAX {
        return Err(BundleError::Archive(
            "ZIP64 entry directories are forbidden by the v1 resource profile".to_string(),
        ));
    }
    let comment_length = bytes
        .get(offset + 20..offset + 22)
        .map(|value| u16::from_le_bytes([value[0], value[1]]) as usize)
        .ok_or_else(|| {
            BundleError::Archive("ZIP end-of-directory record is truncated".to_string())
        })?;
    if comment_length != 0 || offset + 22 != bytes.len() {
        return Err(BundleError::Archive(
            "ZIP comments or trailing bytes are forbidden".to_string(),
        ));
    }
    Ok(count as usize)
}

pub fn read_entry(bytes: &[u8], entry: &EnvelopeEntry, maximum: u64) -> Result<Vec<u8>> {
    if entry.uncompressed_size > maximum {
        return Err(BundleError::Limit(format!(
            "entry '{}' exceeds {} bytes",
            entry.path, maximum
        )));
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| BundleError::Archive(error.to_string()))?;
    let mut source = archive
        .by_index(entry.index)
        .map_err(|error| BundleError::Archive(error.to_string()))?;
    let mut output = Vec::with_capacity(entry.uncompressed_size as usize);
    source
        .by_ref()
        .take(maximum.saturating_add(1))
        .read_to_end(&mut output)?;
    if output.len() as u64 > maximum {
        return Err(BundleError::Limit(format!(
            "entry '{}' expanded beyond its limit",
            entry.path
        )));
    }
    Ok(output)
}

pub fn signature_entry(envelope: &ArchiveEnvelope) -> Option<&EnvelopeEntry> {
    envelope.entry(SIGNATURE_PATH)
}

fn validate_name(raw: &[u8], max_bytes: usize) -> Result<String> {
    if raw.len() > max_bytes {
        return Err(BundleError::Limit(
            "archive path exceeds path limit".to_string(),
        ));
    }
    let path = std::str::from_utf8(raw)
        .map_err(|_| BundleError::Archive("archive path is not UTF-8".to_string()))?;
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(BundleError::Archive(format!(
            "archive path '{path}' is not normalized"
        )));
    }
    let normalized = path.nfc().collect::<String>();
    if normalized != path {
        return Err(BundleError::Archive(format!(
            "archive path '{path}' is not NFC-normalized"
        )));
    }
    let lowercase = path.to_lowercase();
    if lowercase.ends_with(".storyscript") || lowercase.ends_with(".storyscript.map") {
        return Err(BundleError::Archive(
            "raw StoryScript source and source-map entries are forbidden".to_string(),
        ));
    }
    Ok(path.to_string())
}
