use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::interpolation::scan_placeholders;
use rust_decimal::Decimal;

type VarTypes = HashMap<String, VarType>;

/// Performs semantic validation on a parsed StoryScript AST.
/// Returns a list of diagnostics (errors and warnings).
pub fn validate(script: &Script) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let mut declared_vars: VarTypes = HashMap::new();
    let mut actor_map: HashMap<String, &ActorDecl> = HashMap::new();
    let mut scene_labels: HashSet<String> = HashSet::new();

    // -----------------------------------------------------------------------
    // INIT validation
    // -----------------------------------------------------------------------

    // Check duplicate globals and initializer type compatibility
    for var in &script.init.variables {
        match declared_vars.entry(var.name.clone()) {
            Entry::Vacant(slot) => {
                slot.insert(var.var_type);
            }
            Entry::Occupied(_) => {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EGlobalDuplicate,
                    format!("Duplicate global variable '${}'", var.name),
                    Phase::Validation,
                    "INIT",
                    var.line,
                    var.column,
                ));
            }
        }

        if let Some(value_type) = infer_expr_type(
            &var.value,
            var.line,
            var.column,
            Some(var.var_type),
            &declared_vars,
            "INIT",
            &mut diags,
        ) {
            if !is_assignable(var.var_type, value_type) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableTypeMismatch,
                    format!(
                        "Variable '${}' is declared as {}, but initializer has type {}",
                        var.name,
                        type_name(var.var_type),
                        type_name(value_type)
                    ),
                    Phase::Validation,
                    "INIT",
                    var.line,
                    var.column,
                ));
            }
        }
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
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            PrepStatement::VarAssign(assign) => {
                let declared_type = declared_vars.get(&assign.name).copied();
                if declared_type.is_none() {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableUndeclaredWrite,
                        format!("Assignment to undeclared variable '${}'", assign.name),
                        Phase::Validation,
                        scene,
                        assign.line,
                        assign.column,
                    ));
                }

                let value_type = infer_expr_type(
                    &assign.value,
                    assign.line,
                    assign.column,
                    declared_type,
                    declared_vars,
                    scene,
                    diags,
                );

                if let (Some(target_type), Some(rhs_type)) = (declared_type, value_type) {
                    match assign.op {
                        AssignOp::Set => {
                            if !is_assignable(target_type, rhs_type) {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EVariableTypeMismatch,
                                    format!(
                                        "Cannot assign {} to variable '${}' of type {}",
                                        type_name(rhs_type),
                                        assign.name,
                                        type_name(target_type)
                                    ),
                                    Phase::Validation,
                                    scene,
                                    assign.line,
                                    assign.column,
                                ));
                            }
                        }
                        AssignOp::AddEq | AssignOp::SubEq => match target_type {
                            VarType::Integer => {
                                if rhs_type != VarType::Integer {
                                    diags.push(Diagnostic::new(
                                        DiagnosticCode::EVariableTypeMismatch,
                                        format!(
                                            "'{}' with ${} requires integer RHS, found {}",
                                            match assign.op {
                                                AssignOp::AddEq => "+=",
                                                AssignOp::SubEq => "-=",
                                                AssignOp::Set => "=",
                                            },
                                            assign.name,
                                            type_name(rhs_type)
                                        ),
                                        Phase::Validation,
                                        scene,
                                        assign.line,
                                        assign.column,
                                    ));
                                }
                            }
                            VarType::Decimal => {
                                if !is_numeric_type(rhs_type) {
                                    diags.push(Diagnostic::new(
                                            DiagnosticCode::EVariableTypeMismatch,
                                            format!(
                                                "'{}' with ${} requires numeric RHS (integer or decimal), found {}",
                                                match assign.op {
                                                    AssignOp::AddEq => "+=",
                                                    AssignOp::SubEq => "-=",
                                                    AssignOp::Set => "=",
                                                },
                                                assign.name,
                                                type_name(rhs_type)
                                            ),
                                            Phase::Validation,
                                            scene,
                                            assign.line,
                                            assign.column,
                                        ));
                                }
                            }
                            VarType::String | VarType::Boolean => {
                                diags.push(Diagnostic::new(
                                        DiagnosticCode::EVariableCompoundAssignInvalid,
                                        format!(
                                            "'{}' is only valid for integer/decimal variables; '${}' is {}",
                                            match assign.op {
                                                AssignOp::AddEq => "+=",
                                                AssignOp::SubEq => "-=",
                                                AssignOp::Set => "=",
                                            },
                                            assign.name,
                                            type_name(target_type)
                                        ),
                                        Phase::Validation,
                                        scene,
                                        assign.line,
                                        assign.column,
                                    ));
                            }
                        },
                    }
                }
            }
            PrepStatement::IfElse(if_else) => {
                let condition_type = infer_expr_type(
                    &if_else.condition,
                    if_else.line,
                    if_else.column,
                    None,
                    declared_vars,
                    scene,
                    diags,
                );
                if let Some(cond_ty) = condition_type {
                    if cond_ty != VarType::Boolean {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EConditionTypeInvalid,
                            format!("if condition must be boolean, found {}", type_name(cond_ty)),
                            Phase::Validation,
                            scene,
                            if_else.line,
                            if_else.column,
                        ));
                    }
                }

                validate_prep_statements(&if_else.then_branch, declared_vars, scene, diags);
                if let Some(else_branch) = &if_else.else_branch {
                    validate_prep_statements(else_branch, declared_vars, scene, diags);
                }
            }
            PrepStatement::BgDirective { path, line, column } => {
                validate_interpolated_string(path, *line, *column, declared_vars, scene, diags);
            }
            PrepStatement::BgmDirective {
                value,
                line,
                column,
            } => {
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
    declared_vars: &VarTypes,
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
            StoryStatement::Jump {
                target,
                line,
                column,
            } => {
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
                        let cond_expr = opt.condition.as_ref().expect("checked is_some");
                        let cond_ty = infer_expr_type(
                            cond_expr,
                            opt.line,
                            opt.column,
                            None,
                            declared_vars,
                            scene,
                            diags,
                        );
                        if let Some(ty) = cond_ty {
                            if ty != VarType::Boolean {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EConditionTypeInvalid,
                                    format!(
                                        "@choice condition must be boolean, found {}",
                                        type_name(ty)
                                    ),
                                    Phase::Validation,
                                    scene,
                                    opt.line,
                                    opt.column,
                                ));
                            }
                        }

                        // Check if condition is provably false at compile time
                        if let Some(val) = try_const_eval_bool(cond_expr) {
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
                let cond_ty = infer_expr_type(
                    &if_else.condition,
                    if_else.line,
                    if_else.column,
                    None,
                    declared_vars,
                    scene,
                    diags,
                );
                if let Some(ty) = cond_ty {
                    if ty != VarType::Boolean {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EConditionTypeInvalid,
                            format!("if condition must be boolean, found {}", type_name(ty)),
                            Phase::Validation,
                            scene,
                            if_else.line,
                            if_else.column,
                        ));
                    }
                }

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
                if !declared_vars.contains_key(name) {
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
// Expression type inference
// ---------------------------------------------------------------------------

fn infer_expr_type(
    expr: &Expr,
    fallback_line: usize,
    fallback_column: usize,
    assignment_target: Option<VarType>,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<VarType> {
    match expr {
        Expr::IntLit(_) => Some(VarType::Integer),
        Expr::DecimalLit(_) => Some(VarType::Decimal),
        Expr::BoolLit(_) => Some(VarType::Boolean),
        Expr::StringLit(text) => {
            validate_interpolated_string(
                text,
                fallback_line,
                fallback_column,
                declared_vars,
                scene,
                diags,
            );
            Some(VarType::String)
        }
        Expr::VarRef { name, line, column } => match declared_vars.get(name).copied() {
            Some(var_type) => Some(var_type),
            None => {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableUndeclaredRead,
                    format!("Read of undeclared variable '${}'", name),
                    Phase::Validation,
                    scene,
                    *line,
                    *column,
                ));
                None
            }
        },
        Expr::Call {
            name,
            args,
            line,
            column,
        } => infer_call_type(
            name,
            args,
            *line,
            *column,
            assignment_target,
            declared_vars,
            scene,
            diags,
        ),
        Expr::ListLit { .. } => {
            diags.push(Diagnostic::new(
                DiagnosticCode::EExpressionTypeInvalid,
                "List literals are only valid as arguments to pick([ ... ])",
                Phase::Validation,
                scene,
                fallback_line,
                fallback_column,
            ));
            None
        }
        Expr::BinOp { left, op, right } => {
            let left_type = infer_expr_type(
                left,
                fallback_line,
                fallback_column,
                assignment_target,
                declared_vars,
                scene,
                diags,
            );
            let right_type = infer_expr_type(
                right,
                fallback_line,
                fallback_column,
                assignment_target,
                declared_vars,
                scene,
                diags,
            );

            let (left_type, right_type) = match (left_type, right_type) {
                (Some(l), Some(r)) => (l, r),
                _ => return None,
            };

            match op {
                BinOperator::Add | BinOperator::Sub | BinOperator::Mul | BinOperator::Div => {
                    if left_type == VarType::Integer && right_type == VarType::Integer {
                        Some(VarType::Integer)
                    } else if is_numeric_type(left_type) && is_numeric_type(right_type) {
                        Some(VarType::Decimal)
                    } else {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            format!(
                                "Operator '{}' requires numeric operands, found {} and {}",
                                operator_symbol(op),
                                type_name(left_type),
                                type_name(right_type)
                            ),
                            Phase::Validation,
                            scene,
                            fallback_line,
                            fallback_column,
                        ));
                        None
                    }
                }
                BinOperator::Mod => {
                    if left_type == VarType::Integer && right_type == VarType::Integer {
                        Some(VarType::Integer)
                    } else {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            format!(
                                "Operator '%' requires integer operands, found {} and {}",
                                type_name(left_type),
                                type_name(right_type)
                            ),
                            Phase::Validation,
                            scene,
                            fallback_line,
                            fallback_column,
                        ));
                        None
                    }
                }
                BinOperator::EqEq | BinOperator::NotEq => {
                    if left_type == right_type
                        || (is_numeric_type(left_type) && is_numeric_type(right_type))
                    {
                        Some(VarType::Boolean)
                    } else {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            format!(
                                "Operator '{}' cannot compare {} with {}",
                                operator_symbol(op),
                                type_name(left_type),
                                type_name(right_type)
                            ),
                            Phase::Validation,
                            scene,
                            fallback_line,
                            fallback_column,
                        ));
                        None
                    }
                }
                BinOperator::Lt | BinOperator::LtEq | BinOperator::Gt | BinOperator::GtEq => {
                    if is_numeric_type(left_type) && is_numeric_type(right_type) {
                        Some(VarType::Boolean)
                    } else {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            format!(
                                "Operator '{}' requires numeric operands, found {} and {}",
                                operator_symbol(op),
                                type_name(left_type),
                                type_name(right_type)
                            ),
                            Phase::Validation,
                            scene,
                            fallback_line,
                            fallback_column,
                        ));
                        None
                    }
                }
            }
        }
    }
}

