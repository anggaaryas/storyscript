use std::path::PathBuf;
use std::process;

use clap::Parser;
use storyscript_parser::{compiler, diagnostic};

#[derive(Debug, Parser)]
#[command(
    name = "storyscript-parser",
    version,
    about = "Compile and validate a StoryScript file"
)]
struct Cli {
    /// StoryScript file to compile and validate.
    file: PathBuf,

    /// Emit diagnostics as JSON.
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let compile = match compiler::compile_file(&cli.file) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let mut all_diagnostics = compile.diagnostics;
    let script = match compile.script {
        Some(s) => s,
        None => {
            print_diagnostics(&all_diagnostics, cli.json);
            process::exit(1);
        }
    };

    all_diagnostics.sort();

    let has_errors = all_diagnostics.iter().any(|d| d.is_error());

    // Print summary
    let scene_count = script.scenes.len();
    let actor_count = script.init.actors.len();
    let var_count = script.init.variables.len();

    if !cli.json {
        println!("=== StoryScript Parser ===");
        println!("File:   {}", cli.file.display());
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

    print_diagnostics(&all_diagnostics, cli.json);

    if has_errors {
        if !cli.json {
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
        if !cli.json {
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
