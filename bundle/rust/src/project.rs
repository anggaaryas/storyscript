use std::path::{Path, PathBuf};

use crate::assets::{Asset, discover};
use crate::config::ProjectConfig;
use crate::contract;
use crate::ir::v1;
use crate::proto::storybundle::v1::CompiledStory;
use crate::{BundleError, Result};

#[derive(Debug)]
pub struct CompiledProject {
    pub root: PathBuf,
    pub config: ProjectConfig,
    pub story: CompiledStory,
    pub assets: Vec<Asset>,
}

pub fn compile(project_root: &Path) -> Result<CompiledProject> {
    let root = project_root.canonicalize().map_err(|error| {
        BundleError::Project(format!(
            "could not resolve project root '{}': {error}",
            project_root.display()
        ))
    })?;
    if !root.is_dir() {
        return Err(BundleError::Project(format!(
            "project root '{}' is not a directory",
            project_root.display()
        )));
    }

    let config = ProjectConfig::from_path(&root.join("StoryScript.toml"))?;
    let compile =
        storyscript_parser::compiler::compile_project(&root, Path::new(&config.project.entry))
            .map_err(BundleError::Project)?;
    if compile
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error())
    {
        return Err(BundleError::Compile(
            compile
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let script = compile
        .script
        .ok_or_else(|| BundleError::Compile("compiler produced no script".to_string()))?;
    let story = v1::convert(&script, &config.project)?;
    contract::validate_story(&story)?;
    let assets = discover(&root, &config.assets, &story)?;

    Ok(CompiledProject {
        root,
        config,
        story,
        assets,
    })
}
