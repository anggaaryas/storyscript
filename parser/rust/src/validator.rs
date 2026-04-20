use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::interpolation::scan_placeholders;
use rust_decimal::Decimal;

type VarTypes = HashMap<String, VarType>;
type LogicSignatures = HashMap<String, LogicSignature>;

#[derive(Debug, Clone)]
struct LogicSignature {
    params: Vec<LogicParam>,
    return_type: Option<VarType>,
}

#[derive(Debug, Clone)]
struct LogicCallEdge {
    target: String,
    line: usize,
    column: usize,
}

/// Performs semantic validation on a parsed StoryScript AST.
/// Returns a list of diagnostics (errors and warnings).
pub fn validate(script: &Script) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let mut declared_vars: VarTypes = HashMap::new();
    let mut actor_map: HashMap<String, &ActorDecl> = HashMap::new();
    let mut scene_labels: HashSet<String> = HashSet::new();
    let logic_signatures = collect_logic_signatures(script, &mut diags);

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
            true,
            false,
            &logic_signatures,
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
    // Logic block validation
    // -----------------------------------------------------------------------

    for logic in &script.logic_blocks {
        let Some(signature) = logic_signatures.get(&logic.name) else {
            continue;
        };

        let mut scoped_vars = declared_vars.clone();
        let mut local_vars: HashSet<String> = HashSet::new();
        let mut readonly_vars: HashSet<String> = HashSet::new();

        for param in &signature.params {
            if declared_vars.contains_key(&param.name) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableScopeConflict,
                    format!(
                        "Logic parameter '${}' conflicts with global variable of the same name",
                        param.name
                    ),
                    Phase::Validation,
                    &logic.name,
                    param.line,
                    param.column,
                ));
                continue;
            }

            local_vars.insert(param.name.clone());
            scoped_vars.insert(param.name.clone(), param.var_type);
        }

        validate_prep_statements(
            &logic.body,
            &declared_vars,
            &logic_signatures,
            &mut scoped_vars,
            &mut local_vars,
            &logic.name,
            0,
            &mut readonly_vars,
            &mut diags,
            true,
            signature.return_type,
        );

        if signature.return_type.is_some() && !logic_returns(&logic.body) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionReturnMissing,
                format!(
                    "Logic function '{}' declares a return type and must return on all reachable paths",
                    logic.name
                ),
                Phase::Validation,
                &logic.name,
                logic.line,
                logic.column,
            ));
        }
    }

    validate_logic_recursion(script, &logic_signatures, &mut diags);

    // -----------------------------------------------------------------------
    // Per-scene validation
    // -----------------------------------------------------------------------

    for scene in &script.scenes {
        let mut scoped_vars = declared_vars.clone();
        let mut local_vars: HashSet<String> = HashSet::new();

        // Validate #PREP
        if let Some(prep) = &scene.prep {
            let mut readonly_vars: HashSet<String> = HashSet::new();
            validate_prep_statements(
                &prep.statements,
                &declared_vars,
                &logic_signatures,
                &mut scoped_vars,
                &mut local_vars,
                &scene.label,
                0,
                &mut readonly_vars,
                &mut diags,
                false,
                None,
            );
        }

        // Validate #STORY
        validate_story_statements(
            &scene.story.statements,
            &logic_signatures,
            &scoped_vars,
            &actor_map,
            &scene_labels,
            &scene.label,
            0,
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

fn collect_logic_signatures(script: &Script, diags: &mut Vec<Diagnostic>) -> LogicSignatures {
    let mut signatures: LogicSignatures = HashMap::new();

    for logic in &script.logic_blocks {
        if is_builtin_function_name(&logic.name) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionDuplicate,
                format!(
                    "Logic function '{}' conflicts with reserved built-in function name",
                    logic.name
                ),
                Phase::Validation,
                &logic.name,
                logic.line,
                logic.column,
            ));
            continue;
        }

        if signatures.contains_key(&logic.name) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionDuplicate,
                format!("Duplicate logic function '{}'", logic.name),
                Phase::Validation,
                &logic.name,
                logic.line,
                logic.column,
            ));
            continue;
        }

        let mut seen_params: HashSet<&str> = HashSet::new();
        for param in &logic.params {
            if !seen_params.insert(param.name.as_str()) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionParamDuplicate,
                    format!(
                        "Duplicate logic parameter '${}' in function '{}'",
                        param.name, logic.name
                    ),
                    Phase::Validation,
                    &logic.name,
                    param.line,
                    param.column,
                ));
            }
        }

        signatures.insert(
            logic.name.clone(),
            LogicSignature {
                params: logic.params.clone(),
                return_type: logic.return_type,
            },
        );
    }

    signatures
}

fn validate_logic_recursion(
    script: &Script,
    signatures: &LogicSignatures,
    diags: &mut Vec<Diagnostic>,
) {
    let mut graph: HashMap<String, Vec<LogicCallEdge>> = HashMap::new();

    for logic in &script.logic_blocks {
        if !signatures.contains_key(&logic.name) {
            continue;
        }
        let mut edges = Vec::new();
        collect_logic_calls_from_stmts(&logic.body, signatures, &mut edges);
        graph.insert(logic.name.clone(), edges);
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Done,
    }

    fn dfs(
        node: &str,
        graph: &HashMap<String, Vec<LogicCallEdge>>,
        states: &mut HashMap<String, VisitState>,
        stack: &mut Vec<String>,
        flagged_edges: &mut HashSet<(String, usize, usize)>,
        diags: &mut Vec<Diagnostic>,
    ) {
        states.insert(node.to_string(), VisitState::Visiting);
        stack.push(node.to_string());

        if let Some(edges) = graph.get(node) {
            for edge in edges {
                let target_state = states.get(&edge.target).copied();
                if target_state == Some(VisitState::Done) {
                    continue;
                }

                if target_state == Some(VisitState::Visiting)
                    || stack.iter().any(|entry| entry == &edge.target)
                {
                    let edge_key = (edge.target.clone(), edge.line, edge.column);
                    if flagged_edges.insert(edge_key) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EFunctionRecursionForbidden,
                            format!(
                                "Recursive logic call detected from '{}' to '{}'",
                                node, edge.target
                            ),
                            Phase::Validation,
                            node,
                            edge.line,
                            edge.column,
                        ));
                    }
                    continue;
                }

                dfs(
                    &edge.target,
                    graph,
                    states,
                    stack,
                    flagged_edges,
                    diags,
                );
            }
        }

        stack.pop();
        states.insert(node.to_string(), VisitState::Done);
    }

    let mut states: HashMap<String, VisitState> = HashMap::new();
    let mut flagged_edges: HashSet<(String, usize, usize)> = HashSet::new();
    for node in graph.keys() {
        if states.get(node).copied() == Some(VisitState::Done) {
            continue;
        }
        let mut stack = Vec::new();
        dfs(
            node,
            &graph,
            &mut states,
            &mut stack,
            &mut flagged_edges,
            diags,
        );
    }
}

