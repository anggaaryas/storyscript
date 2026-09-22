use clap::{Parser, Subcommand};
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::pkcs8::DecodePublicKey;
use std::path::PathBuf;

use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{self, VerificationPolicy};
use storyscript_bundle::trust::TrustStore;
use storyscript_bundle::{BundleError, Result, exporter, init, schema};

#[derive(Debug, Parser)]
#[command(name = "storyscript-bundle", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a minimal StoryScript project in a new directory.
    Init {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Validate the checked-in Protobuf descriptor fingerprint.
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    /// Compile and export a deterministic signed .storybundle.
    Export {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        signing_key: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Display bounded archive-envelope metadata without trusting or decoding the model.
    Inspect {
        bundle: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Strictly verify and decode a bundle using a trusted Ed25519 public key.
    Verify {
        bundle: PathBuf,
        #[arg(long)]
        public_key: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SchemaCommand {
    Check,
}

pub fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Init {
            path,
            id,
            name,
            json,
        } => {
            init::create(&path, &id, &name)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "path": path,
                    })
                );
            } else {
                println!("Initialized {}", path.display());
            }
            Ok(())
        }
        Command::Schema {
            command: SchemaCommand::Check,
        } => {
            let fingerprint = schema::check()?;
            println!("StoryBundle v1 schema OK: {fingerprint}");
            Ok(())
        }
        Command::Export {
            project,
            output,
            signing_key,
            json,
        } => {
            exporter::export_with_pkcs8_file(&project, &output, &signing_key)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "output": output,
                    })
                );
            } else {
                println!("Exported {}", output.display());
            }
            Ok(())
        }
        Command::Inspect { bundle, json } => {
            let bytes = std::fs::read(&bundle)?;
            let inspection = loader::inspect(&bytes, ResourceLimits::HARD)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "trust": "untrusted",
                        "archive_size": inspection.archive_size,
                        "entry_paths": inspection.entry_paths,
                        "format_version": inspection.format_version,
                        "project_id": inspection.project_id,
                        "signer_key_id": inspection.signer_key_id,
                    })
                );
            } else {
                println!("UNTRUSTED StoryBundle envelope");
                println!("Archive bytes: {}", inspection.archive_size);
                println!("Entries: {}", inspection.entry_paths.len());
                for path in inspection.entry_paths {
                    println!("  {path}");
                }
            }
            Ok(())
        }
        Command::Verify {
            bundle,
            public_key,
            json,
        } => {
            let bytes = std::fs::read(&bundle)?;
            let pem = std::fs::read_to_string(public_key)?;
            let key = VerifyingKey::from_public_key_pem(&pem)
                .map_err(|error| BundleError::SigningKey(error.to_string()))?;
            let mut trust = TrustStore::new();
            let key_id = trust.insert_key(key);
            let loaded = loader::load(
                &bytes,
                &trust,
                VerificationPolicy::Strict,
                ResourceLimits::HARD,
            )?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "verified",
                        "project_id": loaded.manifest().project.id,
                        "signer_key_id": key_id,
                        "asset_count": loaded.assets().len(),
                    })
                );
            } else {
                println!("Verified project: {}", loaded.manifest().project.id);
                println!("Signer: {key_id}");
                println!("Assets: {}", loaded.assets().len());
            }
            Ok(())
        }
    }
}
