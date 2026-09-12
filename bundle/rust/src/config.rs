use std::fs;
use std::path::{Component, Path};

use semver::Version;
use serde::Deserialize;

use crate::{BundleError, COMPILER_VERSION, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub project: Project,
    #[serde(default)]
    pub assets: Assets,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub compiler_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Assets {
    #[serde(default = "default_asset_root")]
    pub root: String,
    #[serde(default)]
    pub literal_policy: LiteralAssetPolicy,
    #[serde(default)]
    pub dynamic_files: Vec<String>,
    #[serde(default)]
    pub dynamic_globs: Vec<String>,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            root: default_asset_root(),
            literal_policy: LiteralAssetPolicy::default(),
            dynamic_files: Vec::new(),
            dynamic_globs: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiteralAssetPolicy {
    #[default]
    Required,
}

impl ProjectConfig {
    pub fn from_path(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        Self::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Self> {
        let config: Self =
            toml::from_str(source).map_err(|error| BundleError::Config(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.project.id.trim().is_empty()
            || self.project.name.trim().is_empty()
            || self.project.version.trim().is_empty()
        {
            return Err(BundleError::Config(
                "project id, name, and version must be non-empty".to_string(),
            ));
        }
        Version::parse(&self.project.version).map_err(|error| {
            BundleError::Config(format!("project version must be SemVer: {error}"))
        })?;
        Version::parse(&self.project.compiler_version).map_err(|error| {
            BundleError::Config(format!("compiler-version must be exact SemVer: {error}"))
        })?;
        if self.project.compiler_version != COMPILER_VERSION {
            return Err(BundleError::CompilerMismatch {
                expected: COMPILER_VERSION.to_string(),
                found: self.project.compiler_version.clone(),
            });
        }

        validate_relative_path("project entry", &self.project.entry)?;
        validate_relative_path("asset root", &self.assets.root)?;
        for path in &self.assets.dynamic_files {
            validate_relative_path("dynamic asset file", path)?;
        }
        for pattern in &self.assets.dynamic_globs {
            validate_relative_path("dynamic asset glob", pattern)?;
        }
        Ok(())
    }
}

fn default_asset_root() -> String {
    "assets".to_string()
}

fn validate_relative_path(label: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(BundleError::Config(format!("{label} must not be empty")));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(BundleError::Config(format!(
            "{label} must be a project-relative path without parent traversal"
        )));
    }
    Ok(())
}