fn collect_logic_calls_from_stmts(
    stmts: &[PrepStatement],
    signatures: &LogicSignatures,
    out: &mut Vec<LogicCallEdge>,
) {
    for stmt in stmts {
        match stmt {
            PrepStatement::VarDecl(decl) => {
                collect_logic_calls_from_expr(&decl.value, signatures, out);
            }
            PrepStatement::VarAssign(assign) => {
                collect_logic_calls_from_expr(&assign.value, signatures, out);
            }
            PrepStatement::Call {
                name,
                args,
                line,
                column,
            } => {
                if signatures.contains_key(name) {
                    out.push(LogicCallEdge {
                        target: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                for arg in args {
                    collect_logic_calls_from_expr(arg, signatures, out);
                }
            }
            PrepStatement::IfElse(if_else) => {
                collect_logic_calls_from_expr(&if_else.condition, signatures, out);
                collect_logic_calls_from_stmts(&if_else.then_branch, signatures, out);
                if let Some(else_branch) = &if_else.else_branch {
                    collect_logic_calls_from_stmts(else_branch, signatures, out);
                }
            }
            PrepStatement::ForSnapshot(loop_stmt) => {
                collect_logic_calls_from_stmts(&loop_stmt.body, signatures, out);
            }
            PrepStatement::Repeat(repeat_stmt) => {
                collect_logic_calls_from_stmts(&repeat_stmt.body, signatures, out);
            }
            PrepStatement::Return { value, .. } => {
                if let Some(expr) = value {
                    collect_logic_calls_from_expr(expr, signatures, out);
                }
            }
            PrepStatement::BgDirective { .. }
            | PrepStatement::BgmDirective { .. }
            | PrepStatement::SfxDirective { .. }
            | PrepStatement::Break { .. }
            | PrepStatement::Continue { .. } => {}
        }
    }
}

fn collect_logic_calls_from_expr(
    expr: &Expr,
    signatures: &LogicSignatures,
    out: &mut Vec<LogicCallEdge>,
) {
    match expr {
        Expr::Call {
            name,
            args,
            line,
            column,
        } => {
            if signatures.contains_key(name) {
                out.push(LogicCallEdge {
                    target: name.clone(),
                    line: *line,
                    column: *column,
                });
            }

            for arg in args {
                collect_logic_calls_from_expr(arg, signatures, out);
            }
        }
        Expr::BinOp { left, right, .. } => {
            collect_logic_calls_from_expr(left, signatures, out);
            collect_logic_calls_from_expr(right, signatures, out);
        }
        Expr::ListLit { items, .. } => {
            for item in items {
                collect_logic_calls_from_expr(item, signatures, out);
            }
        }
        Expr::IntLit(_)
        | Expr::DecimalLit(_)
        | Expr::BoolLit(_)
        | Expr::StringLit(_)
        | Expr::VarRef { .. } => {}
    }
}

fn logic_returns(stmts: &[PrepStatement]) -> bool {
    if stmts.is_empty() {
        return false;
    }

    match stmts.last().unwrap() {
        PrepStatement::Return { .. } => true,
        PrepStatement::IfElse(if_else) => {
            let then_returns = logic_returns(&if_else.then_branch);
            let else_returns = if_else
                .else_branch
                .as_ref()
                .map(|branch| logic_returns(branch))
                .unwrap_or(false);
            then_returns && else_returns
        }
        PrepStatement::BgDirective { .. }
        | PrepStatement::BgmDirective { .. }
        | PrepStatement::SfxDirective { .. }
        | PrepStatement::VarDecl(_)
        | PrepStatement::VarAssign(_)
        | PrepStatement::Call { .. }
        | PrepStatement::ForSnapshot(_)
        | PrepStatement::Repeat(_)
        | PrepStatement::Break { .. }
        | PrepStatement::Continue { .. } => false,
    }
}

fn is_builtin_function_name(name: &str) -> bool {
    matches!(
        name,
        "abs"
            | "rand"
            | "pick"
            | "array_push"
            | "array_pop"
            | "array_strip"
            | "array_clear"
            | "array_contains"
            | "array_size"
            | "array_join"
            | "array_get"
            | "array_insert"
            | "array_remove"
    )
}

pub fn validate_requirements(init: &InitBlock, modules: &[ChildModule]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let mut global_types: HashMap<&str, VarType> = HashMap::new();
    for var in &init.variables {
        global_types.insert(var.name.as_str(), var.var_type);
    }

    let mut actor_emotions: HashMap<&str, HashSet<&str>> = HashMap::new();
    for actor in &init.actors {
        let mut emotions = HashSet::new();
        for portrait in &actor.portraits {
            emotions.insert(portrait.emotion.as_str());
        }
        actor_emotions.insert(actor.id.as_str(), emotions);
    }

    for module in modules {
        let req = &module.require;

        let mut seen_vars: HashSet<&str> = HashSet::new();
        for var in &req.variables {
            if !seen_vars.insert(var.name.as_str()) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EGlobalDuplicate,
                    format!("Duplicate REQUIRE variable '${}'", var.name),
                    Phase::Validation,
                    "REQUIRE",
                    var.line,
                    var.column,
                ));
                continue;
            }

            match global_types.get(var.name.as_str()).copied() {
                None => diags.push(Diagnostic::new(
                    DiagnosticCode::ERequireVariableMissing,
                    format!(
                        "REQUIRE variable '${}' is not declared in root INIT",
                        var.name
                    ),
                    Phase::Validation,
                    "REQUIRE",
                    var.line,
                    var.column,
                )),
                Some(root_type) if root_type != var.var_type => diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableTypeMismatch,
                    format!(
                        "REQUIRE variable '${}' expects {}, but root INIT declares {}",
                        var.name,
                        type_name(var.var_type),
                        type_name(root_type)
                    ),
                    Phase::Validation,
                    "REQUIRE",
                    var.line,
                    var.column,
                )),
                Some(_) => {}
            }
        }

        let mut seen_actors: HashSet<&str> = HashSet::new();
        for actor in &req.actors {
            if !seen_actors.insert(actor.id.as_str()) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EActorDuplicate,
                    format!("Duplicate REQUIRE actor '{}'", actor.id),
                    Phase::Validation,
                    "REQUIRE",
                    actor.line,
                    actor.column,
                ));
                continue;
            }

            let Some(available_emotions) = actor_emotions.get(actor.id.as_str()) else {
                diags.push(Diagnostic::new(
                    DiagnosticCode::ERequireActorMissing,
                    format!("REQUIRE actor '{}' is not declared in root INIT", actor.id),
                    Phase::Validation,
                    "REQUIRE",
                    actor.line,
                    actor.column,
                ));
                continue;
            };

            let mut seen_emotions: HashSet<&str> = HashSet::new();
            for emotion in &actor.emotions {
                if !seen_emotions.insert(emotion.name.as_str()) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EEmotionDuplicate,
                        format!(
                            "Duplicate REQUIRE emotion '{}' for actor '{}'",
                            emotion.name, actor.id
                        ),
                        Phase::Validation,
                        "REQUIRE",
                        emotion.line,
                        emotion.column,
                    ));
                    continue;
                }

                if !available_emotions.contains(emotion.name.as_str()) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ERequireEmotionMissing,
                        format!(
                            "REQUIRE emotion '{}' for actor '{}' is not declared in root INIT",
                            emotion.name, actor.id
                        ),
                        Phase::Validation,
                        "REQUIRE",
                        emotion.line,
                        emotion.column,
                    ));
                }
            }
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
    global_vars: &VarTypes,
    logic_signatures: &LogicSignatures,
    scoped_vars: &mut VarTypes,
    local_vars: &mut HashSet<String>,
    scene: &str,
    loop_depth: usize,
    readonly_vars: &mut HashSet<String>,
    diags: &mut Vec<Diagnostic>,
    in_logic_body: bool,
    logic_return_type: Option<VarType>,
) {
    for stmt in stmts {
        match stmt {
            PrepStatement::VarDecl(decl) => {
                if global_vars.contains_key(&decl.name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableScopeConflict,
                        format!(
                            "Local variable '${}' conflicts with global variable of the same name",
                            decl.name
                        ),
                        Phase::Validation,
                        scene,
                        decl.line,
                        decl.column,
                    ));
                    continue;
                }

                if local_vars.contains(&decl.name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ELocalDuplicate,
                        format!(
                            "Duplicate local variable '${}' in scene '{}'",
                            decl.name, scene
                        ),
                        Phase::Validation,
                        scene,
                        decl.line,
                        decl.column,
                    ));
                    continue;
                }

                if let Some(value_type) = infer_expr_type(
                    &decl.value,
                    decl.line,
                    decl.column,
                    Some(decl.var_type),
                    true,
                    true,
                    logic_signatures,
                    scoped_vars,
                    scene,
                    diags,
                ) {
                    if !is_assignable(decl.var_type, value_type) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EVariableTypeMismatch,
                            format!(
                                "Variable '${}' is declared as {}, but initializer has type {}",
                                decl.name,
                                type_name(decl.var_type),
                                type_name(value_type)
                            ),
                            Phase::Validation,
                            scene,
                            decl.line,
                            decl.column,
                        ));
                    }
                }

                local_vars.insert(decl.name.clone());
                scoped_vars.insert(decl.name.clone(), decl.var_type);
            }
            PrepStatement::VarAssign(assign) => {
                if readonly_vars.contains(&assign.name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ELoopIteratorReadOnly,
                        format!(
                            "Loop iterator '${}' is read-only and cannot be assigned",
                            assign.name
                        ),
                        Phase::Validation,
                        scene,
                        assign.line,
                        assign.column,
                    ));
                    continue;
                }

                let declared_type = scoped_vars.get(&assign.name).copied();
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
                    true,
                    true,
                    logic_signatures,
                    scoped_vars,
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
                            VarType::String
                            | VarType::Boolean
                            | VarType::ArrayInteger
                            | VarType::ArrayString
                            | VarType::ArrayBoolean
                            | VarType::ArrayDecimal => {
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
                    true,
                    true,
                    logic_signatures,
                    scoped_vars,
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

                validate_prep_statements(
                    &if_else.then_branch,
                    global_vars,
                    logic_signatures,
                    scoped_vars,
                    local_vars,
                    scene,
                    loop_depth,
                    readonly_vars,
                    diags,
                    in_logic_body,
                    logic_return_type,
                );
                if let Some(else_branch) = &if_else.else_branch {
                    validate_prep_statements(
                        else_branch,
                        global_vars,
                        logic_signatures,
                        scoped_vars,
                        local_vars,
                        scene,
                        loop_depth,
                        readonly_vars,
                        diags,
                        in_logic_body,
                        logic_return_type,
                    );
                }
            }
            PrepStatement::ForSnapshot(loop_stmt) => {
                let mut can_enter_body = true;

                if global_vars.contains_key(&loop_stmt.item_name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableScopeConflict,
                        format!(
                            "Loop iterator '${}' conflicts with global variable of the same name",
                            loop_stmt.item_name
                        ),
                        Phase::Validation,
                        scene,
                        loop_stmt.line,
                        loop_stmt.column,
                    ));
                    can_enter_body = false;
                } else if local_vars.contains(&loop_stmt.item_name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ELocalDuplicate,
                        format!(
                            "Loop iterator '${}' conflicts with existing local variable in scene '{}'",
                            loop_stmt.item_name, scene
                        ),
                        Phase::Validation,
                        scene,
                        loop_stmt.line,
                        loop_stmt.column,
                    ));
                    can_enter_body = false;
                }

                let iterator_type = match scoped_vars.get(&loop_stmt.array_name).copied() {
                    Some(array_type) => match array_element_type(array_type) {
                        Some(element_type) => Some(element_type),
                        None => {
                            diags.push(Diagnostic::new(
                                DiagnosticCode::EFunctionArgumentInvalid,
                                format!(
                                    "for (...) snapshot source '${}' must be an array variable",
                                    loop_stmt.array_name
                                ),
                                Phase::Validation,
                                scene,
                                loop_stmt.line,
                                loop_stmt.column,
                            ));
                            None
                        }
                    },
                    None => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EVariableUndeclaredRead,
                            format!(
                                "Read of undeclared variable '${}' in for snapshot source",
                                loop_stmt.array_name
                            ),
                            Phase::Validation,
                            scene,
                            loop_stmt.line,
                            loop_stmt.column,
                        ));
                        None
                    }
                };

                let mut inserted_iterator = false;
                if can_enter_body {
                    if let Some(element_type) = iterator_type {
                        scoped_vars.insert(loop_stmt.item_name.clone(), element_type);
                        local_vars.insert(loop_stmt.item_name.clone());
                        readonly_vars.insert(loop_stmt.item_name.clone());
                        inserted_iterator = true;
                    }
                }

                validate_prep_statements(
                    &loop_stmt.body,
                    global_vars,
                    logic_signatures,
                    scoped_vars,
                    local_vars,
                    scene,
                    loop_depth + 1,
                    readonly_vars,
                    diags,
                    in_logic_body,
                    logic_return_type,
                );

                if inserted_iterator {
                    scoped_vars.remove(&loop_stmt.item_name);
                    local_vars.remove(&loop_stmt.item_name);
                    readonly_vars.remove(&loop_stmt.item_name);
                }
            }
            PrepStatement::Repeat(repeat_stmt) => {
                validate_repeat_count(&repeat_stmt.count, scoped_vars, scene, diags);
                validate_prep_statements(
                    &repeat_stmt.body,
                    global_vars,
                    logic_signatures,
                    scoped_vars,
                    local_vars,
                    scene,
                    loop_depth + 1,
                    readonly_vars,
                    diags,
                    in_logic_body,
                    logic_return_type,
                );
            }
            PrepStatement::Break { line, column } | PrepStatement::Continue { line, column } => {
                if loop_depth == 0 {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ELoopControlOutsideLoop,
                        "break/continue is only valid inside loop bodies",
                        Phase::Validation,
                        scene,
                        *line,
                        *column,
                    ));
                }
            }
            PrepStatement::Call {
                name,
                args,
                line,
                column,
            } => {
                let _ = infer_call_type(
                    name,
                    args,
                    *line,
                    *column,
                    None,
                    true,
                    true,
                    logic_signatures,
                    true,
                    scoped_vars,
                    scene,
                    diags,
                );
            }
            PrepStatement::BgDirective { path, line, column } => {
                validate_interpolated_string(path, *line, *column, scoped_vars, scene, diags);
            }
            PrepStatement::BgmDirective {
                value,
                line,
                column,
            } => {
                if let BgmValue::Path(path) = value {
                    validate_interpolated_string(path, *line, *column, scoped_vars, scene, diags);
                }
            }
            PrepStatement::SfxDirective { path, line, column } => {
                validate_interpolated_string(path, *line, *column, scoped_vars, scene, diags);
            }
            PrepStatement::Return { value, line, column } => {
                if !in_logic_body {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EReturnContextInvalid,
                        "return is only valid inside logic blocks",
                        Phase::Validation,
                        scene,
                        *line,
                        *column,
                    ));
                    continue;
                }

                match (logic_return_type, value) {
                    (None, None) => {}
                    (None, Some(_)) => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EReturnTypeMismatch,
                            "Void logic function cannot return a value",
                            Phase::Validation,
                            scene,
                            *line,
                            *column,
                        ));
                    }
                    (Some(_), None) => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EReturnTypeMismatch,
                            "Typed logic function must return a value",
                            Phase::Validation,
                            scene,
                            *line,
                            *column,
                        ));
                    }
                    (Some(expected), Some(expr)) => {
                        if let Some(actual) = infer_expr_type(
                            expr,
                            *line,
                            *column,
                            Some(expected),
                            true,
                            true,
                            logic_signatures,
                            scoped_vars,
                            scene,
                            diags,
                        ) {
                            if !is_assignable(expected, actual) {
                                diags.push(Diagnostic::new(
                                    DiagnosticCode::EReturnTypeMismatch,
                                    format!(
                                        "return expression type {} is incompatible with declared return type {}",
                                        type_name(actual),
                                        type_name(expected)
                                    ),
                                    Phase::Validation,
                                    scene,
                                    *line,
                                    *column,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// #STORY statement validation
// ---------------------------------------------------------------------------

fn validate_story_statements(
    stmts: &[StoryStatement],
    logic_signatures: &LogicSignatures,
    declared_vars: &VarTypes,
    actor_map: &HashMap<String, &ActorDecl>,
    scene_labels: &HashSet<String>,
    scene: &str,
    loop_depth: usize,
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
                let availability = validate_choice_entries(
                    &choice.entries,
                    logic_signatures,
                    declared_vars,
                    scene_labels,
                    scene,
                    diags,
                );

                if !availability.can_produce {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EChoiceStaticEmpty,
                        "@choice block is provably empty at compile time",
                        Phase::Validation,
                        scene,
                        choice.line,
                        choice.column,
                    ));
                } else if !availability.guaranteed_non_empty {
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
                    false,
                    false,
                    logic_signatures,
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
                    logic_signatures,
                    declared_vars,
                    actor_map,
                    scene_labels,
                    scene,
                    loop_depth,
                    diags,
                );
                if let Some(else_branch) = &if_else.else_branch {
                    validate_story_statements(
                        else_branch,
                        logic_signatures,
                        declared_vars,
                        actor_map,
                        scene_labels,
                        scene,
                        loop_depth,
                        diags,
                    );
                }
            }
            StoryStatement::ForSnapshot(loop_stmt) => {
                if declared_vars.contains_key(&loop_stmt.item_name) {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableScopeConflict,
                        format!(
                            "Loop iterator '${}' conflicts with an existing variable in scene scope",
                            loop_stmt.item_name
                        ),
                        Phase::Validation,
                        scene,
                        loop_stmt.line,
                        loop_stmt.column,
                    ));
                }

                let iterator_type = match declared_vars.get(&loop_stmt.array_name).copied() {
                    Some(array_type) => match array_element_type(array_type) {
                        Some(element_type) => Some(element_type),
                        None => {
                            diags.push(Diagnostic::new(
                                DiagnosticCode::EFunctionArgumentInvalid,
                                format!(
                                    "for (...) snapshot source '${}' must be an array variable",
                                    loop_stmt.array_name
                                ),
                                Phase::Validation,
                                scene,
                                loop_stmt.line,
                                loop_stmt.column,
                            ));
                            None
                        }
                    },
                    None => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EVariableUndeclaredRead,
                            format!(
                                "Read of undeclared variable '${}' in for snapshot source",
                                loop_stmt.array_name
                            ),
                            Phase::Validation,
                            scene,
                            loop_stmt.line,
                            loop_stmt.column,
                        ));
                        None
                    }
                };

                let mut loop_scope = declared_vars.clone();
                if let Some(element_type) = iterator_type {
                    loop_scope.insert(loop_stmt.item_name.clone(), element_type);
                }

                validate_story_statements(
                    &loop_stmt.body,
                    logic_signatures,
                    &loop_scope,
                    actor_map,
                    scene_labels,
                    scene,
                    loop_depth + 1,
                    diags,
                );
            }
            StoryStatement::Repeat(repeat_stmt) => {
                validate_repeat_count(&repeat_stmt.count, declared_vars, scene, diags);
                validate_story_statements(
                    &repeat_stmt.body,
                    logic_signatures,
                    declared_vars,
                    actor_map,
                    scene_labels,
                    scene,
                    loop_depth + 1,
                    diags,
                );
            }
            StoryStatement::Break { line, column }
            | StoryStatement::Continue { line, column } => {
                if loop_depth == 0 {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::ELoopControlOutsideLoop,
                        "break/continue is only valid inside loop bodies",
                        Phase::Validation,
                        scene,
                        *line,
                        *column,
                    ));
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

