use std::collections::{HashMap, VecDeque};

use rand::RngExt;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use storyscript_parser::ast::*;
use storyscript_parser::interpolation::{ESCAPED_DOLLAR_MARKER, render_interpolated};

// ---------------------------------------------------------------------------
// Runtime value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Decimal(Decimal),
    Bool(bool),
    Str(String),
    Array {
        items: Vec<Value>,
        element_type: VarType,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Decimal(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Array {
                items,
                element_type: _,
            } => {
                let rendered: Vec<String> = items.iter().map(value_to_plain_text).collect();
                write!(f, "[{}]", rendered.join(", "))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Actor info (runtime)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ActorInfo {
    pub display_name: String,
}

// ---------------------------------------------------------------------------
// Step results (what the UI renders)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ChoiceDisplay {
    pub text: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum StepResult {
    Narration(String),
    Dialogue {
        actor_name: String,
        actor_id: String,
        emotion: Option<String>,
        position: Option<String>,
        text: String,
    },
    Choices(Vec<ChoiceDisplay>),
    End,
}

// ---------------------------------------------------------------------------
// Internal events (before filtering)
// ---------------------------------------------------------------------------

enum InternalEvent {
    Narration(String),
    Dialogue {
        actor_name: String,
        actor_id: String,
        emotion: Option<String>,
        position: Option<String>,
        text: String,
    },
    Choices(Vec<ChoiceDisplay>),
    Jump(String),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallMode {
    Expression,
    Statement,
}

#[derive(Debug, Clone, PartialEq)]
enum PrepFlow {
    Next,
    BreakLoop,
    ContinueLoop,
    Return(Option<Value>),
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoryFlow {
    Open,
    BreakLoop,
    ContinueLoop,
    Terminated,
    Error,
}

const CHOICE_OPTION_CAP: usize = 9;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct Engine {
    scenes: HashMap<String, Scene>,
    logic_blocks: HashMap<String, LogicBlock>,
    actors: HashMap<String, ActorInfo>,
    pub variables: HashMap<String, Value>,
    var_types: HashMap<String, VarType>,
    local_variables: HashMap<String, Value>,
    local_var_types: HashMap<String, VarType>,
    pub current_scene: String,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pending: VecDeque<InternalEvent>,
    pub finished: bool,
}

impl Engine {
    pub fn new(script: &Script) -> Self {
        // Initialize variables and immutable types from INIT block.
        let mut variables = HashMap::new();
        let mut var_types = HashMap::new();
        for var in &script.init.variables {
            var_types.insert(var.name.clone(), var.var_type);

            let value = eval_init_expr(&var.value, &variables, Some(var.var_type))
                .and_then(|v| coerce_value_for_type(v, var.var_type))
                .unwrap_or_else(|| default_value_for_type(var.var_type));
            variables.insert(var.name.clone(), value);
        }

        // Build actor map
        let mut actors = HashMap::new();
        for actor in &script.init.actors {
            actors.insert(
                actor.id.clone(),
                ActorInfo {
                    display_name: actor.display_name.clone(),
                },
            );
        }

        // Build scene map
        let mut scenes = HashMap::new();
        for scene in &script.scenes {
            scenes.insert(scene.label.clone(), scene.clone());
        }

        let mut logic_blocks = HashMap::new();
        for logic in &script.logic_blocks {
            logic_blocks.insert(logic.name.clone(), logic.clone());
        }

        let start = script.init.start.target.clone();

        let mut engine = Engine {
            scenes,
            logic_blocks,
            actors,
            variables,
            var_types,
            local_variables: HashMap::new(),
            local_var_types: HashMap::new(),
            current_scene: String::new(),
            bg: None,
            bgm: None,
            pending: VecDeque::new(),
            finished: false,
        };

        engine.enter_scene(&start);
        engine
    }

    // -----------------------------------------------------------------------
    // Scene entry
    // -----------------------------------------------------------------------

    fn enter_scene(&mut self, label: &str) {
        let scene = match self.scenes.get(label) {
            Some(s) => s.clone(),
            None => {
                self.finished = true;
                return;
            }
        };

        self.current_scene = label.to_string();
        self.local_variables.clear();
        self.local_var_types.clear();

        // Execute #PREP (silent — modifies state, sets assets)
        if let Some(prep) = &scene.prep {
            if !self.execute_prep(&prep.statements) {
                return;
            }
        }

        // Emit scene header
        self.pending.push_back(InternalEvent::Narration(format!(
            "─── Scene: {} ───",
            label
        )));

        // Flatten #STORY into pending events
        self.flatten_story(&scene.story.statements);
    }

    // -----------------------------------------------------------------------
    // #PREP execution
    // -----------------------------------------------------------------------

    fn execute_prep(&mut self, stmts: &[PrepStatement]) -> bool {
        matches!(self.execute_prep_block(stmts, false, false, None), PrepFlow::Next)
    }

    fn execute_prep_block(
        &mut self,
        stmts: &[PrepStatement],
        in_loop: bool,
        in_logic: bool,
        logic_return_type: Option<VarType>,
    ) -> PrepFlow {
        for stmt in stmts {
            match stmt {
                PrepStatement::BgDirective { path, .. } => {
                    let resolved = match self.resolve_string_or_error(path, "@bg path") {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };
                    self.bg = Some(resolved);
                }
                PrepStatement::BgmDirective { value, .. } => {
                    self.bgm = match value {
                        BgmValue::Path(p) => {
                            let resolved = match self.resolve_string_or_error(p, "@bgm path") {
                                Some(value) => value,
                                None => return PrepFlow::Error,
                            };
                            Some(resolved)
                        }
                        BgmValue::Stop => None,
                    };
                }
                PrepStatement::SfxDirective { .. } => {
                    // SFX: can't play audio in TUI, skip
                }
                PrepStatement::VarDecl(decl) => {
                    if self.var_types.contains_key(&decl.name)
                        || self.local_var_types.contains_key(&decl.name)
                    {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "Duplicate variable declaration for '${}' in scene '{}'",
                                decl.name, self.current_scene
                            ),
                        );
                        return PrepFlow::Error;
                    }

                    let value = match self.eval_expr(&decl.value, Some(decl.var_type)) {
                        Some(v) => v,
                        None => return PrepFlow::Error,
                    };
                    let value_type = value_type(&value);
                    let coerced = match coerce_value_for_type(value, decl.var_type) {
                        Some(v) => v,
                        None => {
                            self.raise_runtime_error(
                                "RUNTIME",
                                format!(
                                    "Type mismatch initializing local ${}. Expected {}, got {}",
                                    decl.name,
                                    type_name(decl.var_type),
                                    type_name(value_type)
                                ),
                            );
                            return PrepFlow::Error;
                        }
                    };

                    self.local_var_types.insert(decl.name.clone(), decl.var_type);
                    self.local_variables.insert(decl.name.clone(), coerced);
                }
                PrepStatement::VarAssign(assign) => {
                    let declared_type = match self.resolve_var_type(&assign.name) {
                        Some(ty) => ty,
                        None => {
                            self.raise_runtime_error(
                                "RUNTIME",
                                format!("Unknown variable '${}' in PREP assignment", assign.name),
                            );
                            return PrepFlow::Error;
                        }
                    };

                    let rhs = match self.eval_expr(&assign.value, Some(declared_type)) {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };
                    let rhs_type = value_type(&rhs);

                    match assign.op {
                        AssignOp::Set => {
                            let coerced = match coerce_value_for_type(rhs, declared_type) {
                                Some(value) => value,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Type mismatch assigning to ${}. Expected {}, got {}",
                                            assign.name,
                                            type_name(declared_type),
                                            type_name(rhs_type)
                                        ),
                                    );
                                    return PrepFlow::Error;
                                }
                            };
                            self.write_variable(&assign.name, coerced);
                        }
                        AssignOp::AddEq | AssignOp::SubEq => {
                            let current = match self.resolve_var_value(&assign.name).cloned() {
                                Some(value) => value,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Variable '${}' missing in runtime state",
                                            assign.name
                                        ),
                                    );
                                    return PrepFlow::Error;
                                }
                            };

                            match declared_type {
                                VarType::Integer => {
                                    let (a, b) = match (current, rhs.clone()) {
                                        (Value::Int(a), Value::Int(b)) => (a, b),
                                        (_, other) => {
                                            self.raise_runtime_error(
                                                "RUNTIME",
                                                format!(
                                                    "'{}' on ${} requires integer RHS, got {}",
                                                    match assign.op {
                                                        AssignOp::AddEq => "+=",
                                                        AssignOp::SubEq => "-=",
                                                        AssignOp::Set => "=",
                                                    },
                                                    assign.name,
                                                    type_name(value_type(&other))
                                                ),
                                            );
                                            return PrepFlow::Error;
                                        }
                                    };

                                    let updated = match assign.op {
                                        AssignOp::AddEq => Value::Int(a + b),
                                        AssignOp::SubEq => Value::Int(a - b),
                                        AssignOp::Set => Value::Int(a),
                                    };
                                    self.write_variable(&assign.name, updated);
                                }
                                VarType::Decimal => {
                                    let current_num = match as_decimal(&current) {
                                        Some(value) => value,
                                        None => {
                                            self.raise_runtime_error(
                                                "RUNTIME",
                                                format!(
                                                    "Variable ${} expected decimal value in runtime state",
                                                    assign.name
                                                ),
                                            );
                                            return PrepFlow::Error;
                                        }
                                    };
                                    let rhs_num = match as_decimal(&rhs) {
                                        Some(value) => value,
                                        None => {
                                            self.raise_runtime_error(
                                                "RUNTIME",
                                                format!(
                                                    "'{}' on ${} requires numeric RHS, got {}",
                                                    match assign.op {
                                                        AssignOp::AddEq => "+=",
                                                        AssignOp::SubEq => "-=",
                                                        AssignOp::Set => "=",
                                                    },
                                                    assign.name,
                                                    type_name(value_type(&rhs))
                                                ),
                                            );
                                            return PrepFlow::Error;
                                        }
                                    };

                                    let updated = match assign.op {
                                        AssignOp::AddEq => Value::Decimal(current_num + rhs_num),
                                        AssignOp::SubEq => Value::Decimal(current_num - rhs_num),
                                        AssignOp::Set => Value::Decimal(current_num),
                                    };
                                    self.write_variable(&assign.name, updated);
                                }
                                VarType::String
                                | VarType::Boolean
                                | VarType::ArrayInteger
                                | VarType::ArrayString
                                | VarType::ArrayBoolean
                                | VarType::ArrayDecimal => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "'{}' is invalid for variable ${} of type {}",
                                            match assign.op {
                                                AssignOp::AddEq => "+=",
                                                AssignOp::SubEq => "-=",
                                                AssignOp::Set => "=",
                                            },
                                            assign.name,
                                            type_name(declared_type)
                                        ),
                                    );
                                    return PrepFlow::Error;
                                }
                            }
                        }
                    }
                }
                PrepStatement::Call {
                    name,
                    args,
                    line,
                    column,
                } => {
                    if self
                        .eval_call(name, args, *line, *column, None, CallMode::Statement)
                        .is_none()
                    {
                        return PrepFlow::Error;
                    }
                }
                PrepStatement::IfElse(if_else) => {
                    let condition = match self.eval_bool(&if_else.condition) {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };

                    let branch_flow = if condition {
                        self.execute_prep_block(
                            &if_else.then_branch,
                            in_loop,
                            in_logic,
                            logic_return_type,
                        )
                    } else if let Some(else_branch) = &if_else.else_branch {
                        self.execute_prep_block(
                            else_branch,
                            in_loop,
                            in_logic,
                            logic_return_type,
                        )
                    } else {
                        PrepFlow::Next
                    };

                    if branch_flow != PrepFlow::Next {
                        return branch_flow;
                    }
                }
                PrepStatement::ForSnapshot(loop_stmt) => {
                    let (snapshot_items, element_type) =
                        match self.resolve_snapshot_array(&loop_stmt.array_name) {
                            Some(result) => result,
                            None => return PrepFlow::Error,
                        };

                    let previous_type =
                        self.local_var_types.insert(loop_stmt.item_name.clone(), element_type);
                    let previous_value = self.local_variables.remove(&loop_stmt.item_name);

                    for item in snapshot_items {
                        self.local_variables.insert(loop_stmt.item_name.clone(), item);

                        match self.execute_prep_block(
                            &loop_stmt.body,
                            true,
                            in_logic,
                            logic_return_type,
                        ) {
                            PrepFlow::Next => {}
                            PrepFlow::ContinueLoop => continue,
                            PrepFlow::BreakLoop => break,
                            PrepFlow::Return(value) => {
                                self.restore_loop_binding(
                                    &loop_stmt.item_name,
                                    previous_type,
                                    previous_value,
                                );
                                return PrepFlow::Return(value);
                            }
                            PrepFlow::Error => {
                                self.restore_loop_binding(
                                    &loop_stmt.item_name,
                                    previous_type,
                                    previous_value,
                                );
                                return PrepFlow::Error;
                            }
                        }
                    }

                    self.restore_loop_binding(&loop_stmt.item_name, previous_type, previous_value);
                }
                PrepStatement::Repeat(repeat_stmt) => {
                    let count = match self.eval_repeat_count(&repeat_stmt.count) {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };

                    for _ in 0..count {
                        match self.execute_prep_block(
                            &repeat_stmt.body,
                            true,
                            in_logic,
                            logic_return_type,
                        ) {
                            PrepFlow::Next => {}
                            PrepFlow::ContinueLoop => continue,
                            PrepFlow::BreakLoop => break,
                            PrepFlow::Return(value) => return PrepFlow::Return(value),
                            PrepFlow::Error => return PrepFlow::Error,
                        }
                    }
                }
                PrepStatement::Break { .. } => {
                    if in_loop {
                        return PrepFlow::BreakLoop;
                    }
                    self.raise_runtime_error(
                        "RUNTIME",
                        "break is only valid inside loop bodies".to_string(),
                    );
                    return PrepFlow::Error;
                }
                PrepStatement::Continue { .. } => {
                    if in_loop {
                        return PrepFlow::ContinueLoop;
                    }
                    self.raise_runtime_error(
                        "RUNTIME",
                        "continue is only valid inside loop bodies".to_string(),
                    );
                    return PrepFlow::Error;
                }
                PrepStatement::Return { value, .. } => {
                    if !in_logic {
                        self.raise_runtime_error(
                            "RUNTIME",
                            "return is only valid inside logic blocks".to_string(),
                        );
                        return PrepFlow::Error;
                    }

                    let returned = match (logic_return_type, value) {
                        (None, None) => None,
                        (None, Some(_)) => {
                            self.raise_runtime_error(
                                "RUNTIME",
                                "Void logic function cannot return a value".to_string(),
                            );
                            return PrepFlow::Error;
                        }
                        (Some(_), None) => {
                            self.raise_runtime_error(
                                "RUNTIME",
                                "Typed logic function must return a value".to_string(),
                            );
                            return PrepFlow::Error;
                        }
                        (Some(expected), Some(expr)) => {
                            let raw = match self.eval_expr(expr, Some(expected)) {
                                Some(v) => v,
                                None => return PrepFlow::Error,
                            };
                            match coerce_value_for_type(raw.clone(), expected) {
                                Some(v) => Some(v),
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "return expression type {} is incompatible with {}",
                                            type_name(value_type(&raw)),
                                            type_name(expected)
                                        ),
                                    );
                                    return PrepFlow::Error;
                                }
                            }
                        }
                    };

                    return PrepFlow::Return(returned);
                }
            }
        }

        PrepFlow::Next
    }

    // -----------------------------------------------------------------------
    // #STORY flattening
    // -----------------------------------------------------------------------

    fn flatten_story(&mut self, stmts: &[StoryStatement]) {
        let _ = self.flatten_story_block(stmts, false);
    }

    fn flatten_story_block(&mut self, stmts: &[StoryStatement], in_loop: bool) -> StoryFlow {
        let len = stmts.len();

        for (idx, stmt) in stmts.iter().enumerate() {
            let is_last_stmt = idx + 1 == len;

            match stmt {
                StoryStatement::Narration { text, .. } => {
                    let resolved = match self.resolve_string_or_error(text, "narration") {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };
                    self.pending.push_back(InternalEvent::Narration(resolved));
                }
                StoryStatement::VarOutput { name, .. } => {
                    if let Some(value) = self.resolve_var_value(name) {
                        self.pending
                            .push_back(InternalEvent::Narration(Self::value_to_plain_text(value)));
                    } else {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!("Variable '${}' was missing during STORY output", name),
                        );
                        return StoryFlow::Error;
                    }
                }
                StoryStatement::Dialogue(dlg) => {
                    let actor_name_template = self
                        .actors
                        .get(&dlg.actor_id)
                        .map(|a| a.display_name.clone())
                        .unwrap_or_else(|| dlg.actor_id.clone());

                    let actor_name = match self
                        .resolve_string_or_error(&actor_name_template, "actor display name")
                    {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };

                    let (emotion, position) = match &dlg.form {
                        DialogueForm::NameOnly => (None, None),
                        DialogueForm::Portrait { emotion, position } => {
                            let pos_str = match position {
                                Position::Left => "Left",
                                Position::Center => "Center",
                                Position::Right => "Right",
                            };
                            (Some(emotion.clone()), Some(pos_str.to_string()))
                        }
                    };

                    let text = match self.resolve_string_or_error(&dlg.text, "dialogue") {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };

                    self.pending.push_back(InternalEvent::Dialogue {
                        actor_name,
                        actor_id: dlg.actor_id.clone(),
                        emotion,
                        position,
                        text,
                    });
                }
                StoryStatement::IfElse(if_else) => {
                    let condition = match self.eval_bool(&if_else.condition) {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };

                    let branch_flow = if condition {
                        self.flatten_story_block(&if_else.then_branch, in_loop)
                    } else if let Some(else_branch) = &if_else.else_branch {
                        self.flatten_story_block(else_branch, in_loop)
                    } else {
                        StoryFlow::Open
                    };

                    if branch_flow != StoryFlow::Open {
                        return branch_flow;
                    }
                }
                StoryStatement::ForSnapshot(loop_stmt) => {
                    let flow = self.flatten_story_for_snapshot(loop_stmt, is_last_stmt);
                    if flow != StoryFlow::Open {
                        return flow;
                    }
                }
                StoryStatement::Repeat(repeat_stmt) => {
                    let flow = self.flatten_story_repeat(repeat_stmt, is_last_stmt);
                    if flow != StoryFlow::Open {
                        return flow;
                    }
                }
                StoryStatement::Break { .. } => {
                    if in_loop {
                        return StoryFlow::BreakLoop;
                    }
                    self.raise_runtime_error(
                        "RUNTIME",
                        "break is only valid inside loop bodies".to_string(),
                    );
                    return StoryFlow::Error;
                }
                StoryStatement::Continue { .. } => {
                    if in_loop {
                        return StoryFlow::ContinueLoop;
                    }
                    self.raise_runtime_error(
                        "RUNTIME",
                        "continue is only valid inside loop bodies".to_string(),
                    );
                    return StoryFlow::Error;
                }
                StoryStatement::Choice(choice_block) => {
                    let mut options: Vec<ChoiceDisplay> = Vec::new();
                    let flow = self.flatten_choice_entries(&choice_block.entries, &mut options);
                    if flow != StoryFlow::Open {
                        return flow;
                    }

                    if options.is_empty() {
                        self.raise_runtime_error(
                            "R_CHOICE_EXHAUSTED",
                            "All @choice options were filtered out at runtime".to_string(),
                        );
                        return StoryFlow::Error;
                    }

                    self.pending.push_back(InternalEvent::Choices(options));
                    return StoryFlow::Terminated;
                }
                StoryStatement::Jump { target, .. } => {
                    self.pending.push_back(InternalEvent::Jump(target.clone()));
                    return StoryFlow::Terminated;
                }
                StoryStatement::End { .. } => {
                    self.pending.push_back(InternalEvent::End);
                    return StoryFlow::Terminated;
                }
                StoryStatement::SfxDirective { .. } => {
                    // SFX in STORY: skip in TUI mode
                }
            }
        }

        StoryFlow::Open
    }

    fn flatten_choice_entries(
        &mut self,
        entries: &[ChoiceEntry],
        options: &mut Vec<ChoiceDisplay>,
    ) -> StoryFlow {
        for entry in entries {
            let flow = self.flatten_choice_entry(entry, options);
            if flow != StoryFlow::Open {
                return flow;
            }
        }

        StoryFlow::Open
    }

    fn flatten_choice_entry(
        &mut self,
        entry: &ChoiceEntry,
        options: &mut Vec<ChoiceDisplay>,
    ) -> StoryFlow {
        match entry {
            ChoiceEntry::Option(opt) => {
                let text = match self.resolve_string_or_error(&opt.text, "choice label") {
                    Some(value) => value,
                    None => return StoryFlow::Error,
                };

                self.push_choice_option(options, text, opt.target.clone())
            }
            ChoiceEntry::If(if_entry) => {
                let condition = match self.eval_bool(&if_entry.condition) {
                    Some(value) => value,
                    None => return StoryFlow::Error,
                };
                if condition {
                    self.flatten_choice_entries(&if_entry.body, options)
                } else {
                    StoryFlow::Open
                }
            }
            ChoiceEntry::Repeat(repeat_entry) => {
                let count = match self.eval_repeat_count(&repeat_entry.count) {
                    Some(value) => value,
                    None => return StoryFlow::Error,
                };

                for _ in 0..count {
                    let flow = self.flatten_choice_entries(&repeat_entry.body, options);
                    if flow != StoryFlow::Open {
                        return flow;
                    }
                }

                StoryFlow::Open
            }
            ChoiceEntry::ForSnapshot(loop_entry) => {
                let (snapshot_items, element_type) =
                    match self.resolve_snapshot_array(&loop_entry.array_name) {
                        Some(result) => result,
                        None => return StoryFlow::Error,
                    };

                let previous_type = self
                    .local_var_types
                    .insert(loop_entry.item_name.clone(), element_type);
                let previous_value = self.local_variables.remove(&loop_entry.item_name);

                for item in snapshot_items {
                    self.local_variables.insert(loop_entry.item_name.clone(), item);
                    let flow = self.flatten_choice_entries(&loop_entry.body, options);
                    if flow != StoryFlow::Open {
                        self.restore_loop_binding(
                            &loop_entry.item_name,
                            previous_type,
                            previous_value,
                        );
                        return flow;
                    }
                }

                self.restore_loop_binding(&loop_entry.item_name, previous_type, previous_value);
                StoryFlow::Open
            }
        }
    }

    fn push_choice_option(
        &mut self,
        options: &mut Vec<ChoiceDisplay>,
        text: String,
        target: String,
    ) -> StoryFlow {
        if options.len() >= CHOICE_OPTION_CAP {
            self.raise_runtime_error(
                "R_CHOICE_OPTION_CAP_EXCEEDED",
                format!(
                    "@choice expanded to more than {} options at runtime",
                    CHOICE_OPTION_CAP
                ),
            );
            return StoryFlow::Error;
        }

        options.push(ChoiceDisplay { text, target });
        StoryFlow::Open
    }

    fn flatten_story_for_snapshot(
        &mut self,
        loop_stmt: &StoryForSnapshot,
        is_last_stmt: bool,
    ) -> StoryFlow {
        let (snapshot_items, element_type) = match self.resolve_snapshot_array(&loop_stmt.array_name)
        {
            Some(result) => result,
            None => return StoryFlow::Error,
        };

        let total_iterations = snapshot_items.len();
        let previous_type = self.local_var_types.insert(loop_stmt.item_name.clone(), element_type);
        let previous_value = self.local_variables.remove(&loop_stmt.item_name);

        for (idx, item) in snapshot_items.into_iter().enumerate() {
            self.local_variables.insert(loop_stmt.item_name.clone(), item);

            match self.flatten_story_block(&loop_stmt.body, true) {
                StoryFlow::Open => {}
                StoryFlow::ContinueLoop => continue,
                StoryFlow::BreakLoop => break,
                StoryFlow::Terminated => {
                    if idx + 1 < total_iterations {
                        self.raise_runtime_error(
                            "R_STORY_LOOP_TERMINATION_INVALID",
                            "Loop terminal directive would execute multiple times".to_string(),
                        );
                        self.restore_loop_binding(
                            &loop_stmt.item_name,
                            previous_type,
                            previous_value,
                        );
                        return StoryFlow::Error;
                    }

                    self.restore_loop_binding(
                        &loop_stmt.item_name,
                        previous_type,
                        previous_value,
                    );
                    return StoryFlow::Terminated;
                }
                StoryFlow::Error => {
                    self.restore_loop_binding(
                        &loop_stmt.item_name,
                        previous_type,
                        previous_value,
                    );
                    return StoryFlow::Error;
                }
            }
        }

        self.restore_loop_binding(&loop_stmt.item_name, previous_type, previous_value);

        if is_last_stmt {
            self.raise_runtime_error(
                "R_STORY_LOOP_TERMINATION_INVALID",
                "Loop completed without emitting @choice, @jump, or @end".to_string(),
            );
            return StoryFlow::Error;
        }

        StoryFlow::Open
    }

    fn flatten_story_repeat(&mut self, repeat_stmt: &StoryRepeat, is_last_stmt: bool) -> StoryFlow {
        let count = match self.eval_repeat_count(&repeat_stmt.count) {
            Some(value) => value,
            None => return StoryFlow::Error,
        };

        for idx in 0..count {
            match self.flatten_story_block(&repeat_stmt.body, true) {
                StoryFlow::Open => {}
                StoryFlow::ContinueLoop => continue,
                StoryFlow::BreakLoop => break,
                StoryFlow::Terminated => {
                    if idx + 1 < count {
                        self.raise_runtime_error(
                            "R_STORY_LOOP_TERMINATION_INVALID",
                            "Loop terminal directive would execute multiple times".to_string(),
                        );
                        return StoryFlow::Error;
                    }
                    return StoryFlow::Terminated;
                }
                StoryFlow::Error => return StoryFlow::Error,
            }
        }

        if is_last_stmt {
            self.raise_runtime_error(
                "R_STORY_LOOP_TERMINATION_INVALID",
                "Loop completed without emitting @choice, @jump, or @end".to_string(),
            );
            return StoryFlow::Error;
        }

        StoryFlow::Open
    }

    // -----------------------------------------------------------------------
    // Stepping
    // -----------------------------------------------------------------------

    /// Advance the engine to the next displayable event.
    /// Returns None only when the story is fully exhausted.
    pub fn step(&mut self) -> Option<StepResult> {
        loop {
            match self.pending.pop_front() {
                Some(InternalEvent::Jump(target)) => {
                    self.pending.clear();
                    self.enter_scene(&target);
                    continue;
                }
                Some(InternalEvent::Narration(text)) => {
                    return Some(StepResult::Narration(text));
                }
                Some(InternalEvent::Dialogue {
                    actor_name,
                    actor_id,
                    emotion,
                    position,
                    text,
                }) => {
                    return Some(StepResult::Dialogue {
                        actor_name,
                        actor_id,
                        emotion,
                        position,
                        text,
                    });
                }
                Some(InternalEvent::Choices(options)) => {
                    return Some(StepResult::Choices(options));
                }
                Some(InternalEvent::End) => {
                    self.finished = true;
                    return Some(StepResult::End);
                }
                None => {
                    return None;
                }
            }
        }
    }

    /// Execute a player's choice — enter the target scene.
    pub fn select_choice(&mut self, choice: &ChoiceDisplay) {
        self.pending.clear();
        self.enter_scene(&choice.target);
    }

    // -----------------------------------------------------------------------
    // Expression evaluation
    // -----------------------------------------------------------------------

    fn eval_expr(&mut self, expr: &Expr, assignment_target: Option<VarType>) -> Option<Value> {
        match expr {
            Expr::IntLit(n) => Some(Value::Int(*n)),
            Expr::DecimalLit(n) => Some(Value::Decimal(*n)),
            Expr::BoolLit(b) => Some(Value::Bool(*b)),
            Expr::StringLit(s) => self
                .resolve_string_or_error(s, "string expression")
                .map(Value::Str),
            Expr::VarRef { name, .. } => {
                if let Some(value) = self.resolve_var_value(name).cloned() {
                    Some(value)
                } else {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("Read of undeclared variable '${}'", name),
                    );
                    None
                }
            }
            Expr::Call {
                name,
                args,
                line,
                column,
            } => self.eval_call(
                name,
                args,
                *line,
                *column,
                assignment_target,
                CallMode::Expression,
            ),
            Expr::ListLit { items, .. } => self.eval_array_literal(items, assignment_target),
            Expr::BinOp { left, op, right } => {
                let l = self.eval_expr(left, assignment_target)?;
                let r = self.eval_expr(right, assignment_target)?;
                match op {
                    BinOperator::Add => self.eval_numeric_binop("+", l, r),
                    BinOperator::Sub => self.eval_numeric_binop("-", l, r),
                    BinOperator::Mul => self.eval_numeric_binop("*", l, r),
                    BinOperator::Div => self.eval_numeric_binop("/", l, r),
                    BinOperator::Mod => self.eval_numeric_binop("%", l, r),
                    BinOperator::EqEq => self.eval_equality_binop("==", l, r, true),
                    BinOperator::NotEq => self.eval_equality_binop("!=", l, r, false),
                    BinOperator::Lt => self.eval_relational_binop("<", l, r, |a, b| a < b),
                    BinOperator::LtEq => self.eval_relational_binop("<=", l, r, |a, b| a <= b),
                    BinOperator::Gt => self.eval_relational_binop(">", l, r, |a, b| a > b),
                    BinOperator::GtEq => self.eval_relational_binop(">=", l, r, |a, b| a >= b),
                }
            }
        }
    }

    fn eval_bool(&mut self, expr: &Expr) -> Option<bool> {
        match self.eval_expr(expr, None)? {
            Value::Bool(b) => Some(b),
            other => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Condition must evaluate to boolean, got {}",
                        type_name(value_type(&other))
                    ),
                );
                None
            }
        }
    }

    fn eval_numeric_binop(&mut self, op: &str, left: Value, right: Value) -> Option<Value> {
        if let (Value::Int(a), Value::Int(b)) = (&left, &right) {
            return Some(match op {
                "+" => Value::Int(a + b),
                "-" => Value::Int(a - b),
                "*" => Value::Int(a * b),
                "/" => {
                    if *b == 0 {
                        self.raise_runtime_error(
                            "R_DIVIDE_BY_ZERO",
                            "Division by zero is not allowed".to_string(),
                        );
                        return None;
                    }
                    Value::Int(a / b)
                }
                "%" => {
                    if *b == 0 {
                        self.raise_runtime_error(
                            "R_MODULO_BY_ZERO",
                            "Modulo by zero is not allowed".to_string(),
                        );
                        return None;
                    }
                    Value::Int(a % b)
                }
                _ => Value::Int(*a),
            });
        }

        if op == "%" {
            self.raise_runtime_error(
                "RUNTIME",
                format!(
                    "Operator '%' requires integer operands, got {} and {}",
                    type_name(value_type(&left)),
                    type_name(value_type(&right))
                ),
            );
            return None;
        }

        let l = match as_decimal(&left) {
            Some(v) => v,
            None => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Operator '{}' requires numeric operands, got {} and {}",
                        op,
                        type_name(value_type(&left)),
                        type_name(value_type(&right))
                    ),
                );
                return None;
            }
        };
        let r = match as_decimal(&right) {
            Some(v) => v,
            None => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Operator '{}' requires numeric operands, got {} and {}",
                        op,
                        type_name(value_type(&left)),
                        type_name(value_type(&right))
                    ),
                );
                return None;
            }
        };

        if op == "/" && r == Decimal::ZERO {
            self.raise_runtime_error(
                "R_DIVIDE_BY_ZERO",
                "Division by zero is not allowed".to_string(),
            );
            return None;
        }

        let result = match op {
            "+" => l + r,
            "-" => l - r,
            "*" => l * r,
            "/" => l / r,
            _ => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Unknown numeric operator '{}'", op),
                );
                return None;
            }
        };

        Some(Value::Decimal(result))
    }

    fn eval_equality_binop(
        &mut self,
        op: &str,
        left: Value,
        right: Value,
        equals: bool,
    ) -> Option<Value> {
        if let (Some(l), Some(r)) = (as_decimal(&left), as_decimal(&right)) {
            return Some(Value::Bool(if equals { l == r } else { l != r }));
        }

        let same_type = value_type(&left) == value_type(&right);
        if !same_type {
            self.raise_runtime_error(
                "RUNTIME",
                format!(
                    "Operator '{}' cannot compare {} with {}",
                    op,
                    type_name(value_type(&left)),
                    type_name(value_type(&right))
                ),
            );
            return None;
        }

        let result = if equals { left == right } else { left != right };
        Some(Value::Bool(result))
    }

    fn eval_relational_binop<F>(
        &mut self,
        op: &str,
        left: Value,
        right: Value,
        f: F,
    ) -> Option<Value>
    where
        F: FnOnce(Decimal, Decimal) -> bool,
    {
        let l = match as_decimal(&left) {
            Some(v) => v,
            None => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Operator '{}' requires numeric operands, got {} and {}",
                        op,
                        type_name(value_type(&left)),
                        type_name(value_type(&right))
                    ),
                );
                return None;
            }
        };
        let r = match as_decimal(&right) {
            Some(v) => v,
            None => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Operator '{}' requires numeric operands, got {} and {}",
                        op,
                        type_name(value_type(&left)),
                        type_name(value_type(&right))
                    ),
                );
                return None;
            }
        };

        Some(Value::Bool(f(l, r)))
    }

    fn eval_array_literal(
        &mut self,
        items: &[Expr],
        assignment_target: Option<VarType>,
    ) -> Option<Value> {
        let expected_element = assignment_target.and_then(array_element_type);

        if items.is_empty() {
            if let Some(element_type) = expected_element {
                return Some(Value::Array {
                    items: Vec::new(),
                    element_type,
                });
            }

            self.raise_runtime_error(
                "RUNTIME",
                "Empty array literal [] requires known target array type context".to_string(),
            );
            return None;
        }

        let mut evaluated = Vec::with_capacity(items.len());

        if let Some(element_type) = expected_element {
            for item in items {
                let raw = self.eval_expr(item, Some(element_type))?;
                let coerced = match coerce_value_for_type(raw, element_type) {
                    Some(value) => value,
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "Array literal element is incompatible with {}",
                                type_name(element_type)
                            ),
                        );
                        return None;
                    }
                };
                evaluated.push(coerced);
            }

            return Some(Value::Array {
                items: evaluated,
                element_type,
            });
        }

        let mut inferred_element: Option<VarType> = None;
        for item in items {
            let value = self.eval_expr(item, None)?;
            let value_ty = value_type(&value);

            if is_array_type(value_ty) {
                self.raise_runtime_error(
                    "RUNTIME",
                    "Nested arrays are not supported".to_string(),
                );
                return None;
            }

            match inferred_element {
                None => {
                    inferred_element = Some(value_ty);
                    evaluated.push(value);
                }
                Some(current) if current == value_ty => {
                    evaluated.push(value);
                }
                Some(current) if is_numeric_type(current) && is_numeric_type(value_ty) => {
                    if current == VarType::Integer {
                        for existing in &mut evaluated {
                            if let Value::Int(n) = existing {
                                *existing = Value::Decimal(Decimal::from(*n));
                            }
                        }
                        inferred_element = Some(VarType::Decimal);
                    }

                    match value {
                        Value::Int(n) => evaluated.push(Value::Decimal(Decimal::from(n))),
                        Value::Decimal(n) => evaluated.push(Value::Decimal(n)),
                        _ => unreachable!(),
                    }
                }
                Some(current) => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "Array literal elements must share one scalar type, found {} and {}",
                            type_name(current),
                            type_name(value_ty)
                        ),
                    );
                    return None;
                }
            }
        }

        let element_type = inferred_element.expect("non-empty array literal has inferred element");
        Some(Value::Array {
            items: evaluated,
            element_type,
        })
    }

    fn eval_array_argument(
        &mut self,
        expr: &Expr,
        assignment_target_hint: Option<VarType>,
        function_name: &str,
    ) -> Option<(Vec<Value>, VarType, Option<String>)> {
        match expr {
            Expr::VarRef { name, .. } => match self.resolve_var_value(name).cloned() {
                Some(Value::Array {
                    items,
                    element_type,
                }) => Some((items, element_type, Some(name.clone()))),
                Some(other) => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "{}() expected array argument, got {}",
                            function_name,
                            type_name(value_type(&other))
                        ),
                    );
                    None
                }
                None => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("Read of undeclared variable '${}'", name),
                    );
                    None
                }
            },
            Expr::ListLit { items, .. } => match self.eval_array_literal(items, assignment_target_hint)
            {
                Some(Value::Array {
                    items,
                    element_type,
                }) => Some((items, element_type, None)),
                Some(_) => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("{}() expected array literal argument", function_name),
                    );
                    None
                }
                None => None,
            },
            _ => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "{}() array argument must be a $variable or array literal",
                        function_name
                    ),
                );
                None
            }
        }
    }

    fn eval_scalar_argument(
        &mut self,
        expr: &Expr,
        assignment_target: Option<VarType>,
        function_name: &str,
        argument_name: &str,
    ) -> Option<Value> {
        if !matches!(
            expr,
            Expr::IntLit(_)
                | Expr::DecimalLit(_)
                | Expr::BoolLit(_)
                | Expr::StringLit(_)
                | Expr::VarRef { .. }
        ) {
            self.raise_runtime_error(
                "RUNTIME",
                format!(
                    "{}() {} argument must be a literal or $variable",
                    function_name, argument_name
                ),
            );
            return None;
        }

        let value = self.eval_expr(expr, assignment_target)?;
        if is_array_type(value_type(&value)) {
            self.raise_runtime_error(
                "RUNTIME",
                format!(
                    "{}() {} argument must be scalar",
                    function_name, argument_name
                ),
            );
            return None;
        }

        Some(value)
    }

    fn eval_integer_argument(&mut self, expr: &Expr, function_name: &str) -> Option<i64> {
        let value = self.eval_scalar_argument(expr, Some(VarType::Integer), function_name, "index")?;
        match value {
            Value::Int(n) => Some(n),
            other => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "{}() requires integer argument, got {}",
                        function_name,
                        type_name(value_type(&other))
                    ),
                );
                None
            }
        }
    }

    fn eval_call(
        &mut self,
        name: &str,
        args: &[Expr],
        line: usize,
        column: usize,
        assignment_target: Option<VarType>,
        mode: CallMode,
    ) -> Option<Value> {
        if self.logic_blocks.contains_key(name) {
            return self.eval_logic_call(name, args, line, column, assignment_target, mode);
        }

        match name {
            "abs" => {
                if args.len() != 1 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("abs() expects exactly 1 argument, found {}", args.len()),
                    );
                    return None;
                }

                let value = self.eval_expr(&args[0], assignment_target)?;
                match value {
                    Value::Int(n) => {
                        if let Some(abs) = n.checked_abs() {
                            Some(Value::Int(abs))
                        } else {
                            self.raise_runtime_error(
                                "R_NUMERIC_OVERFLOW",
                                "abs() overflow for integer minimum value".to_string(),
                            );
                            None
                        }
                    }
                    Value::Decimal(n) => Some(Value::Decimal(n.abs())),
                    other => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "abs() requires numeric argument, got {}",
                                type_name(value_type(&other))
                            ),
                        );
                        None
                    }
                }
            }
            "rand" => {
                let target = match assignment_target {
                    Some(VarType::Integer) => VarType::Integer,
                    Some(VarType::Decimal) => VarType::Decimal,
                    Some(other) => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "rand() requires integer or decimal assignment target, got {}",
                                type_name(other)
                            ),
                        );
                        return None;
                    }
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            "rand() requires typed assignment context".to_string(),
                        );
                        return None;
                    }
                };

                match args.len() {
                    0 => match target {
                        VarType::Integer => Some(Value::Int(rand::rng().random::<i64>())),
                        VarType::Decimal => {
                            let sample = rand::rng().random_range(0.0f64..=1.0f64);
                            Decimal::from_f64(sample).map(Value::Decimal).or_else(|| {
                                self.raise_runtime_error(
                                    "RUNTIME",
                                    "Failed to generate decimal random value".to_string(),
                                );
                                None
                            })
                        }
                        _ => None,
                    },
                    2 => match target {
                        VarType::Integer => {
                            let min = self.eval_expr(&args[0], assignment_target)?;
                            let max = self.eval_expr(&args[1], assignment_target)?;
                            let (min, max) = match (min, max) {
                                (Value::Int(min), Value::Int(max)) => (min, max),
                                (a, b) => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Integer rand(min, max) requires integer bounds, got {} and {}",
                                            type_name(value_type(&a)),
                                            type_name(value_type(&b))
                                        ),
                                    );
                                    return None;
                                }
                            };

                            if min > max {
                                self.raise_runtime_error(
                                    "RUNTIME",
                                    "rand(min, max) requires min <= max".to_string(),
                                );
                                return None;
                            }

                            Some(Value::Int(rand::rng().random_range(min..=max)))
                        }
                        VarType::Decimal => {
                            let min = self.eval_expr(&args[0], assignment_target)?;
                            let max = self.eval_expr(&args[1], assignment_target)?;

                            let min_dec = match as_decimal(&min) {
                                Some(value) => value,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Decimal rand(min, max) requires numeric bounds, got {}",
                                            type_name(value_type(&min))
                                        ),
                                    );
                                    return None;
                                }
                            };
                            let max_dec = match as_decimal(&max) {
                                Some(value) => value,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Decimal rand(min, max) requires numeric bounds, got {}",
                                            type_name(value_type(&max))
                                        ),
                                    );
                                    return None;
                                }
                            };

                            if min_dec > max_dec {
                                self.raise_runtime_error(
                                    "RUNTIME",
                                    "rand(min, max) requires min <= max".to_string(),
                                );
                                return None;
                            }

                            let min_f = match min_dec.to_f64() {
                                Some(v) => v,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        "Decimal rand(min, max) bound conversion failed"
                                            .to_string(),
                                    );
                                    return None;
                                }
                            };
                            let max_f = match max_dec.to_f64() {
                                Some(v) => v,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        "Decimal rand(min, max) bound conversion failed"
                                            .to_string(),
                                    );
                                    return None;
                                }
                            };

                            let sample = rand::rng().random_range(min_f..=max_f);
                            Decimal::from_f64(sample).map(Value::Decimal).or_else(|| {
                                self.raise_runtime_error(
                                    "RUNTIME",
                                    "Failed to generate decimal random value".to_string(),
                                );
                                None
                            })
                        }
                        _ => None,
                    },
                    _ => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!("rand() expects 0 or 2 arguments, found {}", args.len()),
                        );
                        None
                    }
                }
            }
            "pick" => {
                if args.len() == 1 {
                    let (items, _element_type, _) = self.eval_array_argument(&args[0], None, "pick")?;
                    if items.is_empty() {
                        self.raise_runtime_error(
                            "R_ARRAY_EMPTY",
                            "pick() requires non-empty array".to_string(),
                        );
                        return None;
                    }

                    let index = rand::rng().random_range(0..items.len());
                    return Some(items[index].clone());
                }

                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("pick() expects 1 or 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                let count_value = self.eval_scalar_argument(
                    &args[0],
                    Some(VarType::Integer),
                    "pick",
                    "count",
                )?;
                let count = match count_value {
                    Value::Int(n) if n >= 0 => n as usize,
                    Value::Int(_) => {
                        self.raise_runtime_error(
                            "R_ARRAY_SAMPLE_COUNT_INVALID",
                            "pick(count, array) requires count >= 0".to_string(),
                        );
                        return None;
                    }
                    other => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "pick(count, array) requires integer count, got {}",
                                type_name(value_type(&other))
                            ),
                        );
                        return None;
                    }
                };

                let hint = assignment_target.filter(|ty| is_array_type(*ty));
                let (items, element_type, _) = self.eval_array_argument(&args[1], hint, "pick")?;

                if count > items.len() {
                    self.raise_runtime_error(
                        "R_ARRAY_SAMPLE_COUNT_INVALID",
                        format!(
                            "pick(count, array) requires count <= array_size (count={}, size={})",
                            count,
                            items.len()
                        ),
                    );
                    return None;
                }

                if count == 0 {
                    return Some(Value::Array {
                        items: Vec::new(),
                        element_type,
                    });
                }

                let mut pool: Vec<usize> = (0..items.len()).collect();
                let mut selected = Vec::with_capacity(count);
                for _ in 0..count {
                    let random_index = rand::rng().random_range(0..pool.len());
                    let source_index = pool.swap_remove(random_index);
                    selected.push(items[source_index].clone());
                }

                Some(Value::Array {
                    items: selected,
                    element_type,
                })
            }
            "array_push" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_push() expects exactly 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_push() returns void and cannot be used as an expression".to_string(),
                    );
                    return None;
                }

                let (mut items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_push")?;
                let value = self.eval_scalar_argument(
                    &args[1],
                    Some(element_type),
                    "array_push",
                    "value",
                )?;
                let coerced = match coerce_value_for_type(value, element_type) {
                    Some(v) => v,
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "array_push() value is incompatible with {}",
                                type_name(element_type)
                            ),
                        );
                        return None;
                    }
                };
                items.push(coerced);
                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items,
                            element_type,
                        },
                    );
                }

                Some(Value::Bool(true))
            }
            "array_pop" => {
                if args.len() != 1 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_pop() expects exactly 1 argument, found {}", args.len()),
                    );
                    return None;
                }

                let (mut items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_pop")?;
                let popped = match items.pop() {
                    Some(value) => value,
                    None => {
                        self.raise_runtime_error(
                            "R_ARRAY_EMPTY",
                            "array_pop() requires non-empty array".to_string(),
                        );
                        return None;
                    }
                };

                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items,
                            element_type,
                        },
                    );
                }

                Some(popped)
            }
            "array_strip" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_strip() expects exactly 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_strip() returns void and cannot be used as an expression".to_string(),
                    );
                    return None;
                }

                let (mut items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_strip")?;
                let raw_value = self.eval_scalar_argument(
                    &args[1],
                    Some(element_type),
                    "array_strip",
                    "value",
                )?;
                let probe = match coerce_value_for_type(raw_value, element_type) {
                    Some(v) => v,
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "array_strip() value is incompatible with {}",
                                type_name(element_type)
                            ),
                        );
                        return None;
                    }
                };
                items.retain(|item| item != &probe);

                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items,
                            element_type,
                        },
                    );
                }

                Some(Value::Bool(true))
            }
            "array_clear" => {
                if args.len() != 1 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_clear() expects exactly 1 argument, found {}", args.len()),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_clear() returns void and cannot be used as an expression".to_string(),
                    );
                    return None;
                }

                let (_items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_clear")?;
                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items: Vec::new(),
                            element_type,
                        },
                    );
                }

                Some(Value::Bool(true))
            }
            "array_contains" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "array_contains() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                let (items, element_type, _) =
                    self.eval_array_argument(&args[0], None, "array_contains")?;
                let raw_value = self.eval_scalar_argument(
                    &args[1],
                    Some(element_type),
                    "array_contains",
                    "value",
                )?;
                let probe = match coerce_value_for_type(raw_value, element_type) {
                    Some(v) => v,
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "array_contains() value is incompatible with {}",
                                type_name(element_type)
                            ),
                        );
                        return None;
                    }
                };

                Some(Value::Bool(items.iter().any(|item| item == &probe)))
            }
            "array_size" => {
                if args.len() != 1 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_size() expects exactly 1 argument, found {}", args.len()),
                    );
                    return None;
                }

                let (items, _element_type, _) =
                    self.eval_array_argument(&args[0], None, "array_size")?;
                Some(Value::Int(items.len() as i64))
            }
            "array_join" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_join() expects exactly 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                let (items, _element_type, _) =
                    self.eval_array_argument(&args[0], None, "array_join")?;
                let separator = self.eval_scalar_argument(
                    &args[1],
                    Some(VarType::String),
                    "array_join",
                    "separator",
                )?;
                let separator = match separator {
                    Value::Str(s) => s,
                    other => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "array_join() separator must be string, got {}",
                                type_name(value_type(&other))
                            ),
                        );
                        return None;
                    }
                };

                let parts: Vec<String> = items.iter().map(Self::value_to_plain_text).collect();
                Some(Value::Str(parts.join(&separator)))
            }
            "array_get" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("array_get() expects exactly 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                let (items, _element_type, _) = self.eval_array_argument(&args[0], None, "array_get")?;
                let index = self.eval_integer_argument(&args[1], "array_get")?;
                if index < 0 || (index as usize) >= items.len() {
                    self.raise_runtime_error(
                        "R_ARRAY_INDEX_OUT_OF_RANGE",
                        format!(
                            "array_get() index {} out of range for size {}",
                            index,
                            items.len()
                        ),
                    );
                    return None;
                }

                Some(items[index as usize].clone())
            }
            "array_insert" => {
                if args.len() != 3 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "array_insert() expects exactly 3 arguments, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_insert() returns void and cannot be used as an expression"
                            .to_string(),
                    );
                    return None;
                }

                let (mut items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_insert")?;
                let index = self.eval_integer_argument(&args[1], "array_insert")?;
                if index < 0 || (index as usize) > items.len() {
                    self.raise_runtime_error(
                        "R_ARRAY_INDEX_OUT_OF_RANGE",
                        format!(
                            "array_insert() index {} out of range for size {}",
                            index,
                            items.len()
                        ),
                    );
                    return None;
                }

                let value = self.eval_scalar_argument(
                    &args[2],
                    Some(element_type),
                    "array_insert",
                    "value",
                )?;
                let coerced = match coerce_value_for_type(value, element_type) {
                    Some(v) => v,
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "array_insert() value is incompatible with {}",
                                type_name(element_type)
                            ),
                        );
                        return None;
                    }
                };

                items.insert(index as usize, coerced);
                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items,
                            element_type,
                        },
                    );
                }

                Some(Value::Bool(true))
            }
            "array_remove" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "array_remove() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                let (mut items, element_type, target_name) =
                    self.eval_array_argument(&args[0], None, "array_remove")?;
                let index = self.eval_integer_argument(&args[1], "array_remove")?;
                if index < 0 || (index as usize) >= items.len() {
                    self.raise_runtime_error(
                        "R_ARRAY_INDEX_OUT_OF_RANGE",
                        format!(
                            "array_remove() index {} out of range for size {}",
                            index,
                            items.len()
                        ),
                    );
                    return None;
                }

                let removed = items.remove(index as usize);
                if let Some(name) = target_name {
                    self.write_variable(
                        &name,
                        Value::Array {
                            items,
                            element_type,
                        },
                    );
                }

                Some(removed)
            }
            _ => {
                self.raise_runtime_error("RUNTIME", format!("Unknown function '{}'", name));
                None
            }
        }
    }

    fn eval_logic_call(
        &mut self,
        name: &str,
        args: &[Expr],
        _line: usize,
        _column: usize,
        _assignment_target: Option<VarType>,
        mode: CallMode,
    ) -> Option<Value> {
        let logic = match self.logic_blocks.get(name).cloned() {
            Some(block) => block,
            None => {
                self.raise_runtime_error("RUNTIME", format!("Unknown function '{}'", name));
                return None;
            }
        };

        if args.len() != logic.params.len() {
            self.raise_runtime_error(
                "RUNTIME",
                format!(
                    "{}() expects exactly {} arguments, found {}",
                    name,
                    logic.params.len(),
                    args.len()
                ),
            );
            return None;
        }

        let baseline_local_types = self.local_var_types.clone();
        let baseline_local_values = self.local_variables.clone();
        let baseline_local_names: Vec<String> = baseline_local_types.keys().cloned().collect();

        for (arg_expr, param) in args.iter().zip(logic.params.iter()) {
            if self.local_var_types.contains_key(&param.name) {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Logic parameter '${}' conflicts with existing local variable",
                        param.name
                    ),
                );
                self.restore_logic_scope(
                    baseline_local_types.clone(),
                    baseline_local_values.clone(),
                    baseline_local_names.clone(),
                );
                return None;
            }

            let raw = match self.eval_expr(arg_expr, Some(param.var_type)) {
                Some(value) => value,
                None => {
                    self.restore_logic_scope(
                        baseline_local_types.clone(),
                        baseline_local_values.clone(),
                        baseline_local_names.clone(),
                    );
                    return None;
                }
            };

            let coerced = match coerce_value_for_type(raw.clone(), param.var_type) {
                Some(value) => value,
                None => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "Argument for '${}' is incompatible with {}",
                            param.name,
                            type_name(param.var_type)
                        ),
                    );
                    self.restore_logic_scope(
                        baseline_local_types.clone(),
                        baseline_local_values.clone(),
                        baseline_local_names.clone(),
                    );
                    return None;
                }
            };

            self.local_var_types.insert(param.name.clone(), param.var_type);
            self.local_variables.insert(param.name.clone(), coerced);
        }

        let flow = self.execute_prep_block(&logic.body, false, true, logic.return_type);
        let result = match (logic.return_type, flow) {
            (_, PrepFlow::Error) => None,
            (_, PrepFlow::BreakLoop | PrepFlow::ContinueLoop) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Logic function '{}' terminated with invalid loop control", name),
                );
                None
            }
            (None, PrepFlow::Next) => {
                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("{}() returns void and cannot be used as an expression", name),
                    );
                    None
                } else {
                    Some(Value::Bool(true))
                }
            }
            (None, PrepFlow::Return(Some(_))) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Void logic function '{}' cannot return a value", name),
                );
                None
            }
            (None, PrepFlow::Return(None)) => Some(Value::Bool(true)),
            (Some(return_type), PrepFlow::Next) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Logic function '{}' must return a value of type {}",
                        name,
                        type_name(return_type)
                    ),
                );
                None
            }
            (Some(_), PrepFlow::Return(None)) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Logic function '{}' returned without a value", name),
                );
                None
            }
            (Some(return_type), PrepFlow::Return(Some(value))) => {
                match coerce_value_for_type(value, return_type) {
                    Some(v) => Some(v),
                    None => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!(
                                "Logic function '{}' returned incompatible value for {}",
                                name,
                                type_name(return_type)
                            ),
                        );
                        None
                    }
                }
            }
        };

        self.restore_logic_scope(
            baseline_local_types,
            baseline_local_values,
            baseline_local_names,
        );

        result
    }

    fn restore_logic_scope(
        &mut self,
        baseline_types: HashMap<String, VarType>,
        mut baseline_values: HashMap<String, Value>,
        baseline_local_names: Vec<String>,
    ) {
        for name in baseline_local_names {
            if let Some(value) = self.local_variables.get(&name).cloned() {
                baseline_values.insert(name, value);
            }
        }

        self.local_var_types = baseline_types;
        self.local_variables = baseline_values;
    }

    fn resolve_string_or_error(&mut self, template: &str, context: &str) -> Option<String> {
        match self.resolve_string(template) {
            Ok(value) => Some(value),
            Err(message) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Interpolation failed in {}: {}", context, message),
                );
                None
            }
        }
    }

    fn resolve_string(&self, template: &str) -> Result<String, String> {
        render_interpolated(template, |name| self.resolve_var_value(name).map(Self::value_to_plain_text))
        .map_err(|e| e.message)
    }

    fn resolve_var_type(&self, name: &str) -> Option<VarType> {
        self.local_var_types
            .get(name)
            .copied()
            .or_else(|| self.var_types.get(name).copied())
    }

    fn eval_repeat_count(&mut self, count: &RepeatCount) -> Option<usize> {
        let raw_count = match count {
            RepeatCount::IntLiteral { value, .. } => *value,
            RepeatCount::Variable { name, .. } => match self.resolve_var_value(name).cloned() {
                Some(Value::Int(value)) => value,
                Some(other) => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "repeat(count) requires integer count variable, got {}",
                            type_name(value_type(&other))
                        ),
                    );
                    return None;
                }
                None => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("Read of undeclared variable '${}' in repeat count", name),
                    );
                    return None;
                }
            },
        };

        if raw_count <= 0 {
            self.raise_runtime_error(
                "R_REPEAT_COUNT_INVALID",
                format!("repeat(count) requires count > 0, got {}", raw_count),
            );
            return None;
        }

        Some(raw_count as usize)
    }

    fn resolve_snapshot_array(&mut self, name: &str) -> Option<(Vec<Value>, VarType)> {
        match self.resolve_var_value(name).cloned() {
            Some(Value::Array {
                items,
                element_type,
            }) => Some((items, element_type)),
            Some(other) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "for (...) snapshot source '${}' must be an array, got {}",
                        name,
                        type_name(value_type(&other))
                    ),
                );
                None
            }
            None => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!("Read of undeclared variable '${}' in for snapshot source", name),
                );
                None
            }
        }
    }

    fn resolve_var_value(&self, name: &str) -> Option<&Value> {
        self.local_variables
            .get(name)
            .or_else(|| self.variables.get(name))
    }

    fn restore_loop_binding(
        &mut self,
        name: &str,
        previous_type: Option<VarType>,
        previous_value: Option<Value>,
    ) {
        match previous_type {
            Some(var_type) => {
                self.local_var_types.insert(name.to_string(), var_type);
            }
            None => {
                self.local_var_types.remove(name);
            }
        }

        match previous_value {
            Some(value) => {
                self.local_variables.insert(name.to_string(), value);
            }
            None => {
                self.local_variables.remove(name);
            }
        }
    }

    fn write_variable(&mut self, name: &str, value: Value) {
        if self.local_var_types.contains_key(name) {
            self.local_variables.insert(name.to_string(), value);
        } else {
            self.variables.insert(name.to_string(), value);
        }
    }

    fn value_to_plain_text(value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Decimal(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
            Value::Array {
                items,
                element_type: _,
            } => {
                let rendered: Vec<String> = items.iter().map(Self::value_to_plain_text).collect();
                format!("[{}]", rendered.join(", "))
            }
        }
    }

    fn raise_runtime_error(&mut self, code: &str, message: String) {
        self.pending.clear();
        self.pending
            .push_back(InternalEvent::Narration(format!("[{}] {}", code, message)));
        self.pending.push_back(InternalEvent::End);
        self.finished = true;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn eval_init_expr(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
) -> Option<Value> {
    match expr {
        Expr::IntLit(n) => Some(Value::Int(*n)),
        Expr::DecimalLit(n) => Some(Value::Decimal(*n)),
        Expr::BoolLit(b) => Some(Value::Bool(*b)),
        Expr::StringLit(s) => {
            render_interpolated(s, |name| vars.get(name).map(value_to_plain_text))
                .ok()
                .map(|v| Value::Str(v))
        }
        Expr::VarRef { name, .. } => vars.get(name).cloned(),
        Expr::Call { name, args, .. } => eval_init_call(name, args, vars, assignment_target),
        Expr::ListLit { items, .. } => eval_init_array_literal(items, vars, assignment_target),
        Expr::BinOp { left, op, right } => {
            let l = eval_init_expr(left, vars, assignment_target)?;
            let r = eval_init_expr(right, vars, assignment_target)?;
            match op {
                BinOperator::Add => eval_init_numeric_binop("+", l, r),
                BinOperator::Sub => eval_init_numeric_binop("-", l, r),
                BinOperator::Mul => eval_init_numeric_binop("*", l, r),
                BinOperator::Div => eval_init_numeric_binop("/", l, r),
                BinOperator::Mod => eval_init_numeric_binop("%", l, r),
                BinOperator::EqEq => eval_init_equality_binop(l, r, true),
                BinOperator::NotEq => eval_init_equality_binop(l, r, false),
                BinOperator::Lt => eval_init_rel_binop(l, r, |a, b| a < b),
                BinOperator::LtEq => eval_init_rel_binop(l, r, |a, b| a <= b),
                BinOperator::Gt => eval_init_rel_binop(l, r, |a, b| a > b),
                BinOperator::GtEq => eval_init_rel_binop(l, r, |a, b| a >= b),
            }
        }
    }
}

fn eval_init_numeric_binop(op: &str, left: Value, right: Value) -> Option<Value> {
    if let (Value::Int(a), Value::Int(b)) = (&left, &right) {
        return Some(match op {
            "+" => Value::Int(a + b),
            "-" => Value::Int(a - b),
            "*" => Value::Int(a * b),
            "/" => {
                if *b == 0 {
                    return None;
                }
                Value::Int(a / b)
            }
            "%" => {
                if *b == 0 {
                    return None;
                }
                Value::Int(a % b)
            }
            _ => Value::Int(*a),
        });
    }

    if op == "%" {
        return None;
    }

    let l = as_decimal(&left)?;
    let r = as_decimal(&right)?;

    if op == "/" && r == Decimal::ZERO {
        return None;
    }

    let result = match op {
        "+" => l + r,
        "-" => l - r,
        "*" => l * r,
        "/" => l / r,
        _ => return None,
    };

    Some(Value::Decimal(result))
}

fn eval_init_array_literal(
    items: &[Expr],
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
) -> Option<Value> {
    let expected_element = assignment_target.and_then(array_element_type);

    if items.is_empty() {
        if let Some(element_type) = expected_element {
            return Some(Value::Array {
                items: Vec::new(),
                element_type,
            });
        }
        return None;
    }

    let mut evaluated = Vec::with_capacity(items.len());

    if let Some(element_type) = expected_element {
        for item in items {
            let value = eval_init_expr(item, vars, Some(element_type))?;
            let coerced = coerce_value_for_type(value, element_type)?;
            evaluated.push(coerced);
        }
        return Some(Value::Array {
            items: evaluated,
            element_type,
        });
    }

    let mut inferred_element: Option<VarType> = None;
    for item in items {
        let value = eval_init_expr(item, vars, None)?;
        let value_ty = value_type(&value);
        if is_array_type(value_ty) {
            return None;
        }

        match inferred_element {
            None => {
                inferred_element = Some(value_ty);
                evaluated.push(value);
            }
            Some(current) if current == value_ty => {
                evaluated.push(value);
            }
            Some(current) if is_numeric_type(current) && is_numeric_type(value_ty) => {
                if current == VarType::Integer {
                    for existing in &mut evaluated {
                        if let Value::Int(n) = existing {
                            *existing = Value::Decimal(Decimal::from(*n));
                        }
                    }
                    inferred_element = Some(VarType::Decimal);
                }

                match value {
                    Value::Int(n) => evaluated.push(Value::Decimal(Decimal::from(n))),
                    Value::Decimal(n) => evaluated.push(Value::Decimal(n)),
                    _ => return None,
                }
            }
            Some(_) => return None,
        }
    }

    Some(Value::Array {
        items: evaluated,
        element_type: inferred_element?,
    })
}

fn eval_init_array_argument(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target_hint: Option<VarType>,
) -> Option<(Vec<Value>, VarType)> {
    match expr {
        Expr::VarRef { name, .. } => match vars.get(name)? {
            Value::Array {
                items,
                element_type,
            } => Some((items.clone(), *element_type)),
            _ => None,
        },
        Expr::ListLit { items, .. } => {
            let value = eval_init_array_literal(items, vars, assignment_target_hint)?;
            match value {
                Value::Array {
                    items,
                    element_type,
                } => Some((items, element_type)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn eval_init_scalar_argument(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
) -> Option<Value> {
    if !matches!(
        expr,
        Expr::IntLit(_)
            | Expr::DecimalLit(_)
            | Expr::BoolLit(_)
            | Expr::StringLit(_)
            | Expr::VarRef { .. }
    ) {
        return None;
    }

    let value = eval_init_expr(expr, vars, assignment_target)?;
    if is_array_type(value_type(&value)) {
        return None;
    }

    Some(value)
}

fn eval_init_call(
    name: &str,
    args: &[Expr],
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
) -> Option<Value> {
    match name {
        "abs" => {
            if args.len() != 1 {
                return None;
            }

            match eval_init_expr(&args[0], vars, assignment_target)? {
                Value::Int(n) => n.checked_abs().map(Value::Int),
                Value::Decimal(n) => Some(Value::Decimal(n.abs())),
                _ => None,
            }
        }
        "rand" => {
            let target = match assignment_target {
                Some(VarType::Integer) => VarType::Integer,
                Some(VarType::Decimal) => VarType::Decimal,
                _ => return None,
            };

            match args.len() {
                0 => match target {
                    VarType::Integer => Some(Value::Int(rand::rng().random::<i64>())),
                    VarType::Decimal => Decimal::from_f64(rand::rng().random_range(0.0..=1.0))
                        .map(Value::Decimal),
                    _ => None,
                },
                2 => match target {
                    VarType::Integer => {
                        let min = eval_init_expr(&args[0], vars, assignment_target)?;
                        let max = eval_init_expr(&args[1], vars, assignment_target)?;
                        let (min, max) = match (min, max) {
                            (Value::Int(min), Value::Int(max)) => (min, max),
                            _ => return None,
                        };
                        if min > max {
                            return None;
                        }
                        Some(Value::Int(rand::rng().random_range(min..=max)))
                    }
                    VarType::Decimal => {
                        let min = as_decimal(&eval_init_expr(&args[0], vars, assignment_target)?)?;
                        let max = as_decimal(&eval_init_expr(&args[1], vars, assignment_target)?)?;

                        if min > max {
                            return None;
                        }

                        let min_f = min.to_f64()?;
                        let max_f = max.to_f64()?;
                        Decimal::from_f64(rand::rng().random_range(min_f..=max_f))
                            .map(Value::Decimal)
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        "pick" => {
            if args.len() == 1 {
                let (items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
                if items.is_empty() {
                    return None;
                }

                let index = rand::rng().random_range(0..items.len());
                return Some(items[index].clone());
            }

            if args.len() != 2 {
                return None;
            }

            let count = match eval_init_scalar_argument(
                &args[0],
                vars,
                Some(VarType::Integer),
            )? {
                Value::Int(n) if n >= 0 => n as usize,
                _ => return None,
            };

            let hint = assignment_target.filter(|ty| is_array_type(*ty));
            let (items, element_type) = eval_init_array_argument(&args[1], vars, hint)?;
            if count > items.len() {
                return None;
            }

            if count == 0 {
                return Some(Value::Array {
                    items: Vec::new(),
                    element_type,
                });
            }

            let mut pool: Vec<usize> = (0..items.len()).collect();
            let mut selected = Vec::with_capacity(count);
            for _ in 0..count {
                let random_index = rand::rng().random_range(0..pool.len());
                let source_index = pool.swap_remove(random_index);
                selected.push(items[source_index].clone());
            }

            Some(Value::Array {
                items: selected,
                element_type,
            })
        }
        "array_push" | "array_strip" | "array_clear" | "array_insert" => None,
        "array_pop" => {
            if args.len() != 1 {
                return None;
            }

            let (mut items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
            items.pop()
        }
        "array_contains" => {
            if args.len() != 2 {
                return None;
            }

            let (items, element_type) = eval_init_array_argument(&args[0], vars, None)?;
            let probe = eval_init_scalar_argument(&args[1], vars, Some(element_type))?;
            let probe = coerce_value_for_type(probe, element_type)?;
            Some(Value::Bool(items.iter().any(|item| item == &probe)))
        }
        "array_size" => {
            if args.len() != 1 {
                return None;
            }

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
            Some(Value::Int(items.len() as i64))
        }
        "array_join" => {
            if args.len() != 2 {
                return None;
            }

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
            let separator = match eval_init_scalar_argument(
                &args[1],
                vars,
                Some(VarType::String),
            )? {
                Value::Str(s) => s,
                _ => return None,
            };

            let parts: Vec<String> = items.iter().map(value_to_plain_text).collect();
            Some(Value::Str(parts.join(&separator)))
        }
        "array_get" => {
            if args.len() != 2 {
                return None;
            }

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
            let index = match eval_init_scalar_argument(
                &args[1],
                vars,
                Some(VarType::Integer),
            )? {
                Value::Int(n) if n >= 0 => n as usize,
                _ => return None,
            };

            items.get(index).cloned()
        }
        "array_remove" => {
            if args.len() != 2 {
                return None;
            }

            let (mut items, _element_type) = eval_init_array_argument(&args[0], vars, None)?;
            let index = match eval_init_scalar_argument(
                &args[1],
                vars,
                Some(VarType::Integer),
            )? {
                Value::Int(n) if n >= 0 => n as usize,
                _ => return None,
            };

            if index >= items.len() {
                return None;
            }
            Some(items.remove(index))
        }
        _ => None,
    }
}

fn eval_init_equality_binop(left: Value, right: Value, equals: bool) -> Option<Value> {
    if let (Some(l), Some(r)) = (as_decimal(&left), as_decimal(&right)) {
        return Some(Value::Bool(if equals { l == r } else { l != r }));
    }

    if value_type(&left) != value_type(&right) {
        return None;
    }

    Some(Value::Bool(if equals {
        left == right
    } else {
        left != right
    }))
}

fn eval_init_rel_binop<F>(left: Value, right: Value, f: F) -> Option<Value>
where
    F: FnOnce(Decimal, Decimal) -> bool,
{
    let l = as_decimal(&left)?;
    let r = as_decimal(&right)?;
    Some(Value::Bool(f(l, r)))
}

fn coerce_value_for_type(value: Value, target_type: VarType) -> Option<Value> {
    match (target_type, value) {
        (VarType::Integer, Value::Int(n)) => Some(Value::Int(n)),
        (VarType::String, Value::Str(s)) => Some(Value::Str(s)),
        (VarType::Boolean, Value::Bool(b)) => Some(Value::Bool(b)),
        (VarType::Decimal, Value::Decimal(n)) => Some(Value::Decimal(n)),
        (VarType::Decimal, Value::Int(n)) => Some(Value::Decimal(Decimal::from(n))),
        (
            VarType::ArrayInteger,
            Value::Array {
                items,
                element_type: VarType::Integer,
            },
        ) => Some(Value::Array {
            items,
            element_type: VarType::Integer,
        }),
        (
            VarType::ArrayString,
            Value::Array {
                items,
                element_type: VarType::String,
            },
        ) => Some(Value::Array {
            items,
            element_type: VarType::String,
        }),
        (
            VarType::ArrayBoolean,
            Value::Array {
                items,
                element_type: VarType::Boolean,
            },
        ) => Some(Value::Array {
            items,
            element_type: VarType::Boolean,
        }),
        (
            VarType::ArrayDecimal,
            Value::Array {
                items,
                element_type: VarType::Decimal,
            },
        ) => Some(Value::Array {
            items,
            element_type: VarType::Decimal,
        }),
        (
            VarType::ArrayDecimal,
            Value::Array {
                items,
                element_type: VarType::Integer,
            },
        ) => {
            let mut converted = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Int(n) => converted.push(Value::Decimal(Decimal::from(n))),
                    Value::Decimal(n) => converted.push(Value::Decimal(n)),
                    _ => return None,
                }
            }
            Some(Value::Array {
                items: converted,
                element_type: VarType::Decimal,
            })
        }
        _ => None,
    }
}

fn default_value_for_type(var_type: VarType) -> Value {
    match var_type {
        VarType::Integer => Value::Int(0),
        VarType::String => Value::Str(String::new()),
        VarType::Boolean => Value::Bool(false),
        VarType::Decimal => Value::Decimal(Decimal::ZERO),
        VarType::ArrayInteger => Value::Array {
            items: Vec::new(),
            element_type: VarType::Integer,
        },
        VarType::ArrayString => Value::Array {
            items: Vec::new(),
            element_type: VarType::String,
        },
        VarType::ArrayBoolean => Value::Array {
            items: Vec::new(),
            element_type: VarType::Boolean,
        },
        VarType::ArrayDecimal => Value::Array {
            items: Vec::new(),
            element_type: VarType::Decimal,
        },
    }
}

fn value_type(value: &Value) -> VarType {
    match value {
        Value::Int(_) => VarType::Integer,
        Value::Decimal(_) => VarType::Decimal,
        Value::Bool(_) => VarType::Boolean,
        Value::Str(_) => VarType::String,
        Value::Array {
            items: _,
            element_type,
        } => array_type_for_element(*element_type).expect("array element type must be scalar"),
    }
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

fn array_type_for_element(element_type: VarType) -> Option<VarType> {
    match element_type {
        VarType::Integer => Some(VarType::ArrayInteger),
        VarType::String => Some(VarType::ArrayString),
        VarType::Boolean => Some(VarType::ArrayBoolean),
        VarType::Decimal => Some(VarType::ArrayDecimal),
        _ => None,
    }
}

fn array_element_type(array_type: VarType) -> Option<VarType> {
    match array_type {
        VarType::ArrayInteger => Some(VarType::Integer),
        VarType::ArrayString => Some(VarType::String),
        VarType::ArrayBoolean => Some(VarType::Boolean),
        VarType::ArrayDecimal => Some(VarType::Decimal),
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

fn is_numeric_type(var_type: VarType) -> bool {
    matches!(var_type, VarType::Integer | VarType::Decimal)
}

fn as_decimal(value: &Value) -> Option<Decimal> {
    match value {
        Value::Int(n) => Some(Decimal::from(*n)),
        Value::Decimal(n) => Some(*n),
        _ => None,
    }
}

fn value_to_plain_text(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Decimal(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => s.clone(),
        Value::Array {
            items,
            element_type: _,
        } => {
            let rendered: Vec<String> = items.iter().map(value_to_plain_text).collect();
            format!("[{}]", rendered.join(", "))
        }
    }
}

fn _resolve_string_best_effort(template: &str, vars: &HashMap<String, Value>) -> String {
    match render_interpolated(template, |name| {
        vars.get(name)
            .map(value_to_plain_text)
            .or_else(|| Some(format!("${{{}}}", name)))
    }) {
        Ok(value) => value,
        Err(_) => template
            .chars()
            .map(|ch| if ch == ESCAPED_DOLLAR_MARKER { '$' } else { ch })
            .collect(),
    }
}