fn infer_call_type(
    name: &str,
    args: &[Expr],
    line: usize,
    column: usize,
    assignment_target: Option<VarType>,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<VarType> {
    match name {
        "abs" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EExpressionTypeInvalid,
                    format!("abs() expects exactly 1 argument, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let arg_type = infer_expr_type(
                &args[0],
                line,
                column,
                assignment_target,
                declared_vars,
                scene,
                diags,
            )?;

            if is_numeric_type(arg_type) {
                Some(arg_type)
            } else {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EExpressionTypeInvalid,
                    format!("abs() requires numeric argument, found {}", type_name(arg_type)),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                None
            }
        }
        "rand" => {
            let target = match assignment_target {
                Some(VarType::Integer) => VarType::Integer,
                Some(VarType::Decimal) => VarType::Decimal,
                Some(other) => {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EExpressionTypeInvalid,
                        format!(
                            "rand() requires integer or decimal assignment target, found {}",
                            type_name(other)
                        ),
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    return None;
                }
                None => {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EExpressionTypeInvalid,
                        "rand() requires typed assignment context",
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    return None;
                }
            };

            match args.len() {
                0 => Some(target),
                2 => {
                    let min_ty = infer_expr_type(
                        &args[0],
                        line,
                        column,
                        assignment_target,
                        declared_vars,
                        scene,
                        diags,
                    )?;
                    let max_ty = infer_expr_type(
                        &args[1],
                        line,
                        column,
                        assignment_target,
                        declared_vars,
                        scene,
                        diags,
                    )?;

                    if !is_numeric_type(min_ty) || !is_numeric_type(max_ty) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            format!(
                                "rand(min, max) requires numeric bounds, found {} and {}",
                                type_name(min_ty),
                                type_name(max_ty)
                            ),
                            Phase::Validation,
                            scene,
                            line,
                            column,
                        ));
                        return None;
                    }

                    match target {
                        VarType::Integer => {
                            if min_ty != VarType::Integer || max_ty != VarType::Integer {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EExpressionTypeInvalid,
                                    "Integer rand(min, max) requires integer bounds",
                                    Phase::Validation,
                                    scene,
                                    line,
                                    column,
                                ));
                                return None;
                            }
                        }
                        VarType::Decimal => {
                            // Decimal assignment accepts integer and decimal bounds.
                        }
                        _ => unreachable!(),
                    }

                    if let Some(false) = try_const_check_rand_bounds(&args[0], &args[1], target) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EExpressionTypeInvalid,
                            "rand(min, max) requires min <= max",
                            Phase::Validation,
                            scene,
                            line,
                            column,
                        ));
                        return None;
                    }

                    Some(target)
                }
                _ => {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EExpressionTypeInvalid,
                        format!(
                            "rand() expects 0 or 2 arguments, found {}",
                            args.len()
                        ),
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    None
                }
            }
        }
        "pick" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EExpressionTypeInvalid,
                    format!("pick() expects exactly 1 argument, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let values = match &args[0] {
                Expr::ListLit { items, .. } => items,
                _ => {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EExpressionTypeInvalid,
                        "pick() expects a list literal argument: pick([a, b, ...])",
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    return None;
                }
            };

            if values.is_empty() {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EExpressionTypeInvalid,
                    "pick() requires a non-empty candidate list",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if let Some(target) = assignment_target {
                match target {
                    VarType::Decimal => {
                        for value in values {
                            let ty = infer_expr_type(
                                value,
                                line,
                                column,
                                assignment_target,
                                declared_vars,
                                scene,
                                diags,
                            )?;
                            if !is_numeric_type(ty) {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EExpressionTypeInvalid,
                                    format!(
                                        "pick() for decimal assignment accepts only integer/decimal candidates, found {}",
                                        type_name(ty)
                                    ),
                                    Phase::Validation,
                                    scene,
                                    line,
                                    column,
                                ));
                                return None;
                            }
                        }
                        return Some(VarType::Decimal);
                    }
                    _ => {
                        for value in values {
                            let ty = infer_expr_type(
                                value,
                                line,
                                column,
                                assignment_target,
                                declared_vars,
                                scene,
                                diags,
                            )?;
                            if !is_assignable(target, ty) {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EExpressionTypeInvalid,
                                    format!(
                                        "pick() candidate type {} is incompatible with assignment target {}",
                                        type_name(ty),
                                        type_name(target)
                                    ),
                                    Phase::Validation,
                                    scene,
                                    line,
                                    column,
                                ));
                                return None;
                            }
                        }
                        return Some(target);
                    }
                }
            }

            let first_type = infer_expr_type(
                &values[0],
                line,
                column,
                None,
                declared_vars,
                scene,
                diags,
            )?;
            for value in values.iter().skip(1) {
                let ty = infer_expr_type(
                    value,
                    line,
                    column,
                    None,
                    declared_vars,
                    scene,
                    diags,
                )?;
                if ty != first_type {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EExpressionTypeInvalid,
                        format!(
                            "pick() candidates must share one type outside assignment context, found {} and {}",
                            type_name(first_type),
                            type_name(ty)
                        ),
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    return None;
                }
            }
            Some(first_type)
        }
        _ => {
            diags.push(Diagnostic::new(
                DiagnosticCode::EExpressionTypeInvalid,
                format!("Unknown function '{}'", name),
                Phase::Validation,
                scene,
                line,
                column,
            ));
            None
        }
    }
}

