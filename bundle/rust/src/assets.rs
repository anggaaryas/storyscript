use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};

use globset::{Glob, GlobSetBuilder};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

use crate::config::Assets;
use crate::proto::storybundle::v1 as pb;
use crate::{BundleError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub logical_path: String,
    pub bytes: Vec<u8>,
}

pub fn discover(root: &Path, config: &Assets, story: &pb::CompiledStory) -> Result<Vec<Asset>> {
    let asset_root = resolve_asset_root(root, &config.root)?;
    let references = collect_references(story);
    let candidates = configured_candidates(&asset_root, config)?;
    let mut selected = BTreeMap::<String, Asset>::new();
    let mut case_paths = HashMap::<String, String>::new();

    for reference in references {
        if let Some(path) = literal_path(reference)? {
            insert_asset(&asset_root, &path, &mut selected, &mut case_paths)?;
            continue;
        }

        if candidates.is_empty() {
            return Err(BundleError::Asset(
                "dynamic asset template requires dynamic-files or dynamic-globs".to_string(),
            ));
        }
        let matcher = template_matcher(reference)?;
        let matching = candidates
            .keys()
            .filter(|path| matcher.is_match(path))
            .cloned()
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(BundleError::Asset(format!(
                "dynamic asset template '{}' has no compatible configured match",
                display_template(reference)
            )));
        }
        for path in matching {
            insert_asset(&asset_root, &path, &mut selected, &mut case_paths)?;
        }
    }

    Ok(selected.into_values().collect())
}

pub fn validate_closure<'a>(
    story: &pb::CompiledStory,
    asset_paths: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let paths = asset_paths.into_iter().collect::<Vec<_>>();
    for reference in collect_references(story) {
        if let Some(path) = literal_path(reference)? {
            let normalized = normalize_logical(&path)?;
            if !paths.contains(&normalized.as_str()) {
                return Err(BundleError::MissingAsset(normalized));
            }
        } else {
            let matcher = template_matcher(reference)?;
            if !paths.iter().any(|path| matcher.is_match(path)) {
                return Err(BundleError::MissingAsset(display_template(reference)));
            }
        }
    }
    Ok(())
}

fn resolve_asset_root(project_root: &Path, configured: &str) -> Result<PathBuf> {
    let project_root = project_root
        .canonicalize()
        .map_err(|error| BundleError::Asset(format!("project root cannot be resolved: {error}")))?;
    let logical = normalize_logical(configured)?;
    let path = project_root.join(&logical);
    let canonical = path.canonicalize().map_err(|error| {
        BundleError::Asset(format!(
            "asset root '{}' cannot be resolved: {error}",
            configured
        ))
    })?;
    if !canonical.starts_with(&project_root) || !canonical.is_dir() {
        return Err(BundleError::Asset(
            "asset root must be a directory inside the project".to_string(),
        ));
    }
    Ok(canonical)
}