#[derive(Debug, Clone, Copy, Default)]
struct ChoiceAvailability {
    can_produce: bool,
    guaranteed_non_empty: bool,
}

impl ChoiceAvailability {
    fn empty() -> Self {
        Self {
            can_produce: false,
            guaranteed_non_empty: false,
        }
    }
}

fn validate_choice_entries(
    entries: &[ChoiceEntry],
    logic_signatures: &LogicSignatures,
    declared_vars: &VarTypes,
    scene_labels: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> ChoiceAvailability {
    let mut aggregate = ChoiceAvailability::empty();

    for entry in entries {
        let availability = validate_choice_entry(
            entry,
            logic_signatures,
            declared_vars,
            scene_labels,
            scene,
            diags,
        );
        aggregate.can_produce |= availability.can_produce;
        aggregate.guaranteed_non_empty |= availability.guaranteed_non_empty;
    }

    aggregate
}

fn validate_choice_entry(
    entry: &ChoiceEntry,
    logic_signatures: &LogicSignatures,
    declared_vars: &VarTypes,
    scene_labels: &HashSet<String>,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> ChoiceAvailability {
    match entry {
        ChoiceEntry::Option(opt) => {
            if !scene_labels.contains(&opt.target) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EChoiceTargetMissing,
                    format!("@choice target '{}' does not match any scene label", opt.target),
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

            ChoiceAvailability {
                can_produce: true,
                guaranteed_non_empty: true,
            }
        }
        ChoiceEntry::If(if_entry) => {
            let cond_ty = infer_expr_type(
                &if_entry.condition,
                if_entry.line,
                if_entry.column,
                None,
                false,
                false,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            );
            if let Some(ty) = cond_ty {
                if ty != VarType::Boolean {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EConditionTypeInvalid,
                        format!("@choice condition must be boolean, found {}", type_name(ty)),
                        Phase::Validation,
                        scene,
                        if_entry.line,
                        if_entry.column,
                    ));
                }
            }

            let body = validate_choice_entries(
                &if_entry.body,
                logic_signatures,
                declared_vars,
                scene_labels,
                scene,
                diags,
            );
            match try_const_eval_bool(&if_entry.condition) {
                Some(false) => ChoiceAvailability::empty(),
                Some(true) => body,
                None => ChoiceAvailability {
                    can_produce: body.can_produce,
                    guaranteed_non_empty: false,
                },
            }
        }
        ChoiceEntry::Repeat(repeat_entry) => {
            validate_repeat_count(&repeat_entry.count, declared_vars, scene, diags);
            let body = validate_choice_entries(
                &repeat_entry.body,
                logic_signatures,
                declared_vars,
                scene_labels,
                scene,
                diags,
            );

            match &repeat_entry.count {
                RepeatCount::IntLiteral { value, .. } if *value <= 0 => ChoiceAvailability::empty(),
                RepeatCount::IntLiteral { .. } => body,
                RepeatCount::Variable { .. } => ChoiceAvailability {
                    can_produce: body.can_produce,
                    guaranteed_non_empty: false,
                },
            }
        }
        ChoiceEntry::ForSnapshot(loop_entry) => {
            let mut iterator_valid = true;
            if declared_vars.contains_key(&loop_entry.item_name) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EVariableScopeConflict,
                    format!(
                        "Loop iterator '${}' conflicts with an existing variable in scene scope",
                        loop_entry.item_name
                    ),
                    Phase::Validation,
                    scene,
                    loop_entry.line,
                    loop_entry.column,
                ));
                iterator_valid = false;
            }

            let iterator_type = match declared_vars.get(&loop_entry.array_name).copied() {
                Some(array_type) => match array_element_type(array_type) {
                    Some(element_type) => Some(element_type),
                    None => {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EFunctionArgumentInvalid,
                            format!(
                                "for (...) snapshot source '${}' must be an array variable",
                                loop_entry.array_name
                            ),
                            Phase::Validation,
                            scene,
                            loop_entry.line,
                            loop_entry.column,
                        ));
                        iterator_valid = false;
                        None
                    }
                },
                None => {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EVariableUndeclaredRead,
                        format!(
                            "Read of undeclared variable '${}' in for snapshot source",
                            loop_entry.array_name
                        ),
                        Phase::Validation,
                        scene,
                        loop_entry.line,
                        loop_entry.column,
                    ));
                    iterator_valid = false;
                    None
                }
            };

            let mut loop_scope = declared_vars.clone();
            if iterator_valid {
                if let Some(element_type) = iterator_type {
                    loop_scope.insert(loop_entry.item_name.clone(), element_type);
                }
            }

            let body = validate_choice_entries(
                &loop_entry.body,
                logic_signatures,
                &loop_scope,
                scene_labels,
                scene,
                diags,
            );

            if iterator_valid {
                ChoiceAvailability {
                    can_produce: body.can_produce,
                    guaranteed_non_empty: false,
                }
            } else {
                ChoiceAvailability::empty()
            }
        }
    }
}

