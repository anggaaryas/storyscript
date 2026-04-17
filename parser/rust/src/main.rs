use std::env;
use std::fs;
use std::process;

use storycript_parser::{diagnostic, lexer, parser, validator};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: storycript-parser <file.story> [--json]");
        process::exit(1);
    }

    let file_path = &args[1];
    let json_output = args.iter().any(|a| a == "--json");

    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    // Phase 1: Lexing
    let mut lex = lexer::Lexer::new(&source);
    let tokens = lex.tokenize();
    let mut all_diagnostics = lex.diagnostics.clone();

    // Phase 2: Parsing
    let mut par = parser::Parser::new(tokens);
    let script = match par.parse() {
        Some(s) => s,
        None => {
            all_diagnostics.extend(par.diagnostics.clone());
            print_diagnostics(&all_diagnostics, json_output);
            process::exit(1);
        }
    };
    all_diagnostics.extend(par.diagnostics.clone());

    // Phase 3: Semantic Validation
    let validation_diags = validator::validate(&script);
    all_diagnostics.extend(validation_diags);

    all_diagnostics.sort();

    let has_errors = all_diagnostics.iter().any(|d| d.is_error());

    // Print summary
    let scene_count = script.scenes.len();
    let actor_count = script.init.actors.len();
    let var_count = script.init.variables.len();

    if !json_output {
        println!("=== StoryScript Parser ===");
        println!("File:   {}", file_path);
        println!("Scenes: {}", scene_count);
        println!("Actors: {}", actor_count);
        println!("Vars:   {}", var_count);
        println!("Entry:  {}", script.init.start.target);
        println!();

        // Print scene details
        for scene in &script.scenes {
            let has_prep = scene.prep.is_some();
            let story_stmts = scene.story.statements.len();
            println!(
                "  * {} (prep: {}, story statements: {})",
                scene.label,
                if has_prep { "yes" } else { "no" },
                story_stmts
            );
        }
        println!();
    }

    print_diagnostics(&all_diagnostics, json_output);

    if has_errors {
        if !json_output {
            let error_count = all_diagnostics.iter().filter(|d| d.is_error()).count();
            let warn_count = all_diagnostics.len() - error_count;
            println!(
                "Compilation FAILED: {} error(s), {} warning(s)",
                error_count, warn_count
            );
        }
        process::exit(1);
    } else {
        let warn_count = all_diagnostics.len();
        if !json_output {
            if warn_count > 0 {
                println!("Compilation OK with {} warning(s)", warn_count);
            } else {
                println!("Compilation OK");
            }
        }
    }
}

fn print_diagnostics(diags: &[diagnostic::Diagnostic], json: bool) {
    if diags.is_empty() {
        return;
    }

    if json {
        println!("[");
        for (i, d) in diags.iter().enumerate() {
            if i > 0 {
                println!(",");
            }
            print!("  {}", d.to_json());
        }
        println!("\n]");
    } else {
        for d in diags {
            if d.is_error() {
                println!("ERROR: {}", d);
            } else {
                println!("WARN:  {}", d);
            }
        }
        println!();
    }
}
