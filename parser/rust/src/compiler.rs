use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

use crate::ast::{ChildModule, Script};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::validator;

pub struct CompileOutput {
    pub script: Option<Script>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile_source(source: &str) -> CompileOutput {
    let (parsed_root, mut diagnostics) = parse_root_script(source);
    let root_script = match parsed_root {
        Some(script) => script,
        None => {
            diagnostics.sort();
            return CompileOutput {
                script: None,
                diagnostics,
            };
        }
    };

    // Raw-source compilation has no filesystem context, so includes cannot be resolved here.
    for include in &root_script.init.includes {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::EIncludeFileNotFound,
            format!(
                "Included file '{}' could not be read when compiling from raw source",
                include.path
            ),
            Phase::Validation,
            "INIT",
            include.line,
            include.column,
        ));
    }

    diagnostics.extend(validator::validate_requirements(&root_script.init, &[]));
    diagnostics.extend(validator::validate(&root_script));
    diagnostics.sort();

    CompileOutput {
        script: Some(root_script),
        diagnostics,
    }
}

pub fn compile_file(path: &Path) -> Result<CompileOutput, String> {
    let canonical_entry = path
        .canonicalize()
        .map_err(|error| format!("Error resolving '{}': {}", path.display(), error))?;
    let root = canonical_entry
        .parent()
        .ok_or_else(|| format!("Entry script '{}' has no parent directory", path.display()))?;
    let entry_name = canonical_entry.file_name().ok_or_else(|| {
        format!(
            "Entry script '{}' does not have a valid file name",
            path.display()
        )
    })?;

    compile_project(root, Path::new(entry_name))
}

/// Compiles a root script while confining it and all includes to `project_root`.
///
/// `entry_path` and every `@include` must be relative UTF-8 paths without parent
/// traversal. Includes remain relative to the entry script's directory for
/// compatibility with the StoryScript source contract.
pub fn compile_project(project_root: &Path, entry_path: &Path) -> Result<CompileOutput, String> {
    let resolver = FilesystemResolver::new(project_root)?;
    let entry = resolver
        .resolve(Path::new(""), entry_path)
        .map_err(|error| format!("Invalid entry script '{}': {}", entry_path.display(), error))?;
    let source = resolver.read_utf8(&entry).map_err(|error| {
        format!(
            "Error reading entry script '{}': {}",
            entry_path.display(),
            error
        )
    })?;

    let (parsed_root, mut diagnostics) = parse_root_script(&source);
    let mut root_script = match parsed_root {
        Some(script) => script,
        None => {
            diagnostics.sort();
            return Ok(CompileOutput {
                script: None,
                diagnostics,
            });
        }
    };

    let mut modules: Vec<ChildModule> = Vec::new();
    let mut seen_include_paths: HashSet<String> = HashSet::new();
    let mut seen_canonical_paths: HashSet<PathBuf> = HashSet::new();
    seen_include_paths.insert(path_key(&entry.logical));
    seen_canonical_paths.insert(entry.canonical.clone());

    let entry_dir = entry.logical.parent().unwrap_or_else(|| Path::new(""));
    for include in &root_script.init.includes {
        let resolved = match resolver.resolve(entry_dir, Path::new(&include.path)) {
            Ok(resolved) => resolved,
            Err(error) => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EIncludeFileNotFound,
                    format!("Included file '{}' was rejected: {}", include.path, error),
                    Phase::Validation,
                    "INIT",
                    include.line,
                    include.column,
                ));
                continue;
            }
        };

        if !seen_include_paths.insert(path_key(&resolved.logical))
            || !seen_canonical_paths.insert(resolved.canonical.clone())
        {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::EIncludeDuplicatePath,
                format!(
                    "Duplicate normalized or case-folded include path '{}' in manifest",
                    include.path
                ),
                Phase::Validation,
                "INIT",
                include.line,
                include.column,
            ));
            continue;
        }

        let child_source = match resolver.read_utf8(&resolved) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EIncludeFileNotFound,
                    format!(
                        "Included file '{}' could not be read: {}",
                        include.path, error
                    ),
                    Phase::Validation,
                    "INIT",
                    include.line,
                    include.column,
                ));
                continue;
            }
        };

        let (child_module, child_diags) = parse_child_script(&child_source);
        diagnostics.extend(child_diags);
        if let Some(module) = child_module {
            modules.push(module);
        }
    }

    for module in &modules {
        root_script.logic_blocks.extend(module.logic_blocks.clone());
        root_script.scenes.extend(module.scenes.clone());
    }

    diagnostics.extend(validator::validate_requirements(
        &root_script.init,
        &modules,
    ));
    diagnostics.extend(validator::validate(&root_script));
    diagnostics.sort();

    Ok(CompileOutput {
        script: Some(root_script),
        diagnostics,
    })
}