fn validate_repeat_count(
    count: &RepeatCount,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match count {
        RepeatCount::IntLiteral {
            value,
            line,
            column,
        } => {
            if *value <= 0 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::ERangeInvalid,
                    "repeat(count) requires count > 0",
                    Phase::Validation,
                    scene,
                    *line,
                    *column,
                ));
            }
        }
        RepeatCount::Variable { name, line, column } => match declared_vars.get(name).copied() {
            None => diags.push(Diagnostic::new(
                DiagnosticCode::EVariableUndeclaredRead,
                format!("Read of undeclared variable '${}' in repeat count", name),
                Phase::Validation,
                scene,
                *line,
                *column,
            )),
            Some(VarType::Integer) => {}
            Some(other) => diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionArgumentInvalid,
                format!(
                    "repeat(count) requires integer count variable, found {}",
                    type_name(other)
                ),
                Phase::Validation,
                scene,
                *line,
                *column,
            )),
        },
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
    allow_mutating_calls: bool,
    allow_user_logic_calls: bool,
    logic_signatures: &LogicSignatures,
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
            allow_mutating_calls,
            allow_user_logic_calls,
            logic_signatures,
            false,
            declared_vars,
            scene,
            diags,
        ),
        Expr::ListLit { items, .. } => infer_list_literal_type(
            items,
            fallback_line,
            fallback_column,
            assignment_target,
            allow_mutating_calls,
            allow_user_logic_calls,
            logic_signatures,
            declared_vars,
            scene,
            diags,
        ),
        Expr::BinOp { left, op, right } => {
            let left_type = infer_expr_type(
                left,
                fallback_line,
                fallback_column,
                assignment_target,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            );
            let right_type = infer_expr_type(
                right,
                fallback_line,
                fallback_column,
                assignment_target,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
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
    allow_mutating_calls: bool,
    allow_user_logic_calls: bool,
    logic_signatures: &LogicSignatures,
    allow_void_return: bool,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<VarType> {
    if let Some(signature) = logic_signatures.get(name) {
        if !allow_user_logic_calls {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionContextInvalid,
                format!("Logic call '{}' is not allowed in this phase", name),
                Phase::Validation,
                scene,
                line,
                column,
            ));
            return None;
        }

        if args.len() != signature.params.len() {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionArityInvalid,
                format!(
                    "{}() expects exactly {} arguments, found {}",
                    name,
                    signature.params.len(),
                    args.len()
                ),
                Phase::Validation,
                scene,
                line,
                column,
            ));
            return None;
        }

        for (arg, param) in args.iter().zip(signature.params.iter()) {
            let Some(arg_type) = infer_expr_type(
                arg,
                line,
                column,
                Some(param.var_type),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            ) else {
                continue;
            };

            if !is_assignable(param.var_type, arg_type) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "{}() argument '${}' expects {}, found {}",
                        name,
                        param.name,
                        type_name(param.var_type),
                        type_name(arg_type)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
            }
        }

        return match signature.return_type {
            Some(return_type) => Some(return_type),
            None => {
                if allow_void_return {
                    None
                } else {
                    diags.push(Diagnostic::new(
                        DiagnosticCode::EFunctionContextInvalid,
                        format!(
                            "{}() returns void and cannot be used as an expression",
                            name
                        ),
                        Phase::Validation,
                        scene,
                        line,
                        column,
                    ));
                    None
                }
            }
        };
    }

    match name {
        "abs" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
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
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if is_numeric_type(arg_type) {
                Some(arg_type)
            } else {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
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
                        DiagnosticCode::EFunctionContextInvalid,
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
                        DiagnosticCode::EFunctionContextInvalid,
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
                        allow_mutating_calls,
                        allow_user_logic_calls,
                        logic_signatures,
                        declared_vars,
                        scene,
                        diags,
                    )?;
                    let max_ty = infer_expr_type(
                        &args[1],
                        line,
                        column,
                        assignment_target,
                        allow_mutating_calls,
                        allow_user_logic_calls,
                        logic_signatures,
                        declared_vars,
                        scene,
                        diags,
                    )?;

                    if !is_numeric_type(min_ty) || !is_numeric_type(max_ty) {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EFunctionArgumentInvalid,
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
                                    DiagnosticCode::EFunctionArgumentInvalid,
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
                            DiagnosticCode::ERangeInvalid,
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
                        DiagnosticCode::EFunctionArityInvalid,
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
        "array_push" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_push() expects exactly 2 arguments, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_push() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_push",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            let element_ty = array_element_type(array_ty)?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_push() value argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let value_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(element_ty),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if !is_assignable(element_ty, value_ty) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_push() value type {} is incompatible with {}",
                        type_name(value_ty),
                        type_name(element_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if allow_void_return {
                return None;
            }

            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionContextInvalid,
                "array_push() returns void and cannot be used as an expression",
                Phase::Validation,
                scene,
                line,
                column,
            ));
            None
        }
        "array_pop" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_pop() expects exactly 1 argument, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_pop() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_pop",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            array_element_type(array_ty)
        }
        "array_strip" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_strip() expects exactly 2 arguments, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_strip() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_strip",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            let element_ty = array_element_type(array_ty)?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_strip() value argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let value_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(element_ty),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            let valid = if element_ty == VarType::Decimal {
                is_numeric_type(value_ty)
            } else {
                is_assignable(element_ty, value_ty)
            };
            if !valid {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_strip() value type {} is incompatible with {}",
                        type_name(value_ty),
                        type_name(element_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if allow_void_return {
                return None;
            }

            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionContextInvalid,
                "array_strip() returns void and cannot be used as an expression",
                Phase::Validation,
                scene,
                line,
                column,
            ));
            None
        }
        "array_clear" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_clear() expects exactly 1 argument, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_clear() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let _ = infer_array_argument_type(
                &args[0],
                "array_clear",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if allow_void_return {
                return None;
            }

            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionContextInvalid,
                "array_clear() returns void and cannot be used as an expression",
                Phase::Validation,
                scene,
                line,
                column,
            ));
            None
        }
        "array_contains" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!(
                        "array_contains() expects exactly 2 arguments, found {}",
                        args.len()
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_contains",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            let element_ty = array_element_type(array_ty)?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_contains() value argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let probe_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(element_ty),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            let valid = if element_ty == VarType::Decimal {
                is_numeric_type(probe_ty)
            } else {
                is_assignable(element_ty, probe_ty)
            };
            if !valid {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_contains() value type {} is incompatible with {}",
                        type_name(probe_ty),
                        type_name(element_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            Some(VarType::Boolean)
        }
        "array_size" => {
            if args.len() != 1 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_size() expects exactly 1 argument, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let _ = infer_array_argument_type(
                &args[0],
                "array_size",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            Some(VarType::Integer)
        }
        "array_join" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_join() expects exactly 2 arguments, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let _ = infer_array_argument_type(
                &args[0],
                "array_join",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_join() separator argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let separator_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(VarType::String),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if separator_ty != VarType::String {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_join() separator must be string, found {}",
                        type_name(separator_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            Some(VarType::String)
        }
        "array_get" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!("array_get() expects exactly 2 arguments, found {}", args.len()),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_get",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_get() index argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let index_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(VarType::Integer),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if index_ty != VarType::Integer {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_get() index must be integer, found {}",
                        type_name(index_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            array_element_type(array_ty)
        }
        "array_insert" => {
            if args.len() != 3 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!(
                        "array_insert() expects exactly 3 arguments, found {}",
                        args.len()
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_insert() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_insert",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            let element_ty = array_element_type(array_ty)?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_insert() index argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
            let index_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(VarType::Integer),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if index_ty != VarType::Integer {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_insert() index must be integer, found {}",
                        type_name(index_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !is_scalar_argument_source(&args[2]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_insert() value argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
            let value_ty = infer_expr_type(
                &args[2],
                line,
                column,
                Some(element_ty),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if !is_assignable(element_ty, value_ty) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_insert() value type {} is incompatible with {}",
                        type_name(value_ty),
                        type_name(element_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if allow_void_return {
                return None;
            }

            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionContextInvalid,
                "array_insert() returns void and cannot be used as an expression",
                Phase::Validation,
                scene,
                line,
                column,
            ));
            None
        }
        "array_remove" => {
            if args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!(
                        "array_remove() expects exactly 2 arguments, found {}",
                        args.len()
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !allow_mutating_calls {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionContextInvalid,
                    "array_remove() is not allowed in this phase",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_ty = infer_array_argument_type(
                &args[0],
                "array_remove",
                line,
                column,
                None,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if !is_scalar_argument_source(&args[1]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "array_remove() index argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
            let index_ty = infer_expr_type(
                &args[1],
                line,
                column,
                Some(VarType::Integer),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if index_ty != VarType::Integer {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "array_remove() index must be integer, found {}",
                        type_name(index_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            array_element_type(array_ty)
        }
        "pick" => {
            if args.len() != 1 && args.len() != 2 {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArityInvalid,
                    format!(
                        "pick() expects 1 or 2 arguments, found {}",
                        args.len()
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if args.len() == 1 {
                if let Expr::ListLit { items, .. } = &args[0] {
                    if items.is_empty() {
                        diags.push(Diagnostic::new(
                            DiagnosticCode::EListEmpty,
                            "pick() requires a non-empty candidate list",
                            Phase::Validation,
                            scene,
                            line,
                            column,
                        ));
                        return None;
                    }
                }

                let array_ty = infer_array_argument_type(
                    &args[0],
                    "pick",
                    line,
                    column,
                    None,
                    allow_mutating_calls,
                    allow_user_logic_calls,
                    logic_signatures,
                    declared_vars,
                    scene,
                    diags,
                )?;
                return array_element_type(array_ty);
            }

            if !is_scalar_argument_source(&args[0]) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "pick(count, array) count argument must be a literal or $variable",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
            let count_ty = infer_expr_type(
                &args[0],
                line,
                column,
                Some(VarType::Integer),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            if count_ty != VarType::Integer {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "pick(count, array) requires integer count, found {}",
                        type_name(count_ty)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            let array_target_hint = assignment_target.filter(|t| is_array_type(*t));
            let array_ty = infer_array_argument_type(
                &args[1],
                "pick",
                line,
                column,
                array_target_hint,
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;
            Some(array_ty)
        }
        _ => {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionUnknown,
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

fn infer_list_literal_type(
    items: &[Expr],
    line: usize,
    column: usize,
    assignment_target: Option<VarType>,
    allow_mutating_calls: bool,
    allow_user_logic_calls: bool,
    logic_signatures: &LogicSignatures,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<VarType> {
    let expected_element = assignment_target.and_then(array_element_type);

    if items.is_empty() {
        if let Some(element_type) = expected_element {
            return array_type_for_element(element_type);
        }

        diags.push(Diagnostic::new(
            DiagnosticCode::EFunctionContextInvalid,
            "Empty array literal [] requires known target array type context",
            Phase::Validation,
            scene,
            line,
            column,
        ));
        return None;
    }

    if let Some(expected) = expected_element {
        for item in items {
            let item_type = infer_expr_type(
                item,
                line,
                column,
                Some(expected),
                allow_mutating_calls,
                allow_user_logic_calls,
                logic_signatures,
                declared_vars,
                scene,
                diags,
            )?;

            if is_array_type(item_type) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    "Nested arrays are not supported",
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }

            if !is_assignable(expected, item_type) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "Array literal element type {} is incompatible with {}",
                        type_name(item_type),
                        type_name(expected)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
        }

        return array_type_for_element(expected);
    }

    let mut inferred_element: Option<VarType> = None;
    for item in items {
        let item_type = infer_expr_type(
            item,
            line,
            column,
            None,
            allow_mutating_calls,
            allow_user_logic_calls,
            logic_signatures,
            declared_vars,
            scene,
            diags,
        )?;

        if is_array_type(item_type) {
            diags.push(Diagnostic::new(
                DiagnosticCode::EFunctionArgumentInvalid,
                "Nested arrays are not supported",
                Phase::Validation,
                scene,
                line,
                column,
            ));
            return None;
        }

        inferred_element = match inferred_element {
            None => Some(item_type),
            Some(current) if current == item_type => Some(current),
            Some(current) if is_numeric_type(current) && is_numeric_type(item_type) => {
                Some(VarType::Decimal)
            }
            Some(current) => {
                diags.push(Diagnostic::new(
                    DiagnosticCode::EFunctionArgumentInvalid,
                    format!(
                        "Array literal elements must share one scalar type, found {} and {}",
                        type_name(current),
                        type_name(item_type)
                    ),
                    Phase::Validation,
                    scene,
                    line,
                    column,
                ));
                return None;
            }
        };
    }

    array_type_for_element(inferred_element?)
}

fn infer_array_argument_type(
    expr: &Expr,
    function_name: &str,
    line: usize,
    column: usize,
    assignment_target_hint: Option<VarType>,
    allow_mutating_calls: bool,
    allow_user_logic_calls: bool,
    logic_signatures: &LogicSignatures,
    declared_vars: &VarTypes,
    scene: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<VarType> {
    if !is_array_argument_source(expr) {
        diags.push(Diagnostic::new(
            DiagnosticCode::EFunctionArgumentInvalid,
            format!(
                "{}() array argument must be a $variable or array literal",
                function_name
            ),
            Phase::Validation,
            scene,
            line,
            column,
        ));
        return None;
    }

    let ty = infer_expr_type(
        expr,
        line,
        column,
        assignment_target_hint,
        allow_mutating_calls,
        allow_user_logic_calls,
        logic_signatures,
        declared_vars,
        scene,
        diags,
    )?;

    if !is_array_type(ty) {
        diags.push(Diagnostic::new(
            DiagnosticCode::EFunctionArgumentInvalid,
            format!("{}() requires array argument", function_name),
            Phase::Validation,
            scene,
            line,
            column,
        ));
        return None;
    }

    Some(ty)
}

fn is_scalar_argument_source(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::IntLit(_)
            | Expr::DecimalLit(_)
            | Expr::BoolLit(_)
            | Expr::StringLit(_)
            | Expr::VarRef { .. }
    )
}

fn is_array_argument_source(expr: &Expr) -> bool {
    matches!(expr, Expr::VarRef { .. } | Expr::ListLit { .. })
}

fn array_element_type(var_type: VarType) -> Option<VarType> {
    match var_type {
        VarType::ArrayInteger => Some(VarType::Integer),
        VarType::ArrayString => Some(VarType::String),
        VarType::ArrayBoolean => Some(VarType::Boolean),
        VarType::ArrayDecimal => Some(VarType::Decimal),
        _ => None,
    }
}

fn array_type_for_element(element_type: VarType) -> Option<VarType> {
    match element_type {
        VarType::Integer => Some(VarType::ArrayInteger),
        VarType::String => Some(VarType::ArrayString),
        VarType::Boolean => Some(VarType::ArrayBoolean),
        VarType::Decimal => Some(VarType::ArrayDecimal),
        _ => None,
    }
}

fn is_array_type(var_type: VarType) -> bool {
    matches!(
        var_type,
        VarType::ArrayInteger
            | VarType::ArrayString
            | VarType::ArrayBoolean
            | VarType::ArrayDecimal
    )
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
        VarType::ArrayInteger => "array<integer>",
        VarType::ArrayString => "array<string>",
        VarType::ArrayBoolean => "array<boolean>",
        VarType::ArrayDecimal => "array<decimal>",
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
        StoryStatement::ForSnapshot(_) | StoryStatement::Repeat(_) => true,
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
        let script = parser.parse();
        let mut all = lexer.diagnostics;
        all.extend(parser.diagnostics);
        if let Some(script) = script {
            all.extend(validate(&script));
        }
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
            d.code == DiagnosticCode::EFunctionContextInvalid
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
            d.code == DiagnosticCode::EListEmpty
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

    #[test]
    fn test_scene_local_visible_in_same_scene_story() {
        let src = r#"
* INIT {
    $global_count as integer = 3
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $damage as integer = $global_count + 2

    #STORY
    "Damage was ${damage}"
    $damage
    @end
}
"#;
        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_scene_local_not_visible_in_other_scene() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start first
}
* first {
    #PREP
    $temp as integer = 10

    #STORY
    @jump second
}
* second {
    #STORY
    "Temp=${temp}"
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EVariableUndeclaredRead
                && d.message.contains("$temp")
                && d.scene == "second"
        }));
    }

    #[test]
    fn test_scene_local_global_collision_rejected() {
        let src = r#"
* INIT {
    $score as integer = 10
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $score as integer = 1

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EVariableScopeConflict));
    }

    #[test]
    fn test_scene_local_duplicate_rejected() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $x as integer = 1
    $x as integer = 2

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::ELocalDuplicate));
    }

    #[test]
    fn test_scene_local_declaration_forbidden_in_story() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #STORY
    $x as integer = 1
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EPhaseTokenForbidden
                && d.message.contains("declaration is forbidden in #STORY")
        }));
    }

    #[test]
    fn test_else_if_chain_valid_in_prep_and_story() {
        let src = r#"
* INIT {
    $score as integer = 42
    @actor A "Alice"
    @start main
}
* path_low {
    #STORY
    @end
}
* path_mid {
    #STORY
    @end
}
* path_high {
    #STORY
    @end
}
* main {
    #PREP
    if ($score < 30) {
        $score += 1
    } else if ($score < 60) {
        $score += 2
    } else {
        $score += 3
    }

    #STORY
    if ($score < 30) {
        @jump path_low
    } else if ($score < 60) {
        @jump path_mid
    } else {
        @jump path_high
    }
}
"#;
        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_else_if_condition_must_be_boolean() {
        let src = r#"
* INIT {
    $flag as boolean = true
    @actor A "Alice"
    @start main
}
* next {
    #STORY
    @end
}
* main {
    #STORY
    if ($flag == true) {
        @jump next
    } else if ("not-boolean") {
        @jump next
    } else {
        @end
    }
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EConditionTypeInvalid));
    }

    #[test]
    fn test_else_if_story_fallthrough_rejected() {
        let src = r#"
* INIT {
    $a as boolean = false
    $b as boolean = false
    @actor A "Alice"
    @start main
}
* alt {
    #STORY
    @end
}
* main {
    #STORY
    if ($a == true) {
        @jump alt
    } else if ($b == true) {
        @jump alt
    }
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EStoryUnterminatedPath));
    }

    #[test]
    fn test_else_if_malformed_syntax_reports_esyntax() {
        let src = r#"
* INIT {
    $score as integer = 1
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    if ($score > 0) {
        $score = 2
    } else if $score < 10 {
        $score = 3
    }

    #STORY
    @end
}
"#;
        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::ESyntax));
    }

    #[test]
    fn test_array_collection_happy_path() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2, 3]
    $prices as array<decimal> = [1, 2.5]
    $count as integer = 2
    $sep as string = ", "
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    array_push($nums, 4)
    $picked as array<integer> = pick($count, $nums)
    $has_price as boolean = array_contains($prices, 2)
    $joined as string = array_join($nums, $sep)
    $first as integer = array_get($nums, 0)
    $removed as integer = array_remove($nums, 1)

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_array_scalar_argument_shape_rejected() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2, 3]
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    $v as integer = array_get($nums, abs(1))

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EFunctionArgumentInvalid
                && d.message.contains("literal or $variable")
        }));
    }

    #[test]
    fn test_mutating_array_call_forbidden_in_story() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2, 3]
    @actor A "Alice"
    @start main
}
* next {
    #STORY
    @end
}
* main {
    #STORY
    if (array_pop($nums) == 1) {
        @jump next
    } else {
        @end
    }
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EFunctionContextInvalid
                && d.message.contains("not allowed in this phase")
        }));
    }

    #[test]
    fn test_break_outside_loop_rejected() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    break

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::ELoopControlOutsideLoop));
    }

    #[test]
    fn test_repeat_literal_non_positive_rejected() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    repeat (0) {
        continue
    }

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::ERangeInvalid));
    }

    #[test]
    fn test_loop_iterator_assignment_rejected() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2]
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    for ($item in snapshot $nums) {
        $item = 9
    }

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::ELoopIteratorReadOnly));
    }

    #[test]
    fn test_valid_loops_in_prep_and_story() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2, 3]
    $count as integer = 2
    @actor A "Alice"
    @start main
}
* main {
    #PREP
    repeat ($count) {
        continue
    }

    for ($item in snapshot $nums) {
        if ($item == 2) {
            break
        }
    }

    #STORY
    for ($item in snapshot $nums) {
        if ($item == 1) {
            continue
        }
    }
    @end
}
"#;

        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_choice_nested_for_repeat_if_valid() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2, 3]
    @actor A "Alice"
    @start main
}
* done {
    #STORY
    @end
}
* main {
    #STORY
    @choice {
        for ($item in snapshot $nums) {
            if ($item > 1) {
                repeat (2) {
                    "Pick ${item}" -> done;
                }
            }
        }

        "Fallback" -> done;
    }
}
"#;

        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_choice_for_snapshot_source_non_array_rejected() {
        let src = r#"
* INIT {
    $count as integer = 2
    @actor A "Alice"
    @start main
}
* done {
    #STORY
    @end
}
* main {
    #STORY
    @choice {
        for ($item in snapshot $count) {
            "Invalid" -> done;
        }
    }
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EFunctionArgumentInvalid
                && d.message.contains("snapshot source")
        }));
    }

    #[test]
    fn test_choice_for_iterator_scope_limited_to_body() {
        let src = r#"
* INIT {
    $nums as array<integer> = [1, 2]
    @actor A "Alice"
    @start main
}
* done {
    #STORY
    @end
}
* main {
    #STORY
    @choice {
        for ($item in snapshot $nums) {
            "Inside ${item}" -> done;
        }

        "Outside ${item}" -> done;
    }
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EVariableUndeclaredRead && d.message.contains("$item")
        }));
    }

    #[test]
    fn test_choice_repeat_zero_rejected() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}
