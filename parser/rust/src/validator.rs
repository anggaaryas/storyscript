use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::interpolation::scan_placeholders;

/// Performs semantic validation on a parsed StoryScript AST.
/// Returns a list of diagnostics (errors and warnings).
pub fn validate(script: &Script) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let mut declared_vars: HashSet<String> = HashSet::new();
    let mut actor_map: HashMap<String, &ActorDecl> = HashMap::new();
    let mut scene_labels: HashSet<String> = HashSet::new();

    // -----------------------------------------------------------------------
    // INIT validation
    // -----------------------------------------------------------------------

    // Check duplicate global variables
    for var in &script.init.variables {
        if !declared_vars.insert(var.name.clone()) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EGlobalDuplicate,
                format!("Duplicate global variable '${}'", var.name),
                Phase::Validation,
                "INIT",
                var.line,
                var.column,
            ));
        }

        validate_expr_contents(
            &var.value,
            var.line,
            var.column,
            &declared_vars,
            "INIT",
            &mut diags,
        );
    }

    // Check duplicate actors & emotion keys
    for actor in &script.init.actors {
        if actor_map.contains_key(&actor.id) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EActorDuplicate,
                format!("Duplicate actor ID '{}'", actor.id),
                Phase::Validation,
                "INIT",
                actor.line,
                actor.column,
            ));
        } else {
            actor_map.insert(actor.id.clone(), actor);
        }

        let mut emotions: HashSet<String> = HashSet::new();

        validate_interpolated_string(
            &actor.display_name,
            actor.line,
            actor.column,
            &declared_vars,
            "INIT",
            &mut diags,
        );

        for portrait in &actor.portraits {
            if !emotions.insert(portrait.emotion.clone()) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EEmotionDuplicate,
                    format!(
                        "Duplicate emotion key '{}' in actor '{}'",
                        portrait.emotion, actor.id
                    ),
                    Phase::Validation,
                    "INIT",
                    portrait.line,
                    portrait.column,
                ));
            }

            validate_interpolated_string(
                &portrait.path,
                portrait.line,
                portrait.column,
                &declared_vars,
                "INIT",
                &mut diags,
            );
        }
    }

    // Collect scene labels
    for scene in &script.scenes {
        if !scene_labels.insert(scene.label.clone()) {
            diags.push(Diagnostic::new(
                DiagnosticCode::ESceneDuplicate,
                format!("Duplicate scene label '{}'", scene.label),
                Phase::Validation,
                &scene.label,
                scene.line,
                scene.column,
            ));
        }
    }

    // Validate @start target
    if !script.init.start.target.is_empty() && !scene_labels.contains(&script.init.start.target) {
        diags.push(Diagnostic::new(
            DiagnosticCode::EStartTargetMissing,
            format!(
                "@start target '{}' does not match any scene label",
                script.init.start.target
            ),
            Phase::Validation,
            "INIT",
            script.init.start.line,
            script.init.start.column,
        ));
    }

    // -----------------------------------------------------------------------
    // Per-scene validation
    // -----------------------------------------------------------------------

    for scene in &script.scenes {
        // Validate #PREP
        if let Some(prep) = &scene.prep {
            validate_prep_statements(&prep.statements, &declared_vars, &scene.label, &mut diags);
        }

        // Validate #STORY
        validate_story_statements(
            &scene.story.statements,
            &declared_vars,
            &actor_map,
            &scene_labels,
            &scene.label,
            &mut diags,
        );

        // Termination analysis
        if !story_terminates(&scene.story.statements) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EStoryUnterminatedPath,
                "Reachable #STORY path can fall through without @choice, @jump, or @end",
                Phase::Validation,
                &scene.label,
                scene.story.line,
                scene.story.column,
            ));
        }
    }

    diags.sort();
    diags
}

// ---------------------------------------------------------------------------
// #PREP statement validation
// ---------------------------------------------------------------------------

