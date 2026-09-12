use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use crate::manifest::{MANIFEST_PATH, SIGNATURE_PATH};
use crate::{BundleError, Result};

pub struct ArchiveEntry<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

pub fn write(
    manifest: &[u8],
    signature: &[u8; 64],
    entries: &[ArchiveEntry<'_>],
) -> Result<Vec<u8>> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(DateTime::default())
            .unix_permissions(0o644);
        write_entry(&mut archive, MANIFEST_PATH, manifest, options)?;
        write_entry(&mut archive, SIGNATURE_PATH, signature, options)?;
        for entry in entries {
            write_entry(&mut archive, entry.path, entry.bytes, options)?;
        }
        archive
            .finish()
            .map_err(|error| BundleError::Archive(error.to_string()))?;
    }
    Ok(output.into_inner())
}

fn write_entry<W: Write + std::io::Seek>(
    archive: &mut ZipWriter<W>,
    path: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> Result<()> {
    archive
        .start_file(path, options)
        .map_err(|error| BundleError::Archive(error.to_string()))?;
    archive.write_all(bytes)?;
    Ok(())
}