* done {
    #STORY
    @end
}
* main {
    #STORY
    @choice {
        repeat (0) {
            "Never" -> done;
        }
    }
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| d.code == DiagnosticCode::ERangeInvalid));
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EChoiceStaticEmpty));
    }

    #[test]
    fn test_choice_for_iterator_collision_rejected() {
        let src = r#"
* INIT {
    $item as integer = 10
    $nums as array<integer> = [1, 2]
    @actor A "Alice"
    @start main
}
* done {
    #STORY
    @end
}
* main {
    #STORY
    @choice {
        for ($item in snapshot $nums) {
            "x" -> done;
        }
    }
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EVariableScopeConflict));
    }

    #[test]
    fn test_logic_happy_path_void_and_typed_return() {
        let src = r#"
* INIT {
    $system_stability as integer = 45
    @actor TEO "Teona"
    @start main
}

logic apply_damage($amount as integer) {
    $system_stability = $system_stability - $amount
}

logic get_variance($modifier as integer) -> integer {
    $total as integer = $system_stability + $modifier
    return $total
}

* main {
    #PREP
    apply_damage(10)
    $v as integer = get_variance(15)

    #STORY
    TEO: "V=${v}"
    @end
}
"#;

        let diags = parse_and_validate(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_logic_call_forbidden_in_init() {
        let src = r#"
* INIT {
    $seed as integer = compute_seed(2)
    @actor A "Alice"
    @start main
}

logic compute_seed($value as integer) -> integer {
    return $value + 1
}

* main {
    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags.iter().any(|d| {
            d.code == DiagnosticCode::EFunctionContextInvalid
                && d.message.contains("not allowed in this phase")
        }));
    }

    #[test]
    fn test_logic_missing_return_rejected() {
        let src = r#"
* INIT {
    $x as integer = 1
    @actor A "Alice"
    @start main
}

logic bump($v as integer) -> integer {
    $x = $x + $v
}

* main {
    #PREP
    $k as integer = bump(3)

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EFunctionReturnMissing));
    }

    #[test]
    fn test_logic_recursion_rejected() {
        let src = r#"
* INIT {
    $x as integer = 1
    @actor A "Alice"
    @start main
}

logic a($n as integer) -> integer {
    return b($n)
}

logic b($n as integer) -> integer {
    return a($n)
}

* main {
    #PREP
    $k as integer = a(1)

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EFunctionRecursionForbidden));
    }

    #[test]
    fn test_return_forbidden_in_prep() {
        let src = r#"
* INIT {
    @actor A "Alice"
    @start main
}

* main {
    #PREP
    return 1

    #STORY
    @end
}
"#;

        let diags = parse_and_validate(src);
        assert!(diags
            .iter()
            .any(|d| d.code == DiagnosticCode::EPhaseTokenForbidden));
    }
}