#[derive(Debug)]
struct ResolvedSource {
    logical: PathBuf,
    canonical: PathBuf,
}

struct FilesystemResolver {
    root: PathBuf,
}

impl FilesystemResolver {
    fn new(project_root: &Path) -> Result<Self, String> {
        let root = project_root.canonicalize().map_err(|error| {
            format!(
                "Could not resolve project root '{}': {}",
                project_root.display(),
                error
            )
        })?;
        if !root.is_dir() {
            return Err(format!(
                "Project root '{}' is not a directory",
                project_root.display()
            ));
        }
        Ok(Self { root })
    }

    fn resolve(&self, base: &Path, requested: &Path) -> Result<ResolvedSource, String> {
        if requested.as_os_str().is_empty() {
            return Err("path is empty".to_string());
        }
        if requested.is_absolute() {
            return Err("absolute paths are not allowed".to_string());
        }

        let mut logical = normalize_relative(base)?;
        for component in requested.components() {
            match component {
                Component::Normal(part) => {
                    if part.to_str().is_none() {
                        return Err("path is not valid UTF-8".to_string());
                    }
                    logical.push(part);
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err("parent traversal is not allowed".to_string());
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err("absolute paths are not allowed".to_string());
                }
            }
        }
        if logical.as_os_str().is_empty() {
            return Err("path is empty".to_string());
        }

        let candidate = self.root.join(&logical);
        let canonical = candidate
            .canonicalize()
            .map_err(|error| format!("path could not be resolved: {}", error))?;
        if !canonical.starts_with(&self.root) {
            return Err("resolved path escapes the project root".to_string());
        }
        if !canonical.is_file() {
            return Err("resolved path is not a regular file".to_string());
        }

        Ok(ResolvedSource { logical, canonical })
    }

    fn read_utf8(&self, source: &ResolvedSource) -> Result<String, String> {
        let bytes = fs::read(&source.canonical).map_err(|error| error.to_string())?;
        String::from_utf8(bytes).map_err(|_| "source is not valid UTF-8".to_string())
    }
}

fn normalize_relative(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                if part.to_str().is_none() {
                    return Err("path is not valid UTF-8".to_string());
                }
                normalized.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir => return Err("parent traversal is not allowed".to_string()),
            Component::RootDir | Component::Prefix(_) => {
                return Err("absolute paths are not allowed".to_string());
            }
        }
    }
    Ok(normalized)
}

fn path_key(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
        .nfc()
        .collect::<String>()
        .to_lowercase()
}

fn parse_root_script(source: &str) -> (Option<Script>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut diagnostics = lexer.diagnostics.clone();

    let mut parser = Parser::new(tokens);
    let script = parser.parse();
    diagnostics.extend(parser.diagnostics);

    if script.is_none() && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::ESyntax,
            "Compiler failed to produce a root script",
            Phase::Parse,
            "GLOBAL",
            1,
            1,
        ));
    }

    (script, diagnostics)
}

fn parse_child_script(source: &str) -> (Option<ChildModule>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut diagnostics = lexer.diagnostics.clone();

    let mut parser = Parser::new(tokens);
    let module = parser.parse_child_module();
    diagnostics.extend(parser.diagnostics);

    (module, diagnostics)
}