fn configured_candidates(asset_root: &Path, config: &Assets) -> Result<BTreeMap<String, PathBuf>> {
    let mut candidates = BTreeMap::new();
    let mut case_paths = HashMap::new();
    for configured in &config.dynamic_files {
        let logical = normalize_logical(configured)?;
        let canonical = resolve_file(asset_root, &logical)?;
        insert_candidate(logical, canonical, &mut candidates, &mut case_paths)?;
    }

    for pattern in &config.dynamic_globs {
        let normalized_pattern = normalize_pattern(pattern)?;
        let mut builder = GlobSetBuilder::new();
        builder.add(
            Glob::new(&normalized_pattern).map_err(|error| {
                BundleError::Asset(format!("invalid glob '{pattern}': {error}"))
            })?,
        );
        let matcher = builder
            .build()
            .map_err(|error| BundleError::Asset(format!("invalid glob '{pattern}': {error}")))?;
        let mut matched = false;
        for entry in WalkDir::new(asset_root).follow_links(false) {
            let entry = entry.map_err(|error| BundleError::Asset(error.to_string()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry.path().strip_prefix(asset_root).map_err(|error| {
                BundleError::Asset(format!("asset path could not be relativized: {error}"))
            })?;
            let logical = path_to_logical(relative)?;
            if matcher.is_match(&logical) {
                matched = true;
                let canonical = resolve_file(asset_root, &logical)?;
                insert_candidate(logical, canonical, &mut candidates, &mut case_paths)?;
            }
        }
        if !matched {
            return Err(BundleError::Asset(format!(
                "dynamic asset glob '{pattern}' matched no files"
            )));
        }
    }
    Ok(candidates)
}

fn insert_candidate(
    logical: String,
    canonical: PathBuf,
    candidates: &mut BTreeMap<String, PathBuf>,
    case_paths: &mut HashMap<String, String>,
) -> Result<()> {
    let folded = logical.to_lowercase();
    if let Some(existing) = case_paths.insert(folded, logical.clone())
        && existing != logical
    {
        return Err(BundleError::Asset(format!(
            "case-folded asset collision between '{existing}' and '{logical}'"
        )));
    }
    if let Some(existing) = candidates.insert(logical.clone(), canonical.clone())
        && existing != canonical
    {
        return Err(BundleError::Asset(format!(
            "duplicate logical asset path '{logical}'"
        )));
    }
    Ok(())
}

fn insert_asset(
    asset_root: &Path,
    logical: &str,
    selected: &mut BTreeMap<String, Asset>,
    case_paths: &mut HashMap<String, String>,
) -> Result<()> {
    let logical = normalize_logical(logical)?;
    let folded = logical.to_lowercase();
    if let Some(existing) = case_paths.insert(folded, logical.clone())
        && existing != logical
    {
        return Err(BundleError::Asset(format!(
            "case-folded asset collision between '{existing}' and '{logical}'"
        )));
    }
    if selected.contains_key(&logical) {
        return Ok(());
    }
    let canonical = resolve_file(asset_root, &logical)?;
    let bytes = fs::read(canonical)?;
    selected.insert(
        logical.clone(),
        Asset {
            logical_path: logical,
            bytes,
        },
    );
    Ok(())
}

fn resolve_file(asset_root: &Path, logical: &str) -> Result<PathBuf> {
    let canonical = asset_root.join(logical).canonicalize().map_err(|error| {
        BundleError::Asset(format!("asset '{logical}' cannot be resolved: {error}"))
    })?;
    if !canonical.starts_with(asset_root) {
        return Err(BundleError::Asset(format!(
            "asset '{logical}' escapes the asset root"
        )));
    }
    if !canonical.is_file() {
        return Err(BundleError::Asset(format!(
            "asset '{logical}' is not a regular file"
        )));
    }
    Ok(canonical)
}

fn normalize_logical(value: &str) -> Result<String> {
    if value.is_empty() || value.contains('\\') {
        return Err(BundleError::Asset(
            "asset paths must be non-empty and use POSIX separators".to_string(),
        ));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(BundleError::Asset(
            "absolute asset path is forbidden".to_string(),
        ));
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| BundleError::Asset("asset path is not UTF-8".to_string()))?;
                parts.push(part.nfc().collect::<String>());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(BundleError::Asset(
                    "asset path traversal is forbidden".to_string(),
                ));
            }
        }
    }
    if parts.is_empty() {
        return Err(BundleError::Asset("asset path is empty".to_string()));
    }
    let normalized = parts.join("/");
    let lowercase = normalized.to_lowercase();
    if lowercase.ends_with(".storyscript") || lowercase.ends_with(".storyscript.map") {
        return Err(BundleError::Asset(
            "raw StoryScript source and source maps are forbidden assets".to_string(),
        ));
    }
    Ok(normalized)
}

fn normalize_pattern(value: &str) -> Result<String> {
    if value.contains('\\') || value.starts_with('/') {
        return Err(BundleError::Asset(
            "asset globs must be relative and use POSIX separators".to_string(),
        ));
    }
    if value
        .split('/')
        .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(BundleError::Asset(
            "asset glob contains an invalid path component".to_string(),
        ));
    }
    Ok(value.nfc().collect())
}

