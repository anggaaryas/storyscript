use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::ProjectConfig;
use crate::{BundleError, COMPILER_VERSION, Result};

const STARTER_STORY: &str = r#"* INIT {
    @start main
}

* main {
    #STORY
    "Welcome to StoryScript."
    @end
}
"#;

#[derive(Serialize)]
struct StarterConfig<'a> {
    project: StarterProject<'a>,
    assets: StarterAssets,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StarterProject<'a> {
    id: &'a str,
    name: &'a str,
    version: &'static str,
    entry: &'static str,
    compiler_version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StarterAssets {
    root: &'static str,
    literal_policy: &'static str,
}

/// Creates a minimal StoryScript project at a path that does not yet exist.
pub fn create(target: &Path, project_id: &str, project_name: &str) -> Result<()> {
    validate_metadata(project_id, project_name)?;
    reject_existing_target(target)?;

    let manifest = render_manifest(project_id, project_name)?;
    // Keep the generated descriptor and the parser's actual config contract in lockstep.
    ProjectConfig::parse(&manifest)?;

    if let Some(parent) = target.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    fs::create_dir(target).map_err(|error| map_target_creation_error(target, error))?;
    let mut partial = PartialProject::new(target);

    partial.create_directory(target.join("story"))?;
    partial.create_directory(target.join("assets"))?;
    partial.write_file(target.join("StoryScript.toml"), manifest.as_bytes())?;
    partial.write_file(
        target.join("story/main.StoryScript"),
        STARTER_STORY.as_bytes(),
    )?;
    partial.write_file(target.join("assets/.gitkeep"), b"")?;
    partial.commit();

    Ok(())
}

fn validate_metadata(project_id: &str, project_name: &str) -> Result<()> {
    if project_id.trim().is_empty() {
        return Err(BundleError::Config(
            "project id must be non-empty".to_string(),
        ));
    }
    if project_name.trim().is_empty() {
        return Err(BundleError::Config(
            "project name must be non-empty".to_string(),
        ));
    }
    Ok(())
}

fn reject_existing_target(target: &Path) -> Result<()> {
    match fs::symlink_metadata(target) {
        Ok(_) => Err(BundleError::Output(format!(
            "initialization target '{}' already exists",
            target.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(BundleError::Io(error)),
    }
}

fn render_manifest<'a>(project_id: &'a str, project_name: &'a str) -> Result<String> {
    toml::to_string_pretty(&StarterConfig {
        project: StarterProject {
            id: project_id,
            name: project_name,
            version: "0.1.0",
            entry: "story/main.StoryScript",
            compiler_version: COMPILER_VERSION,
        },
        assets: StarterAssets {
            root: "assets",
            literal_policy: "required",
        },
    })
    .map_err(|error| BundleError::Config(format!("could not render starter project: {error}")))
}

fn map_target_creation_error(target: &Path, error: io::Error) -> BundleError {
    if error.kind() == io::ErrorKind::AlreadyExists {
        BundleError::Output(format!(
            "initialization target '{}' already exists",
            target.display()
        ))
    } else {
        BundleError::Io(error)
    }
}

struct PartialProject {
    root: PathBuf,
    created: Vec<CreatedEntry>,
    committed: bool,
}

enum CreatedEntry {
    File(PathBuf),
    Directory(PathBuf),
}

impl PartialProject {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            created: Vec::new(),
            committed: false,
        }
    }

    fn create_directory(&mut self, path: PathBuf) -> Result<()> {
        fs::create_dir(&path)?;
        self.created.push(CreatedEntry::Directory(path));
        Ok(())
    }

    fn write_file(&mut self, path: PathBuf, bytes: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        self.created.push(CreatedEntry::File(path));
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for PartialProject {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for entry in self.created.iter().rev() {
            match entry {
                CreatedEntry::File(path) => {
                    let _ = fs::remove_file(path);
                }
                CreatedEntry::Directory(path) => {
                    let _ = fs::remove_dir(path);
                }
            }
        }
        let _ = fs::remove_dir(&self.root);
    }
}
