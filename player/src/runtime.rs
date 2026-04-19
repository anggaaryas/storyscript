use std::path::Path;

use storyscript_parser::ast::Script;

use crate::engine::{Engine, StepResult};

pub struct StoryPlayer {
    script_name: String,
    engine: Engine,
    history: Vec<StepResult>,
    current: Option<StepResult>,
}

impl StoryPlayer {
    pub fn new(script_name: impl Into<String>, script: &Script) -> Self {
        let mut player = Self {
            script_name: script_name.into(),
            engine: Engine::new(script),
            history: Vec::new(),
            current: None,
        };
        player.advance();
        player
    }

    pub fn from_source(script_name: impl Into<String>, source: &str) -> Result<Self, String> {
        let script_name = script_name.into();
        let compile = storyscript_parser::compiler::compile_source(source);

        if compile.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                &format!("Compile errors in {}", script_name),
                &compile
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }

        let script = compile.script.ok_or_else(|| {
            format!("Compile failed to produce script for {}", script_name)
        })?;

        Ok(Self::new(script_name, &script))
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let compile = storyscript_parser::compiler::compile_file(path)
            .map_err(|e| format!("Failed to compile {}: {}", display_path(path), e))?;

        if compile.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                &format!("Compile errors in {}", display_path(path)),
                &compile
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }

        let script = compile.script.ok_or_else(|| {
            format!(
                "Compile failed to produce script for {}",
                display_path(path)
            )
        })?;

        Ok(Self::new(display_path(path), &script))
    }

    pub fn script_name(&self) -> &str {
        &self.script_name
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn history(&self) -> &[StepResult] {
        &self.history
    }

    pub fn current(&self) -> Option<&StepResult> {
        self.current.as_ref()
    }

    pub fn advance(&mut self) {
        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        match self.engine.step() {
            Some(result) => self.current = Some(result),
            None => self.current = Some(StepResult::End),
        }
    }

    pub fn select_choice(&mut self, index: usize) -> bool {
        let choices = match self.current.as_ref() {
            Some(StepResult::Choices(choices)) => choices,
            _ => return false,
        };

        if index >= choices.len() {
            return false;
        }

        let choice = choices[index].clone();

        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        self.history
            .push(StepResult::Narration(format!("▸ {}", choice.text)));
        self.engine.select_choice(&choice);

        match self.engine.step() {
            Some(result) => self.current = Some(result),
            None => self.current = Some(StepResult::End),
        }

        true
    }
}

fn format_diagnostics(title: &str, diags: &[String]) -> String {
    if diags.is_empty() {
        return title.to_string();
    }
    let details = diags
        .iter()
        .map(|diag| format!("- {}", diag))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n{}", title, details)
}

fn display_path(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&cwd) {
            return relative.display().to_string();
        }
    }
    path.display().to_string()
}
