use std::collections::{HashMap, VecDeque};

use rand::Rng;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use storycript_parser::ast::*;
use storycript_parser::interpolation::{ESCAPED_DOLLAR_MARKER, render_interpolated};

// ---------------------------------------------------------------------------
// Runtime value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Decimal(Decimal),
    Bool(bool),
    Str(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Decimal(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "\"{}\"", s),
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

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct Engine {
    scenes: HashMap<String, Scene>,
    actors: HashMap<String, ActorInfo>,
    pub variables: HashMap<String, Value>,
    var_types: HashMap<String, VarType>,
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

        let start = script.init.start.target.clone();

        let mut engine = Engine {
            scenes,
            actors,
            variables,
            var_types,
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
        for stmt in stmts {
            match stmt {
                PrepStatement::BgDirective { path, .. } => {
                    let resolved = match self.resolve_string_or_error(path, "@bg path") {
                        Some(value) => value,
                        None => return false,
                    };
                    self.bg = Some(resolved);
                }
                PrepStatement::BgmDirective { value, .. } => {
                    self.bgm = match value {
                        BgmValue::Path(p) => {
                            let resolved = match self.resolve_string_or_error(p, "@bgm path") {
                                Some(value) => value,
                                None => return false,
                            };
                            Some(resolved)
                        }
                        BgmValue::Stop => None,
                    };
                }
                PrepStatement::SfxDirective { .. } => {
                    // SFX: can't play audio in TUI, skip
                }
                PrepStatement::VarAssign(assign) => {
                    let declared_type = match self.var_types.get(&assign.name).copied() {
                        Some(ty) => ty,
                        None => {
                            self.raise_runtime_error(
                                "RUNTIME",
                                format!("Unknown variable '${}' in PREP assignment", assign.name),
                            );
                            return false;
                        }
                    };

                    let rhs = match self.eval_expr(&assign.value, Some(declared_type)) {
                        Some(value) => value,
                        None => return false,
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
                                    return false;
                                }
                            };
                            self.variables.insert(assign.name.clone(), coerced);
                        }
                        AssignOp::AddEq | AssignOp::SubEq => {
                            let current = match self.variables.get(&assign.name).cloned() {
                                Some(value) => value,
                                None => {
                                    self.raise_runtime_error(
                                        "RUNTIME",
                                        format!(
                                            "Variable '${}' missing in runtime state",
                                            assign.name
                                        ),
                                    );
                                    return false;
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
                                            return false;
                                        }
                                    };

                                    let updated = match assign.op {
                                        AssignOp::AddEq => Value::Int(a + b),
                                        AssignOp::SubEq => Value::Int(a - b),
                                        AssignOp::Set => Value::Int(a),
                                    };
                                    self.variables.insert(assign.name.clone(), updated);
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
                                            return false;
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
                                            return false;
                                        }
                                    };

                                    let updated = match assign.op {
                                        AssignOp::AddEq => Value::Decimal(current_num + rhs_num),
                                        AssignOp::SubEq => Value::Decimal(current_num - rhs_num),
                                        AssignOp::Set => Value::Decimal(current_num),
                                    };
                                    self.variables.insert(assign.name.clone(), updated);
                                }
                                VarType::String | VarType::Boolean => {
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
                                    return false;
                                }
                            }
                        }
                    }
                }
                PrepStatement::IfElse(if_else) => {
                    let condition = match self.eval_bool(&if_else.condition) {
                        Some(value) => value,
                        None => return false,
                    };

                    if condition {
                        if !self.execute_prep(&if_else.then_branch) {
                            return false;
                        }
                    } else if let Some(else_branch) = &if_else.else_branch {
                        if !self.execute_prep(else_branch) {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    // -----------------------------------------------------------------------
    // #STORY flattening
    // -----------------------------------------------------------------------

    fn flatten_story(&mut self, stmts: &[StoryStatement]) {
        for stmt in stmts {
            match stmt {
                StoryStatement::Narration { text, .. } => {
                    let resolved = match self.resolve_string_or_error(text, "narration") {
                        Some(value) => value,
                        None => return,
                    };
                    self.pending.push_back(InternalEvent::Narration(resolved));
                }
                StoryStatement::VarOutput { name, .. } => {
                    if let Some(value) = self.variables.get(name) {
                        self.pending
                            .push_back(InternalEvent::Narration(Self::value_to_plain_text(value)));
                    } else {
                        self.raise_runtime_error(
                            "RUNTIME",
                            format!("Variable '${}' was missing during STORY output", name),
                        );
                        return;
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
                        None => return,
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
                        None => return,
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
                        None => return,
                    };

                    if condition {
                        self.flatten_story(&if_else.then_branch);
                        if self.finished {
                            return;
                        }
                    } else if let Some(else_branch) = &if_else.else_branch {
                        self.flatten_story(else_branch);
                        if self.finished {
                            return;
                        }
                    }
                }
                StoryStatement::Choice(choice_block) => {
                    let mut options: Vec<ChoiceDisplay> = Vec::new();
                    for opt in &choice_block.options {
                        let available = if let Some(condition) = &opt.condition {
                            match self.eval_bool(condition) {
                                Some(value) => value,
                                None => return,
                            }
                        } else {
                            true
                        };

                        if !available {
                            continue;
                        }

                        let text = match self.resolve_string_or_error(&opt.text, "choice label") {
                            Some(value) => value,
                            None => return,
                        };

                        options.push(ChoiceDisplay {
                            text,
                            target: opt.target.clone(),
                        });
                    }

                    if options.is_empty() {
                        self.raise_runtime_error(
                            "R_CHOICE_EXHAUSTED",
                            "All @choice options were filtered out at runtime".to_string(),
                        );
                        return;
                    }

                    self.pending.push_back(InternalEvent::Choices(options));
                }
                StoryStatement::Jump { target, .. } => {
                    self.pending.push_back(InternalEvent::Jump(target.clone()));
                }
                StoryStatement::End { .. } => {
                    self.pending.push_back(InternalEvent::End);
                }
                StoryStatement::SfxDirective { .. } => {
                    // SFX in STORY: skip in TUI mode
                }
            }
        }
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
                if let Some(value) = self.variables.get(name).cloned() {
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
            } => self.eval_call(name, args, *line, *column, assignment_target),
            Expr::ListLit { .. } => {
                self.raise_runtime_error(
                    "RUNTIME",
                    "List literals are only valid as pick([ ... ]) arguments".to_string(),
                );
                None
            }
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
                            "RUNTIME",
                            "Division by zero is not allowed".to_string(),
                        );
                        return None;
                    }
                    Value::Int(a / b)
                }
                "%" => {
                    if *b == 0 {
                        self.raise_runtime_error(
                            "RUNTIME",
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
            self.raise_runtime_error("RUNTIME", "Division by zero is not allowed".to_string());
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

    fn eval_call(
        &mut self,
        name: &str,
        args: &[Expr],
        _line: usize,
        _column: usize,
        assignment_target: Option<VarType>,
    ) -> Option<Value> {
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
                                "RUNTIME",
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
                if args.len() != 1 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("pick() expects exactly 1 argument, found {}", args.len()),
                    );
                    return None;
                }

                let items = match &args[0] {
                    Expr::ListLit { items, .. } => items,
                    _ => {
                        self.raise_runtime_error(
                            "RUNTIME",
                            "pick() expects list literal argument: pick([a, b, ...])"
                                .to_string(),
                        );
                        return None;
                    }
                };

                if items.is_empty() {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "pick() requires a non-empty candidate list".to_string(),
                    );
                    return None;
                }

                let mut evaluated: Vec<Value> = Vec::with_capacity(items.len());
                for item in items {
                    let mut value = self.eval_expr(item, assignment_target)?;

                    if let Some(target) = assignment_target {
                        value = match coerce_value_for_type(value, target) {
                            Some(v) => v,
                            None => {
                                self.raise_runtime_error(
                                    "RUNTIME",
                                    format!(
                                        "pick() candidate is incompatible with assignment target {}",
                                        type_name(target)
                                    ),
                                );
                                return None;
                            }
                        };
                    }

                    evaluated.push(value);
                }

                if assignment_target.is_none() {
                    let first_type = value_type(&evaluated[0]);
                    if evaluated.iter().skip(1).any(|v| value_type(v) != first_type) {
                        self.raise_runtime_error(
                            "RUNTIME",
                            "pick() candidates must share one type outside assignment context"
                                .to_string(),
                        );
                        return None;
                    }
                }

                let index = rand::rng().random_range(0..evaluated.len());
                Some(evaluated[index].clone())
            }
            _ => {
                self.raise_runtime_error("RUNTIME", format!("Unknown function '{}'", name));
                None
            }
        }
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
        render_interpolated(template, |name| {
            self.variables.get(name).map(Self::value_to_plain_text)
        })
        .map_err(|e| e.message)
    }

    fn value_to_plain_text(value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Decimal(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
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
        Expr::ListLit { .. } => None,
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
            if args.len() != 1 {
                return None;
            }

            let items = match &args[0] {
                Expr::ListLit { items, .. } => items,
                _ => return None,
            };

            if items.is_empty() {
                return None;
            }

            let mut values = Vec::with_capacity(items.len());
            for item in items {
                let mut value = eval_init_expr(item, vars, assignment_target)?;

                if let Some(target) = assignment_target {
                    value = coerce_value_for_type(value, target)?;
                }

                values.push(value);
            }

            if assignment_target.is_none() {
                let first = value_type(&values[0]);
                if values.iter().skip(1).any(|v| value_type(v) != first) {
                    return None;
                }
            }

            let index = rand::rng().random_range(0..values.len());
            Some(values[index].clone())
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
        _ => None,
    }
}

fn default_value_for_type(var_type: VarType) -> Value {
    match var_type {
        VarType::Integer => Value::Int(0),
        VarType::String => Value::Str(String::new()),
        VarType::Boolean => Value::Bool(false),
        VarType::Decimal => Value::Decimal(Decimal::ZERO),
    }
}

fn value_type(value: &Value) -> VarType {
    match value {
        Value::Int(_) => VarType::Integer,
        Value::Decimal(_) => VarType::Decimal,
        Value::Bool(_) => VarType::Boolean,
        Value::Str(_) => VarType::String,
    }
}

fn type_name(var_type: VarType) -> &'static str {
    match var_type {
        VarType::Integer => "integer",
        VarType::String => "string",
        VarType::Boolean => "boolean",
        VarType::Decimal => "decimal",
    }
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