fn operator_symbol(op: &BinOperator) -> &'static str {
    match op {
        BinOperator::Add => "+",
        BinOperator::Sub => "-",
        BinOperator::Mul => "*",
        BinOperator::Div => "/",
        BinOperator::Mod => "%",
        BinOperator::EqEq => "==",
        BinOperator::NotEq => "!=",
        BinOperator::Lt => "<",
        BinOperator::LtEq => "<=",
        BinOperator::Gt => ">",
        BinOperator::GtEq => ">=",
    }
}

fn is_numeric_type(var_type: VarType) -> bool {
    matches!(var_type, VarType::Integer | VarType::Decimal)
}

fn is_assignable(target: VarType, source: VarType) -> bool {
    target == source || (target == VarType::Decimal && source == VarType::Integer)
}

fn type_name(var_type: VarType) -> &'static str {
    match var_type {
        VarType::Integer => "integer",
        VarType::String => "string",
        VarType::Boolean => "boolean",
        VarType::Decimal => "decimal",
    }
}

fn validate_interpolated_string(
    text: &str,
    line: usize,
    column: usize,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match scan_placeholders(text) {
        Ok(placeholders) => {
            for placeholder in placeholders {
                if !declared_vars.contains_key(&placeholder.name) {
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
        StoryStatement::Jump { .. } | StoryStatement::End { .. } | StoryStatement::Choice(_) => {
            true
        }
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
                    match (try_const_eval_decimal(left), try_const_eval_decimal(right)) {
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
                BinOperator::Mul => Some(l * r),
                BinOperator::Div => {
                    if r == 0 {
                        None
                    } else {
                        Some(l / r)
                    }
                }
                BinOperator::Mod => {
                    if r == 0 {
                        None
                    } else {
                        Some(l % r)
                    }
                }
                _ => None,
            }
        }
        Expr::Call { name, args, .. } if name == "abs" && args.len() == 1 => {
            let value = try_const_eval_int(&args[0])?;
            value.checked_abs()
        }
        _ => None,
    }
}

fn try_const_eval_decimal(expr: &Expr) -> Option<Decimal> {
    match expr {
        Expr::DecimalLit(n) => Some(*n),
        Expr::IntLit(n) => Some(Decimal::from(*n)),
        Expr::BinOp { left, op, right } => {
            let l = try_const_eval_decimal(left)?;
            let r = try_const_eval_decimal(right)?;
            match op {
                BinOperator::Add => Some(l + r),
                BinOperator::Sub => Some(l - r),
                BinOperator::Mul => Some(l * r),
                BinOperator::Div => {
                    if r == Decimal::ZERO {
                        None
                    } else {
                        Some(l / r)
                    }
                }
                _ => None,
            }
        }
        Expr::Call { name, args, .. } if name == "abs" && args.len() == 1 => {
            let value = try_const_eval_decimal(&args[0])?;
            Some(value.abs())
        }
        _ => None,
    }
}

fn try_const_check_rand_bounds(min_expr: &Expr, max_expr: &Expr, target: VarType) -> Option<bool> {
    match target {
        VarType::Integer => {
            let min = try_const_eval_int(min_expr)?;
            let max = try_const_eval_int(max_expr)?;
            Some(min <= max)
        }
        VarType::Decimal => {
            let min = try_const_eval_decimal(min_expr)?;
            let max = try_const_eval_decimal(max_expr)?;
            Some(min <= max)
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
    $x as integer = 10
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == DiagnosticCode::ESceneDuplicate)
        );
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == DiagnosticCode::EStoryUnterminatedPath)
        );
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == DiagnosticCode::EActorUnknown)
        );
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == DiagnosticCode::EJumpTargetMissing)
        );
    }

    #[test]
    fn test_variable_type_mismatch() {
        let src = r#"
* INIT {
    $score as integer = 10
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $score = "oops"

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(
            diags
                .iter()
                .any(|d| d.code == DiagnosticCode::EVariableTypeMismatch)
        );
    }

    #[test]
    fn test_arithmetic_and_functions_valid() {
        let src = r#"
* INIT {
    $i as integer = 10
    $d as decimal = 1.5
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $i = ($i * 3) / 2 % 5
    $i = abs($i - 10)
    $d = rand(1, 2.5)
    $d = pick([1, 1.5, 2])

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_rand_requires_assignment_context() {
        let src = r#"
* INIT {
    $x as integer = 1
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    if (rand() > 0) {
        $x = 2
    }

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EExpressionTypeInvalid
                && d.message.contains("requires typed assignment context")
        }));
    }

    #[test]
    fn test_pick_empty_list_rejected() {
        let src = r#"
* INIT {
    $x as integer = 1
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $x = pick([])

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EExpressionTypeInvalid
                && d.message.contains("non-empty candidate list")
        }));
    }

    #[test]
    fn test_modulo_decimal_rejected() {
        let src = r#"
* INIT {
    $d as decimal = 5.5
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $d = $d % 2

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EExpressionTypeInvalid && d.message.contains("Operator '%'")
        }));
    }
}