fn path_to_logical(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .ok_or_else(|| BundleError::Asset("asset path is not UTF-8".to_string()))?;
    normalize_logical(&value.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn literal_path(value: &pb::InterpolatedString) -> Result<Option<String>> {
    let mut output = String::new();
    for segment in &value.segments {
        match segment.value.as_ref() {
            Some(pb::string_segment::Value::Literal(value)) => output.push_str(value),
            Some(pb::string_segment::Value::Variable(_)) => return Ok(None),
            None => {
                return Err(BundleError::Contract(
                    "asset template segment is missing a value".to_string(),
                ));
            }
        }
    }
    Ok(Some(output))
}

fn template_matcher(value: &pb::InterpolatedString) -> Result<globset::GlobMatcher> {
    let mut pattern = String::new();
    for segment in &value.segments {
        match segment.value.as_ref() {
            Some(pb::string_segment::Value::Literal(value)) => {
                pattern.push_str(&globset::escape(value));
            }
            Some(pb::string_segment::Value::Variable(_)) => pattern.push('*'),
            None => {
                return Err(BundleError::Contract(
                    "asset template segment is missing a value".to_string(),
                ));
            }
        }
    }
    Glob::new(&pattern)
        .map(|glob| glob.compile_matcher())
        .map_err(|error| BundleError::Asset(format!("invalid asset template: {error}")))
}

fn display_template(value: &pb::InterpolatedString) -> String {
    value
        .segments
        .iter()
        .map(|segment| match segment.value.as_ref() {
            Some(pb::string_segment::Value::Literal(value)) => value.clone(),
            Some(pb::string_segment::Value::Variable(value)) => format!("${{{value}}}"),
            None => "<?>".to_string(),
        })
        .collect()
}

fn collect_references(story: &pb::CompiledStory) -> Vec<&pb::InterpolatedString> {
    let mut references = Vec::new();
    if let Some(initialization) = &story.initialization {
        for actor in &initialization.actors {
            for portrait in &actor.portraits {
                if let Some(path) = &portrait.asset_path {
                    references.push(path);
                }
            }
        }
    }
    for logic in &story.logic_blocks {
        collect_prep(&logic.body, &mut references);
    }
    for scene in &story.scenes {
        if let Some(prep) = &scene.prep {
            collect_prep(&prep.statements, &mut references);
        }
        if let Some(story) = &scene.story {
            collect_story(&story.statements, &mut references);
        }
    }
    references
}

fn collect_prep<'a>(
    statements: &'a [pb::PrepStatement],
    output: &mut Vec<&'a pb::InterpolatedString>,
) {
    use pb::prep_statement::Value;
    for statement in statements {
        match statement.value.as_ref() {
            Some(Value::Background(value)) => output.extend(value.asset_path.as_ref()),
            Some(Value::Bgm(value)) => {
                if let Some(pb::bgm_directive::Value::AssetPath(path)) = &value.value {
                    output.push(path);
                }
            }
            Some(Value::Sfx(value)) => output.extend(value.asset_path.as_ref()),
            Some(Value::IfElse(value)) => {
                if let Some(branch) = &value.then_branch {
                    collect_prep(&branch.statements, output);
                }
                if let Some(branch) = &value.else_branch {
                    collect_prep(&branch.statements, output);
                }
            }
            Some(Value::ForSnapshot(value)) => collect_prep(&value.body, output),
            Some(Value::Repeat(value)) => collect_prep(&value.body, output),
            _ => {}
        }
    }
}

fn collect_story<'a>(
    statements: &'a [pb::StoryStatement],
    output: &mut Vec<&'a pb::InterpolatedString>,
) {
    use pb::story_statement::Value;
    for statement in statements {
        match statement.value.as_ref() {
            Some(Value::Sfx(value)) => output.extend(value.asset_path.as_ref()),
            Some(Value::IfElse(value)) => {
                if let Some(branch) = &value.then_branch {
                    collect_story(&branch.statements, output);
                }
                if let Some(branch) = &value.else_branch {
                    collect_story(&branch.statements, output);
                }
            }
            Some(Value::ForSnapshot(value)) => collect_story(&value.body, output),
            Some(Value::Repeat(value)) => collect_story(&value.body, output),
            _ => {}
        }
    }
}