fn validate_prep_statements(
    stmts: &[PrepStatement],
    declared_vars: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            PrepStatement::VarAssign(assign) => {
                if !declared_vars.contains(&assign.name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableUndeclaredWrite,
                        format!("Assignment to undeclared variable '${}'", assign.name),
                        Phase::Validation,
                        scene,
                        assign.line,
                        assign.column,
                    ));
                }
                validate_expr_contents(
                    &assign.value,
                    assign.line,
                    assign.column,
                    declared_vars,
                    scene,
                    diags,
                );
            }
            PrepStatement::IfElse(if_else) => {
                validate_expr_contents(
                    &if_else.condition,
                    if_else.line,
                    if_else.column,
                    declared_vars,
                    scene,
                    diags,
                );
                validate_prep_statements(&if_else.then_branch, declared_vars, scene, diags);
                if let Some(else_branch) = &if_else.else_branch {
                    validate_prep_statements(else_branch, declared_vars, scene, diags);
                }
            }
            PrepStatement::BgDirective { path, line, column } => {
                validate_interpolated_string(path, *line, *column, declared_vars, scene, diags);
            }
            PrepStatement::BgmDirective { value, line, column } => {
                if let BgmValue::Path(path) = value {
                    validate_interpolated_string(path, *line, *column, declared_vars, scene, diags);
                }
            }
            PrepStatement::SfxDirective { path, line, column } => {
                validate_interpolated_string(path, *line, *column, declared_vars, scene, diags);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// #STORY statement validation
// ---------------------------------------------------------------------------

fn validate_story_statements(
    stmts: &[StoryStatement],
    declared_vars: &HashSet<String>,
    actor_map: &HashMap<String, &ActorDecl>,
    scene_labels: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            StoryStatement::Dialogue(dlg) => {
                validate_interpolated_string(
                    &dlg.text,
                    dlg.line,
                    dlg.column,
                    declared_vars,
                    scene,
                    diags,
                );

                // Check actor exists
                match actor_map.get(&dlg.actor_id) {
                    None => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EActorUnknown,
                            format!("Unknown actor ID '{}'", dlg.actor_id),
                            Phase::Validation,
                            scene,
                            dlg.line,
                            dlg.column,
                        ));
                    }
                    Some(actor) => {
                        if let DialogueForm::Portrait { emotion, .. } = &dlg.form {
                            if actor.portraits.is_empty() {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EPortraitModeInvalid,
                                    format!(
                                        "Actor '{}' was declared without portraits; portrait-form dialogue is invalid",
                                        dlg.actor_id
                                    ),
                                    Phase::Validation,
                                    scene,
                                    dlg.line,
                                    dlg.column,
                                ));
                            } else if !actor.portraits.iter().any(|p| p.emotion == *emotion) {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EEmotionUnknown,
                                    format!(
                                        "Unknown emotion '{}' for actor '{}'",
                                        emotion, dlg.actor_id
                                    ),
                                    Phase::Validation,
                                    scene,
                                    dlg.line,
                                    dlg.column,
                                ));
                            }
                        }
                    }
                }
            }
            StoryStatement::Jump { target, line, column } => {
                if !scene_labels.contains(target) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EJumpTargetMissing,
                        format!("@jump target '{}' does not match any scene label", target),
                        Phase::Validation,
                        scene,
                        *line,
                        *column,
                    ));
                }
            }
            StoryStatement::Choice(choice) => {
                let mut all_conditional = true;
                let mut provably_empty = true;

                for opt in &choice.options {
                    if !scene_labels.contains(&opt.target) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EChoiceTargetMissing,
                            format!(
                                "@choice target '{}' does not match any scene label",
                                opt.target
                            ),
                            Phase::Validation,
                            scene,
                            opt.line,
                            opt.column,
                        ));
                    }

                    validate_interpolated_string(
                        &opt.text,
                        opt.line,
                        opt.column,
                        declared_vars,
                        scene,
                        diags,
                    );

                    if opt.condition.is_none() {
                        all_conditional = false;
                        provably_empty = false;
                    } else {
                        validate_expr_contents(
                            opt.condition.as_ref().unwrap(),
                            opt.line,
                            opt.column,
                            declared_vars,
                            scene,
                            diags,
                        );

                        // Check if condition is provably false at compile time
                        if let Some(val) = try_const_eval_bool(opt.condition.as_ref().unwrap()) {
                            if val {
                                provably_empty = false;
                            }
                            // if false, this option is dead — still provably empty unless another option is alive
                        } else {
                            provably_empty = false;
                        }
                    }
                }

                if choice.options.is_empty() || provably_empty {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EChoiceStaticEmpty,
                        "@choice block is provably empty at compile time",
                        Phase::Validation,
                        scene,
                        choice.line,
                        choice.column,
                    ));
                } else if all_conditional {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::WChoicePossiblyEmpty,
                        "@choice block may evaluate to no options at runtime",
                        Phase::Validation,
                        scene,
                        choice.line,
                        choice.column,
                    ));
                }
            }
            StoryStatement::IfElse(if_else) => {
                validate_expr_contents(
                    &if_else.condition,
                    if_else.line,
                    if_else.column,
                    declared_vars,
                    scene,
                    diags,
                );
                validate_story_statements(
                    &if_else.then_branch,
                    declared_vars,
                    actor_map,
                    scene_labels,
                    scene,
                    diags,
                );
                if let Some(else_branch) = &if_else.else_branch {
                    validate_story_statements(
                        else_branch,
                        declared_vars,
                        actor_map,
                        scene_labels,
                        scene,
                        diags,
                    );
                }
            }
            StoryStatement::Narration { text, line, column } => {
                validate_interpolated_string(text, *line, *column, declared_vars, scene, diags);
            }
            StoryStatement::VarOutput { name, line, column } => {
                if !declared_vars.contains(name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableUndeclaredRead,
                        format!("Read of undeclared variable '${}'", name),
                        Phase::Validation,
                        scene,
                        *line,
                        *column,
                    ));
                }
            }
            StoryStatement::End { .. } | StoryStatement::SfxDirective { .. } => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Variable reference validation in expressions
