use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ConfigInvalid,
    ProjectInvalid,
    CompileFailed,
    AssetInvalid,
    SigningKeyInvalid,
    SigningFailed,
    ArchiveInvalid,
    OutputInvalid,
    PathCollision,
    ResourceLimit,
    ManifestMalformed,
    UnknownSigner,
    BadSignature,
    DigestMismatch,
    UnsupportedFormat,
    ProtobufDecode,
    SemanticViolation,
    MissingAsset,
    UnsignedDisallowed,
    CompilerMismatch,
    SchemaMismatch,
    ContractViolation,
    Io,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ConfigInvalid => "B_CONFIG_INVALID",
            Self::ProjectInvalid => "B_PROJECT_INVALID",
            Self::CompileFailed => "B_COMPILE_FAILED",
            Self::AssetInvalid => "B_ASSET_INVALID",
            Self::SigningKeyInvalid => "B_SIGNING_KEY_INVALID",
            Self::SigningFailed => "B_SIGNING_FAILED",
            Self::ArchiveInvalid => "B_ARCHIVE_INVALID",
            Self::OutputInvalid => "B_OUTPUT_INVALID",
            Self::PathCollision => "B_PATH_COLLISION",
            Self::ResourceLimit => "B_RESOURCE_LIMIT",
            Self::ManifestMalformed => "B_MANIFEST_MALFORMED",
            Self::UnknownSigner => "B_UNKNOWN_SIGNER",
            Self::BadSignature => "B_BAD_SIGNATURE",
            Self::DigestMismatch => "B_DIGEST_MISMATCH",
            Self::UnsupportedFormat => "B_UNSUPPORTED_FORMAT",
            Self::ProtobufDecode => "B_PROTOBUF_DECODE",
            Self::SemanticViolation => "B_SEMANTIC_VIOLATION",
            Self::MissingAsset => "B_MISSING_ASSET",
            Self::UnsignedDisallowed => "B_UNSIGNED_DISALLOWED",
            Self::CompilerMismatch => "B_COMPILER_MISMATCH",
            Self::SchemaMismatch => "B_SCHEMA_MISMATCH",
            Self::ContractViolation => "B_CONTRACT_VIOLATION",
            Self::Io => "B_IO",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("invalid StoryScript project configuration: {0}")]
    Config(String),
    #[error("invalid StoryScript project: {0}")]
    Project(String),
    #[error("StoryScript compilation failed: {0}")]
    Compile(String),
    #[error("invalid project asset graph: {0}")]
    Asset(String),
    #[error("invalid signing key: {0}")]
    SigningKey(String),
    #[error("bundle signing failed: {0}")]
    Signing(String),
    #[error("archive failure: {0}")]
    Archive(String),
    #[error("invalid output: {0}")]
    Output(String),
    #[error("archive path collision: {0}")]
    PathCollision(String),
    #[error("compiler version mismatch: expected {expected}, found {found}")]
    CompilerMismatch { expected: String, found: String },
    #[error("schema fingerprint mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },
    #[error("semantic contract violation: {0}")]
    Contract(String),
    #[error("resource limit exceeded: {0}")]
    Limit(String),
    #[error("malformed manifest: {0}")]
    Manifest(String),
    #[error("unknown signer: {0}")]
    UnknownSigner(String),
    #[error("bad signature: {0}")]
    BadSignature(String),
    #[error("entry digest mismatch: {0}")]
    DigestMismatch(String),
    #[error("unsupported bundle format: {0}")]
    UnsupportedFormat(String),
    #[error("Protobuf decode failed: {0}")]
    ProtobufDecode(String),
    #[error("semantic IR violation: {0}")]
    SemanticViolation(String),
    #[error("required asset is missing: {0}")]
    MissingAsset(String),
    #[error("unsigned bundle is disallowed: {0}")]
    UnsignedDisallowed(String),
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
}

impl BundleError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Config(_) => ErrorCode::ConfigInvalid,
            Self::Project(_) => ErrorCode::ProjectInvalid,
            Self::Compile(_) => ErrorCode::CompileFailed,
            Self::Asset(_) => ErrorCode::AssetInvalid,
            Self::SigningKey(_) => ErrorCode::SigningKeyInvalid,
            Self::Signing(_) => ErrorCode::SigningFailed,
            Self::Archive(_) => ErrorCode::ArchiveInvalid,
            Self::Output(_) => ErrorCode::OutputInvalid,
            Self::PathCollision(_) => ErrorCode::PathCollision,
            Self::CompilerMismatch { .. } => ErrorCode::CompilerMismatch,
            Self::SchemaMismatch { .. } => ErrorCode::SchemaMismatch,
            Self::Contract(_) => ErrorCode::ContractViolation,
            Self::Limit(_) => ErrorCode::ResourceLimit,
            Self::Manifest(_) => ErrorCode::ManifestMalformed,
            Self::UnknownSigner(_) => ErrorCode::UnknownSigner,
            Self::BadSignature(_) => ErrorCode::BadSignature,
            Self::DigestMismatch(_) => ErrorCode::DigestMismatch,
            Self::UnsupportedFormat(_) => ErrorCode::UnsupportedFormat,
            Self::ProtobufDecode(_) => ErrorCode::ProtobufDecode,
            Self::SemanticViolation(_) => ErrorCode::SemanticViolation,
            Self::MissingAsset(_) => ErrorCode::MissingAsset,
            Self::UnsignedDisallowed(_) => ErrorCode::UnsignedDisallowed,
            Self::Io(_) => ErrorCode::Io,
        }
    }
}

pub type Result<T> = std::result::Result<T, BundleError>;
