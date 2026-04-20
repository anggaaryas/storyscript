use std::collections::HashSet;
use std::fs;
use std::path::Path;

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
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Error reading '{}': {}", path.display(), e))?;

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

    let root_dir = path.parent().unwrap_or_else(|| Path::new("."));
    for include in &root_script.init.includes {
        if !seen_include_paths.insert(include.path.clone()) {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::EIncludeDuplicatePath,
                format!("Duplicate include path '{}' in manifest", include.path),
                Phase::Validation,
                "INIT",
                include.line,
                include.column,
            ));
            continue;
        }

        let child_path = root_dir.join(&include.path);
        let child_source = match fs::read_to_string(&child_path) {
            Ok(source) => source,
            Err(_) => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EIncludeFileNotFound,
                    format!("Included file '{}' could not be read", include.path),
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

    diagnostics.extend(validator::validate_requirements(&root_script.init, &modules));
    diagnostics.extend(validator::validate(&root_script));
    diagnostics.sort();

    Ok(CompileOutput {
        script: Some(root_script),
        diagnostics,
    })
}

fn parse_root_script(source: &str) -> (Option<Script>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut diagnostics = lexer.diagnostics.clone();

    let mut parser = Parser::new(tokens);
    let script = parser.parse();
    diagnostics.extend(parser.diagnostics);

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