// ---------------------------------------------------------------------------

fn validate_expr_contents(
    expr: &Expr,
    fallback_line: usize,
    fallback_column: usize,
    declared_vars: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::VarRef { name, line, column } => {
            if !declared_vars.contains(name) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableUndeclaredRead,
                    format!("Read of undeclared variable '${}'", name),
                    Phase::Validation,
                    scene,
                    *line,
                    *column,
                ));
            }
        }
        Expr::StringLit(text) => {
            validate_interpolated_string(
                text,
                fallback_line,
                fallback_column,
                declared_vars,
                scene,
                diags,
            );
        }
        Expr::BinOp { left, right, .. } => {
            validate_expr_contents(
                left,
                fallback_line,
                fallback_column,
                declared_vars,
                scene,
                diags,
            );
            validate_expr_contents(
                right,
                fallback_line,
                fallback_column,
                declared_vars,
                scene,
                diags,
            );
        }
        _ => {}
    }
}

fn validate_interpolated_string(
    text: &str,
    line: usize,
    column: usize,
    declared_vars: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match scan_placeholders(text) {
        Ok(placeholders) => {
            for placeholder in placeholders {
                if !declared_vars.contains(&placeholder.name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableUndeclaredRead,
                        format!(
                            "Read of undeclared variable '${}' in interpolation",
                            placeholder.name
                        ),
                        Phase::Validation,
                        scene,
                        line,
                        column.saturating_add(placeholder.offset.saturating_add(1)),
                    ));
                }
            }
        }
        Err(err) => {
            diags.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                format!("Invalid interpolation syntax: {}", err.message),
                Phase::Validation,
                scene,
                line,
                column.saturating_add(err.offset.saturating_add(1)),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// #STORY termination analysis
// ---------------------------------------------------------------------------

/// Returns true if every reachable path through the given statements ends
/// with a terminal directive (@choice, @jump, @end).
fn story_terminates(stmts: &[StoryStatement]) -> bool {
    if stmts.is_empty() {
        return false;
    }

    // Check the last statement
    match stmts.last().unwrap() {
        StoryStatement::Jump { .. }
        | StoryStatement::End { .. }
        | StoryStatement::Choice(_) => true,
        StoryStatement::IfElse(if_else) => {
            let then_terminates = story_terminates(&if_else.then_branch);
            let else_terminates = if_else
                .else_branch
                .as_ref()
                .map(|b| story_terminates(b))
                .unwrap_or(false);
            // Both branches must terminate for the overall path to terminate
            then_terminates && else_terminates
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Constant folding for compile-time analysis
// ---------------------------------------------------------------------------

fn try_const_eval_bool(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::BoolLit(b) => Some(*b),
        Expr::BinOp { left, op, right } => {
            match (try_const_eval_int(left), try_const_eval_int(right)) {
                (Some(l), Some(r)) => match op {
                    BinOperator::EqEq => Some(l == r),
                    BinOperator::NotEq => Some(l != r),
                    BinOperator::Lt => Some(l < r),
                    BinOperator::LtEq => Some(l <= r),
                    BinOperator::Gt => Some(l > r),
                    BinOperator::GtEq => Some(l >= r),
                    _ => None,
                },
                _ => {
                    // Try bool == bool
                    match (try_const_eval_bool(left), try_const_eval_bool(right)) {
                        (Some(l), Some(r)) => match op {
                            BinOperator::EqEq => Some(l == r),
                            BinOperator::NotEq => Some(l != r),
                            _ => None,
                        },
                        _ => None,
                    }
                }
            }
        }
        _ => None,
    }
}

fn try_const_eval_int(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::IntLit(n) => Some(*n),
        Expr::BinOp { left, op, right } => {
            let l = try_const_eval_int(left)?;
            let r = try_const_eval_int(right)?;
            match op {
                BinOperator::Add => Some(l + r),
                BinOperator::Sub => Some(l - r),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_and_validate(src: &str) -> Vec<Diagnostic> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();
        let mut all = lexer.diagnostics;
        all.extend(parser.diagnostics);
        all.extend(validate(&script));
        all
    }

    #[test]
    fn test_valid_minimal_script() {
        let src = r#"
* INIT {
    $x = 10
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    "Hello world."
    @end
}
"#;
        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_duplicate_scene() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    @end
}
* main {
    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::ESceneDuplicate));
    }

    #[test]
    fn test_unterminated_story() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    "Hello"
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::EStoryUnterminatedPath));
    }

    #[test]
    fn test_unknown_actor_in_dialogue() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    BOGUS: "Hi"
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::EActorUnknown));
    }

    #[test]
    fn test_jump_target_missing() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    @jump nonexistent
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::EJumpTargetMissing));
    }
}
