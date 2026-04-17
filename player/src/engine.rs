use std::collections::{HashMap, VecDeque};
use storycript_parser::ast::*;
use storycript_parser::interpolation::{render_interpolated, ESCAPED_DOLLAR_MARKER};

// ---------------------------------------------------------------------------
// Runtime value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
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
    pub current_scene: String,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pending: VecDeque<InternalEvent>,
    pub finished: bool,
}

impl Engine {
    pub fn new(script: &Script) -> Self {
        // Initialize variables from INIT block
        let mut variables = HashMap::new();
        for var in &script.init.variables {
            variables.insert(var.name.clone(), eval_init_expr(&var.value));
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
        self.pending.push_back(InternalEvent::Narration(
            format!("─── Scene: {} ───", label),
        ));

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
                    let val = self.eval_expr(&assign.value);
                    match assign.op {
                        AssignOp::Set => {
                            self.variables.insert(assign.name.clone(), val);
                        }
                        AssignOp::AddEq => {
                            let current = self
                                .variables
                                .get(&assign.name)
                                .cloned()
                                .unwrap_or(Value::Int(0));
                            if let (Value::Int(a), Value::Int(b)) = (&current, &val) {
                                self.variables.insert(assign.name.clone(), Value::Int(a + b));
                            }
                        }
                        AssignOp::SubEq => {
                            let current = self
                                .variables
                                .get(&assign.name)
                                .cloned()
                                .unwrap_or(Value::Int(0));
                            if let (Value::Int(a), Value::Int(b)) = (&current, &val) {
                                self.variables.insert(assign.name.clone(), Value::Int(a - b));
                            }
                        }
                    }
                }
                PrepStatement::IfElse(if_else) => {
                    if self.eval_bool(&if_else.condition) {
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
                    self.pending
                        .push_back(InternalEvent::Narration(resolved));
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

                    let actor_name = match self.resolve_string_or_error(
                        &actor_name_template,
                        "actor display name",
                    ) {
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
                    if self.eval_bool(&if_else.condition) {
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
                        let available = opt
                            .condition
                            .as_ref()
                            .map(|c| self.eval_bool(c))
                            .unwrap_or(true);
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
                    self.pending
                        .push_back(InternalEvent::Jump(target.clone()));
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

    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::IntLit(n) => Value::Int(*n),
            Expr::BoolLit(b) => Value::Bool(*b),
            Expr::StringLit(s) => Value::Str(self.resolve_string_best_effort(s)),
            Expr::VarRef { name, .. } => self
                .variables
                .get(name)
                .cloned()
                .unwrap_or(Value::Int(0)),
            Expr::BinOp { left, op, right } => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                match op {
                    BinOperator::Add => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                        _ => Value::Int(0),
                    },
                    BinOperator::Sub => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        _ => Value::Int(0),
                    },
                    BinOperator::EqEq => Value::Bool(l == r),
                    BinOperator::NotEq => Value::Bool(l != r),
                    BinOperator::Lt => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Bool(a < b),
                        _ => Value::Bool(false),
                    },
                    BinOperator::LtEq => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Bool(a <= b),
                        _ => Value::Bool(false),
                    },
                    BinOperator::Gt => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Bool(a > b),
                        _ => Value::Bool(false),
                    },
                    BinOperator::GtEq => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Bool(a >= b),
                        _ => Value::Bool(false),
                    },
                }
            }
        }
    }

    fn eval_bool(&self, expr: &Expr) -> bool {
        match self.eval_expr(expr) {
            Value::Bool(b) => b,
            Value::Int(n) => n != 0,
            _ => false,
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

    fn resolve_string_best_effort(&self, template: &str) -> String {
        match render_interpolated(template, |name| {
            self.variables
                .get(name)
                .map(Self::value_to_plain_text)
                .or_else(|| Some(format!("${{{}}}", name)))
        }) {
            Ok(value) => value,
            Err(_) => template
                .chars()
                .map(|ch| if ch == ESCAPED_DOLLAR_MARKER { '$' } else { ch })
                .collect(),
        }
    }

    fn value_to_plain_text(value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
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

fn eval_init_expr(expr: &Expr) -> Value {
    match expr {
        Expr::IntLit(n) => Value::Int(*n),
        Expr::BoolLit(b) => Value::Bool(*b),
        Expr::StringLit(s) => Value::Str(s.clone()),
        _ => Value::Int(0),
    }
}
