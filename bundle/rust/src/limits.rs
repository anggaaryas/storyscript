#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    pub max_archive_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_compiled_ir_bytes: u64,
    pub max_manifest_bytes: u64,
    pub max_entries: usize,
    pub max_path_bytes: usize,
    pub max_semantic_depth: usize,
}

impl ResourceLimits {
    pub const HARD: Self = Self {
        max_archive_bytes: 100 * 1024 * 1024,
        max_total_uncompressed_bytes: 100 * 1024 * 1024,
        max_entry_bytes: 64 * 1024 * 1024,
        max_compiled_ir_bytes: 16 * 1024 * 1024,
        max_manifest_bytes: 1024 * 1024,
        max_entries: 4_096,
        max_path_bytes: 1_024,
        max_semantic_depth: 128,
    };

    pub fn lowered(self, requested: Self) -> Self {
        Self {
            max_archive_bytes: self.max_archive_bytes.min(requested.max_archive_bytes),
            max_total_uncompressed_bytes: self
                .max_total_uncompressed_bytes
                .min(requested.max_total_uncompressed_bytes),
            max_entry_bytes: self.max_entry_bytes.min(requested.max_entry_bytes),
            max_compiled_ir_bytes: self
                .max_compiled_ir_bytes
                .min(requested.max_compiled_ir_bytes),
            max_manifest_bytes: self.max_manifest_bytes.min(requested.max_manifest_bytes),
            max_entries: self.max_entries.min(requested.max_entries),
            max_path_bytes: self.max_path_bytes.min(requested.max_path_bytes),
            max_semantic_depth: self.max_semantic_depth.min(requested.max_semantic_depth),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::HARD
    }
}
