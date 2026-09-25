use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::adapters::ast;
use crate::contract::{
    Choice, HARD_LIMITS, MediaEffect, PlayerLimits, RuntimeError, SemanticEvent,
};
use crate::model::*;
use crate::session_rng::{RngState, SessionRng};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use storyscript_parser::ast::Script as ParserScript;
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
    pub portraits: HashMap<String, String>,
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

#[derive(Clone)]
enum InternalEvent {
    Scene(String, Vec<MediaEffect>),
    Media(MediaEffect),
    Error(RuntimeError),
    Narration(String),
    Dialogue {
        actor_name: String,
        actor_id: String,
        emotion: Option<String>,
        position: Option<String>,
        portrait_path: Option<String>,
        text: String,
    },
    Choices(Vec<ChoiceDisplay>),
    Jump(String),
    End,
}

#[derive(Debug, Clone)]
pub(crate) enum PendingEvent {
    Event(SemanticEvent, Vec<MediaEffect>),
    Jump(String),
}

#[derive(Clone)]
pub(crate) struct EngineState {
    pub variables: HashMap<String, Value>,
    pub local_variables: HashMap<String, Value>,
    pub local_var_types: HashMap<String, VarType>,
    pub current_scene: String,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pub pending: Vec<PendingEvent>,
    pub finished: bool,
    pub rng: RngState,
    pub active_choices: Option<Vec<ChoiceDisplay>>,
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

// Static story indexes are constructed independently of scene entry; a future
// restore can build these without executing the current scene's PREP/STORY.
struct StoryIndexes {
    model: StoryModel,
    scenes: HashMap<String, Scene>,
    logic_blocks: HashMap<String, LogicBlock>,
    actors: HashMap<String, ActorInfo>,
    var_types: HashMap<String, VarType>,
}

impl StoryIndexes {
    fn from_model(script: &StoryModel) -> Self {
        let var_types = script
            .init
            .variables
            .iter()
            .map(|var| (var.name.clone(), var.var_type))
            .collect();
        let actors = script
            .init
            .actors
            .iter()
            .map(|actor| {
                (
                    actor.id.clone(),
                    ActorInfo {
                        display_name: actor.display_name.clone(),
                        portraits: actor
                            .portraits
                            .iter()
                            .map(|p| (p.emotion.clone(), p.path.clone()))
                            .collect(),
                    },
                )
            })
            .collect();
        let scenes = script
            .scenes
            .iter()
            .map(|scene| (scene.label.clone(), scene.clone()))
            .collect();
        let logic_blocks = script
            .logic_blocks
            .iter()
            .map(|logic| (logic.name.clone(), logic.clone()))
            .collect();
        Self {
            model: script.clone(),
            scenes,
            logic_blocks,
            actors,
            var_types,
        }
    }
}

#[derive(Clone)]
pub struct Engine {
    indexes: Arc<StoryIndexes>,
    pub variables: HashMap<String, Value>,
    local_variables: HashMap<String, Value>,
    local_var_types: HashMap<String, VarType>,
    pub current_scene: String,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pending: VecDeque<InternalEvent>,
    pub finished: bool,
    rng: SessionRng,
    scene_effects: Vec<MediaEffect>,
    limits: PlayerLimits,
    operations: usize,
    call_depth: usize,
    last_error: Option<RuntimeError>,
    active_choices: Option<Vec<ChoiceDisplay>>,
}

impl Engine {
    pub fn new(script: &ParserScript) -> Self {
        Self::from_model(&ast::adapt(script))
    }

    pub fn from_model(script: &StoryModel) -> Self {
        let mut engine = Self::without_execution(script, SessionRng::random());
        engine.current_scene = script.init.start.clone();
        if engine.initialize(&script.init.variables) {
            engine.enter_scene(&script.init.start);
        }
        engine
    }

    /// Explicit seeds make a session reproducible, including INIT expressions.
    pub fn new_seeded(script: &ParserScript, seed: [u8; 32]) -> Self {
        Self::from_model_seeded(&ast::adapt(script), seed)
    }

    pub fn from_model_seeded(script: &StoryModel, seed: [u8; 32]) -> Self {
        let mut engine = Self::without_execution(script, SessionRng::from_seed(seed));
        engine.current_scene = script.init.start.clone();
        if engine.initialize(&script.init.variables) {
            engine.enter_scene(&script.init.start);
        }
        engine
    }

    /// Fail closed during scene opening; no candidate engine escapes on error.
    pub fn open_checked(
        script: &ParserScript,
        seed: [u8; 32],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        Self::open_model_checked(&ast::adapt(script), seed, limits)
    }

    pub fn open_model_checked(
        script: &StoryModel,
        seed: [u8; 32],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let limits = limits.lowered().map_err(|invalid| RuntimeError {
            code: "R_LIMIT_CONFIGURATION".into(),
            scene: script.init.start.clone(),
            message: format!("{} exceeds hard limits or is zero", invalid.resource),
            resource: Some(invalid.resource.into()),
            actual: Some(invalid.requested as u64),
            limit: Some(invalid.hard_maximum as u64),
        })?;
        let mut candidate = Self::without_execution(script, SessionRng::from_seed(seed));
        candidate.limits = limits;
        candidate.current_scene = script.init.start.clone();
        if candidate.initialize(&script.init.variables) {
            candidate.enter_scene(&script.init.start);
        }
        if let Some(error) = candidate.last_error.take() {
            return Err(error);
        }
        Ok(candidate)
    }

    /// A failed interaction cannot mutate a previously published checkpoint.
    pub fn advance_checked(
        &mut self,
    ) -> Result<Option<(SemanticEvent, Vec<MediaEffect>)>, RuntimeError> {
        if self.active_choices.is_some() {
            return Err(self.action_error(
                "R_INVALID_ACTION",
                "choose an available option before advancing",
            ));
        }
        let mut candidate = self.clone();
        candidate.operations = 0;
        candidate.last_error = None;
        let event = candidate.step_semantic();
        if let Some(error) = candidate.last_error.take() {
            return Err(error);
        }
        *self = candidate;
        Ok(event)
    }

    pub fn choose_checked(
        &mut self,
        choice: &ChoiceDisplay,
    ) -> Result<Option<(SemanticEvent, Vec<MediaEffect>)>, RuntimeError> {
        if !self.active_choices.as_ref().is_some_and(|available| {
            available
                .iter()
                .any(|item| item.text == choice.text && item.target == choice.target)
        }) {
            return Err(self.action_error("R_INVALID_CHOICE", "choice is not available"));
        }
        let mut candidate = self.clone();
        candidate.operations = 0;
        candidate.last_error = None;
        candidate.active_choices = None;
        candidate.select_choice(choice);
        let event = candidate.step_semantic();
        if let Some(error) = candidate.last_error.take() {
            return Err(error);
        }
        *self = candidate;
        Ok(event)
    }

    fn action_error(&self, code: &str, message: &str) -> RuntimeError {
        RuntimeError {
            code: code.into(),
            scene: self.current_scene.clone(),
            message: message.into(),
            resource: None,
            actual: None,
            limit: None,
        }
    }

    // This constructor builds only story indexes. Restore can inject validated
    // mutable state here without rerunning INIT or entering a scene.
    fn without_execution(script: &StoryModel, rng: SessionRng) -> Self {
        Self {
            indexes: Arc::new(StoryIndexes::from_model(script)),
            variables: HashMap::new(),
            local_variables: HashMap::new(),
            local_var_types: HashMap::new(),
            current_scene: String::new(),
            bg: None,
            bgm: None,
            pending: VecDeque::new(),
            finished: false,
            rng,
            scene_effects: Vec::new(),
            limits: HARD_LIMITS,
            operations: 0,
            call_depth: 0,
            last_error: None,
            active_choices: None,
        }
    }

    pub fn rng_state(&self) -> RngState {
        self.rng.state()
    }

    pub(crate) fn model(&self) -> &StoryModel {
        &self.indexes.model
    }

    pub(crate) fn snapshot(&self) -> EngineState {
        EngineState {
            variables: self.variables.clone(),
            local_variables: self.local_variables.clone(),
            local_var_types: self.local_var_types.clone(),
            current_scene: self.current_scene.clone(),
            bg: self.bg.clone(),
            bgm: self.bgm.clone(),
            pending: self.pending.iter().map(internal_to_pending).collect(),
            finished: self.finished,
            rng: self.rng.state(),
            active_choices: self.active_choices.clone(),
        }
    }

    pub(crate) fn restore_model(
        model: &StoryModel,
        state: EngineState,
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let limits = limits.lowered().map_err(|invalid| RuntimeError {
            code: "R_LIMIT_CONFIGURATION".into(),
            scene: state.current_scene.clone(),
            message: format!("{} exceeds hard limits or is zero", invalid.resource),
            resource: Some(invalid.resource.into()),
            actual: Some(invalid.requested as u64),
            limit: Some(invalid.hard_maximum as u64),
        })?;
        let rng = SessionRng::from_state(state.rng).map_err(|message| RuntimeError {
            code: "R_SAVE_STATE_CORRUPT".into(),
            scene: state.current_scene.clone(),
            message: message.to_string(),
            resource: None,
            actual: None,
            limit: None,
        })?;
        let mut engine = Self::without_execution(model, rng);
        engine.variables = state.variables;
        engine.local_variables = state.local_variables;
        engine.local_var_types = state.local_var_types;
        engine.current_scene = state.current_scene;
        engine.bg = state.bg;
        engine.bgm = state.bgm;
        engine.pending = state.pending.into_iter().map(pending_to_internal).collect();
        engine.finished = state.finished;
        engine.active_choices = state.active_choices;
        engine.limits = limits;
        Ok(engine)
    }

    fn initialize(&mut self, declarations: &[VarDecl]) -> bool {
        for var in declarations {
            if !self.charge() {
                return false;
            }
            let raw = match self.eval_expr(&var.value, Some(var.var_type)) {
                Some(value) => value,
                None => return false,
            };
            let value = match coerce_value_for_type(raw, var.var_type) {
                Some(value) => value,
                None => {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "INIT value for '${}' is incompatible with {}",
                            var.name,
                            type_name(var.var_type)
                        ),
                    );
                    return false;
                }
            };
            if !self.ensure_value_limits(&value) {
                return false;
            }
            self.variables.insert(var.name.clone(), value);
        }
        true
    }

    fn charge(&mut self) -> bool {
        if self.operations >= self.limits.operations_per_interaction {
            self.limit_error(
                "operations_per_interaction",
                self.operations.saturating_add(1),
                self.limits.operations_per_interaction,
            );
            return false;
        }
        self.operations += 1;
        true
    }

    fn limit_error(&mut self, resource: &str, actual: usize, limit: usize) {
        let error = RuntimeError {
            code: "R_EXECUTION_LIMIT".into(),
            scene: self.current_scene.clone(),
            message: format!("{} exceeded: {} > {}", resource, actual, limit),
            resource: Some(resource.into()),
            actual: Some(actual as u64),
            limit: Some(limit as u64),
        };
        self.raise_runtime_error(&error.code, error.message.clone());
        self.last_error = Some(error.clone());
        self.pending.clear();
        self.pending.push_back(InternalEvent::Error(error));
        self.pending.push_back(InternalEvent::End);
    }

    fn push_event(&mut self, event: InternalEvent) -> bool {
        if self.pending.len() >= self.limits.pending_events_per_scene {
            self.limit_error(
                "pending_events_per_scene",
                self.pending.len().saturating_add(1),
                self.limits.pending_events_per_scene,
            );
            false
        } else {
            self.pending.push_back(event);
            true
        }
    }

    fn push_effect(&mut self, effect: MediaEffect) -> bool {
        if self.scene_effects.len() >= self.limits.pending_events_per_scene {
            self.limit_error(
                "pending_events_per_scene",
                self.scene_effects.len().saturating_add(1),
                self.limits.pending_events_per_scene,
            );
            false
        } else {
            self.scene_effects.push(effect);
            true
        }
    }

    // -----------------------------------------------------------------------
    // Scene entry
    // -----------------------------------------------------------------------

    fn enter_scene(&mut self, label: &str) {
        if !self.charge() {
            return;
        }
        let scene = match self.indexes.scenes.get(label) {
            Some(s) => s.clone(),
            None => {
                self.finished = true;
                return;
            }
        };

        self.current_scene = label.to_string();
        self.local_variables.clear();
        self.local_var_types.clear();
        self.scene_effects.clear();

        // Execute #PREP (silent — modifies state, sets assets)
        if !scene.prep.is_empty() {
            if !self.execute_prep(&scene.prep) {
                return;
            }
        }

        let effects = std::mem::take(&mut self.scene_effects);
        if !self.push_event(InternalEvent::Scene(label.to_string(), effects)) {
            return;
        }

        // Flatten #STORY into pending events
        self.flatten_story(&scene.story);
    }

    // -----------------------------------------------------------------------
    // #PREP execution
    // -----------------------------------------------------------------------

    fn execute_prep(&mut self, stmts: &[PrepStatement]) -> bool {
        matches!(
            self.execute_prep_block(stmts, false, false, None),
            PrepFlow::Next
        )
    }

    fn execute_prep_block(
        &mut self,
        stmts: &[PrepStatement],
        in_loop: bool,
        in_logic: bool,
        logic_return_type: Option<VarType>,
    ) -> PrepFlow {
        for stmt in stmts {
            if !self.charge() {
                return PrepFlow::Error;
            }
            match stmt {
                PrepStatement::BgDirective { path, .. } => {
                    let resolved = match self.resolve_string_or_error(path, "@bg path") {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };
                    self.bg = Some(resolved);
                    if !self.push_effect(MediaEffect::Background(self.bg.as_ref().unwrap().clone()))
                    {
                        return PrepFlow::Error;
                    }
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
                    let effect = match &self.bgm {
                        Some(path) => MediaEffect::Bgm(path.clone()),
                        None => MediaEffect::BgmStop,
                    };
                    if !self.push_effect(effect) {
                        return PrepFlow::Error;
                    }
                }
                PrepStatement::SfxDirective { path, .. } => {
                    let resolved = match self.resolve_string_or_error(path, "@sfx path") {
                        Some(value) => value,
                        None => return PrepFlow::Error,
                    };
                    if !self.push_effect(MediaEffect::Sfx(resolved)) {
                        return PrepFlow::Error;
                    }
                }
                PrepStatement::VarDecl(decl) => {
                    if self.indexes.var_types.contains_key(&decl.name)
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

                    self.local_var_types
                        .insert(decl.name.clone(), decl.var_type);
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
                                        AssignOp::AddEq => a.checked_add(b).map(Value::Int),
                                        AssignOp::SubEq => a.checked_sub(b).map(Value::Int),
                                        AssignOp::Set => Some(Value::Int(a)),
                                    };
                                    let updated = match updated {
                                        Some(value) => value,
                                        None => {
                                            self.raise_runtime_error(
                                                "R_NUMERIC_OVERFLOW",
                                                format!(
                                                    "integer assignment overflow for ${}",
                                                    assign.name
                                                ),
                                            );
                                            return PrepFlow::Error;
                                        }
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
                PrepStatement::Call { name, args, span } => {
                    if self
                        .eval_call(
                            name,
                            args,
                            span.line,
                            span.column,
                            None,
                            CallMode::Statement,
                        )
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
                        self.execute_prep_block(else_branch, in_loop, in_logic, logic_return_type)
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

                    let previous_type = self
                        .local_var_types
                        .insert(loop_stmt.item_name.clone(), element_type);
                    let previous_value = self.local_variables.remove(&loop_stmt.item_name);

                    for item in snapshot_items {
                        if !self.charge() {
                            return PrepFlow::Error;
                        }
                        self.local_variables
                            .insert(loop_stmt.item_name.clone(), item);

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
                        if !self.charge() {
                            return PrepFlow::Error;
                        }
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
            if !self.charge() {
                return StoryFlow::Error;
            }
            let is_last_stmt = idx + 1 == len;

            match stmt {
                StoryStatement::Narration { text, .. } => {
                    let resolved = match self.resolve_string_or_error(text, "narration") {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };
                    if !self.push_event(InternalEvent::Narration(resolved)) {
                        return StoryFlow::Error;
                    }
                }
                StoryStatement::VarOutput { name, .. } => {
                    if let Some(value) = self.resolve_var_value(name) {
                        let value = value.clone();
                        let text = match self.render_value(&value) {
                            Some(text) => text,
                            None => return StoryFlow::Error,
                        };
                        if !self.push_event(InternalEvent::Narration(text)) {
                            return StoryFlow::Error;
                        }
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
                        .indexes
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

                    let (emotion, position, portrait_path) = match &dlg.form {
                        DialogueForm::NameOnly => (None, None, None),
                        DialogueForm::Portrait { emotion, position } => {
                            let pos_str = match position {
                                Position::Left => "Left",
                                Position::Center => "Center",
                                Position::Right => "Right",
                            };
                            let path = self
                                .indexes
                                .actors
                                .get(&dlg.actor_id)
                                .and_then(|actor| actor.portraits.get(emotion))
                                .cloned();
                            (Some(emotion.clone()), Some(pos_str.to_string()), path)
                        }
                    };

                    let text = match self.resolve_string_or_error(&dlg.text, "dialogue") {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };

                    if portrait_path
                        .as_ref()
                        .is_some_and(|path| path.len() > self.limits.rendered_bytes)
                    {
                        self.limit_error(
                            "rendered_bytes",
                            portrait_path.as_ref().unwrap().len(),
                            self.limits.rendered_bytes,
                        );
                        return StoryFlow::Error;
                    }

                    if !self.push_event(InternalEvent::Dialogue {
                        actor_name,
                        actor_id: dlg.actor_id.clone(),
                        emotion,
                        position,
                        portrait_path,
                        text,
                    }) {
                        return StoryFlow::Error;
                    }
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

                    if !self.push_event(InternalEvent::Choices(options)) {
                        return StoryFlow::Error;
                    }
                    return StoryFlow::Terminated;
                }
                StoryStatement::Jump { target, .. } => {
                    if !self.push_event(InternalEvent::Jump(target.clone())) {
                        return StoryFlow::Error;
                    }
                    return StoryFlow::Terminated;
                }
                StoryStatement::End { .. } => {
                    if !self.push_event(InternalEvent::End) {
                        return StoryFlow::Error;
                    }
                    return StoryFlow::Terminated;
                }
                StoryStatement::SfxDirective { path, .. } => {
                    let resolved = match self.resolve_string_or_error(path, "@sfx path") {
                        Some(value) => value,
                        None => return StoryFlow::Error,
                    };
                    if !self.push_event(InternalEvent::Media(MediaEffect::Sfx(resolved))) {
                        return StoryFlow::Error;
                    }
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
            if !self.charge() {
                return StoryFlow::Error;
            }
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
                    if !self.charge() {
                        return StoryFlow::Error;
                    }
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
                    if !self.charge() {
                        return StoryFlow::Error;
                    }
                    self.local_variables
                        .insert(loop_entry.item_name.clone(), item);
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
        let (snapshot_items, element_type) =
            match self.resolve_snapshot_array(&loop_stmt.array_name) {
                Some(result) => result,
                None => return StoryFlow::Error,
            };

        let total_iterations = snapshot_items.len();
        let previous_type = self
            .local_var_types
            .insert(loop_stmt.item_name.clone(), element_type);
        let previous_value = self.local_variables.remove(&loop_stmt.item_name);

        for (idx, item) in snapshot_items.into_iter().enumerate() {
            if !self.charge() {
                return StoryFlow::Error;
            }
            self.local_variables
                .insert(loop_stmt.item_name.clone(), item);

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

                    self.restore_loop_binding(&loop_stmt.item_name, previous_type, previous_value);
                    return StoryFlow::Terminated;
                }
                StoryFlow::Error => {
                    self.restore_loop_binding(&loop_stmt.item_name, previous_type, previous_value);
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
            if !self.charge() {
                return StoryFlow::Error;
            }
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
            if !self.charge() {
                return self.last_error.take().map(|error| {
                    StepResult::Narration(format!("[{}] {}", error.code, error.message))
                });
            }
            match self.pending.pop_front() {
                Some(InternalEvent::Scene(label, _)) => {
                    return Some(StepResult::Narration(format!("─── Scene: {} ───", label)));
                }
                Some(InternalEvent::Media(_)) => continue,
                Some(InternalEvent::Error(error)) => {
                    return Some(StepResult::Narration(format!(
                        "[{}] {}",
                        error.code, error.message
                    )));
                }
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
                    portrait_path: _,
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

    /// UI-independent event stream. Scene effects are emitted once and in PREP
    /// order; STORY SFX are standalone events. No legacy header is serialized.
    pub fn step_semantic(&mut self) -> Option<(SemanticEvent, Vec<MediaEffect>)> {
        loop {
            if !self.charge() {
                return None;
            }
            let next = match self.pending.pop_front() {
                Some(event) => event,
                None => return None,
            };
            return Some(match next {
                InternalEvent::Scene(label, effects) => {
                    (SemanticEvent::SceneTransition(label), effects)
                }
                InternalEvent::Media(effect) => (SemanticEvent::Media(effect), Vec::new()),
                InternalEvent::Error(error) => (SemanticEvent::Error(error), Vec::new()),
                InternalEvent::Narration(text) => (SemanticEvent::Narration(text), Vec::new()),
                InternalEvent::Dialogue {
                    actor_name,
                    actor_id,
                    emotion,
                    position,
                    portrait_path,
                    text,
                } => (
                    SemanticEvent::Dialogue {
                        actor_name,
                        actor_id,
                        emotion,
                        position,
                        portrait_path,
                        text,
                    },
                    Vec::new(),
                ),
                InternalEvent::Choices(options) => {
                    self.active_choices = Some(options.clone());
                    (
                        SemanticEvent::Choices(
                            options
                                .into_iter()
                                .map(|item| Choice {
                                    text: item.text,
                                    target_scene: item.target,
                                })
                                .collect(),
                        ),
                        Vec::new(),
                    )
                }
                InternalEvent::End => {
                    self.finished = true;
                    (SemanticEvent::End, Vec::new())
                }
                InternalEvent::Jump(target) => {
                    self.pending.clear();
                    self.enter_scene(&target);
                    continue;
                }
            });
        }
    }

    /// Execute a player's choice — enter the target scene.
    pub fn select_choice(&mut self, choice: &ChoiceDisplay) {
        self.active_choices = None;
        self.pending.clear();
        self.enter_scene(&choice.target);
    }

    // -----------------------------------------------------------------------
    // Expression evaluation
    // -----------------------------------------------------------------------

    fn eval_expr(&mut self, expr: &Expr, assignment_target: Option<VarType>) -> Option<Value> {
        if !self.charge() {
            return None;
        }
        let result = match expr {
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
            Expr::Call { name, args, span } => self.eval_call(
                name,
                args,
                span.line,
                span.column,
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
        };
        result.and_then(|value| {
            if self.ensure_value_limits(&value) {
                Some(value)
            } else {
                None
            }
        })
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
            if *b == 0 && matches!(op, "/" | "%") {
                self.raise_runtime_error(
                    if op == "/" {
                        "R_DIVIDE_BY_ZERO"
                    } else {
                        "R_MODULO_BY_ZERO"
                    },
                    if op == "/" {
                        "Division by zero is not allowed".into()
                    } else {
                        "Modulo by zero is not allowed".into()
                    },
                );
                return None;
            }
            let result = match op {
                "+" => a.checked_add(*b),
                "-" => a.checked_sub(*b),
                "*" => a.checked_mul(*b),
                "/" => a.checked_div(*b),
                "%" => a.checked_rem(*b),
                _ => Some(*a),
            };
            return result.map(Value::Int).or_else(|| {
                self.raise_runtime_error(
                    "R_NUMERIC_OVERFLOW",
                    format!("integer operator '{op}' overflowed"),
                );
                None
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
                self.raise_runtime_error("RUNTIME", format!("Unknown numeric operator '{}'", op));
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
        if items.len() > self.limits.array_elements {
            self.limit_error("array_elements", items.len(), self.limits.array_elements);
            return None;
        }
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
                self.raise_runtime_error("RUNTIME", "Nested arrays are not supported".to_string());
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
            Expr::ListLit { items, .. } => {
                match self.eval_array_literal(items, assignment_target_hint) {
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
                }
            }
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
        let value =
            self.eval_scalar_argument(expr, Some(VarType::Integer), function_name, "index")?;
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
        if !self.charge() {
            return None;
        }
        if self.indexes.logic_blocks.contains_key(name) {
            if self.call_depth >= self.limits.logic_depth {
                self.limit_error(
                    "logic_depth",
                    self.call_depth.saturating_add(1),
                    self.limits.logic_depth,
                );
                return None;
            }
            self.call_depth += 1;
            let result = self.eval_logic_call(name, args, line, column, assignment_target, mode);
            self.call_depth -= 1;
            return result;
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
                        VarType::Integer => Some(Value::Int(self.rng.sample::<i64>())),
                        VarType::Decimal => {
                            let sample = self.rng.range(0.0f64..=1.0f64);
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

                            Some(Value::Int(self.rng.range(min..=max)))
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

                            let sample = self.rng.range(min_f..=max_f);
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
                    let (items, _element_type, _) =
                        self.eval_array_argument(&args[0], None, "pick")?;
                    if items.is_empty() {
                        self.raise_runtime_error(
                            "R_ARRAY_EMPTY",
                            "pick() requires non-empty array".to_string(),
                        );
                        return None;
                    }

                    let index = self.rng.range(0..items.len());
                    return Some(items[index].clone());
                }

                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!("pick() expects 1 or 2 arguments, found {}", args.len()),
                    );
                    return None;
                }

                let count_value =
                    self.eval_scalar_argument(&args[0], Some(VarType::Integer), "pick", "count")?;
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
                    let random_index = self.rng.range(0..pool.len());
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
                        format!(
                            "array_push() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
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
                let value =
                    self.eval_scalar_argument(&args[1], Some(element_type), "array_push", "value")?;
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
                if items.len() >= self.limits.array_elements {
                    self.limit_error(
                        "array_elements",
                        items.len().saturating_add(1),
                        self.limits.array_elements,
                    );
                    return None;
                }
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
                        format!(
                            "array_pop() expects exactly 1 argument, found {}",
                            args.len()
                        ),
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
                        format!(
                            "array_strip() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_strip() returns void and cannot be used as an expression"
                            .to_string(),
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
                        format!(
                            "array_clear() expects exactly 1 argument, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        "array_clear() returns void and cannot be used as an expression"
                            .to_string(),
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
                        format!(
                            "array_size() expects exactly 1 argument, found {}",
                            args.len()
                        ),
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
                        format!(
                            "array_join() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
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

                let mut total = separator
                    .len()
                    .saturating_mul(items.len().saturating_sub(1));
                let mut parts = Vec::with_capacity(items.len());
                for item in &items {
                    let part = self.render_value(item)?;
                    total = total.saturating_add(part.len());
                    if total > self.limits.rendered_bytes {
                        self.limit_error("rendered_bytes", total, self.limits.rendered_bytes);
                        return None;
                    }
                    parts.push(part);
                }
                Some(Value::Str(parts.join(&separator)))
            }
            "array_get" => {
                if args.len() != 2 {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "array_get() expects exactly 2 arguments, found {}",
                            args.len()
                        ),
                    );
                    return None;
                }

                let (items, _element_type, _) =
                    self.eval_array_argument(&args[0], None, "array_get")?;
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

                if items.len() >= self.limits.array_elements {
                    self.limit_error(
                        "array_elements",
                        items.len().saturating_add(1),
                        self.limits.array_elements,
                    );
                    return None;
                }
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
        let logic = match self.indexes.logic_blocks.get(name).cloned() {
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

            self.local_var_types
                .insert(param.name.clone(), param.var_type);
            self.local_variables.insert(param.name.clone(), coerced);
        }

        let flow = self.execute_prep_block(&logic.body, false, true, logic.return_type);
        let result = match (logic.return_type, flow) {
            (_, PrepFlow::Error) => None,
            (_, PrepFlow::BreakLoop | PrepFlow::ContinueLoop) => {
                self.raise_runtime_error(
                    "RUNTIME",
                    format!(
                        "Logic function '{}' terminated with invalid loop control",
                        name
                    ),
                );
                None
            }
            (None, PrepFlow::Next) => {
                if mode == CallMode::Expression {
                    self.raise_runtime_error(
                        "RUNTIME",
                        format!(
                            "{}() returns void and cannot be used as an expression",
                            name
                        ),
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
        if template.len() > self.limits.rendered_bytes {
            self.limit_error("rendered_bytes", template.len(), self.limits.rendered_bytes);
            return None;
        }
        match self.resolve_string(template) {
            Ok(value) if value.len() <= self.limits.rendered_bytes => Some(value),
            Ok(value) => {
                self.limit_error("rendered_bytes", value.len(), self.limits.rendered_bytes);
                None
            }
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
        let mut replacement_bytes = 0usize;
        render_interpolated(template, |name| {
            self.resolve_var_value(name).and_then(|value| {
                let rendered = Self::value_to_plain_text(value);
                replacement_bytes = replacement_bytes.saturating_add(rendered.len());
                (replacement_bytes <= self.limits.rendered_bytes).then_some(rendered)
            })
        })
        .map_err(|e| e.message)
    }

    fn resolve_var_type(&self, name: &str) -> Option<VarType> {
        self.local_var_types
            .get(name)
            .copied()
            .or_else(|| self.indexes.var_types.get(name).copied())
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

        usize::try_from(raw_count).ok().or_else(|| {
            self.raise_runtime_error(
                "R_REPEAT_COUNT_INVALID",
                format!("repeat count {} cannot be represented", raw_count),
            );
            None
        })
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
                    format!(
                        "Read of undeclared variable '${}' in for snapshot source",
                        name
                    ),
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

    fn render_value(&mut self, value: &Value) -> Option<String> {
        if !self.ensure_value_limits(value) {
            return None;
        }
        let rendered = Self::value_to_plain_text(value);
        if rendered.len() > self.limits.rendered_bytes {
            self.limit_error("rendered_bytes", rendered.len(), self.limits.rendered_bytes);
            None
        } else {
            Some(rendered)
        }
    }

    fn ensure_value_limits(&mut self, value: &Value) -> bool {
        match value {
            Value::Str(text) => {
                if text.len() > self.limits.rendered_bytes {
                    self.limit_error("rendered_bytes", text.len(), self.limits.rendered_bytes);
                    false
                } else {
                    true
                }
            }
            Value::Array { items, .. } => {
                if items.len() > self.limits.array_elements {
                    self.limit_error("array_elements", items.len(), self.limits.array_elements);
                    return false;
                }
                items.iter().all(|item| self.ensure_value_limits(item))
            }
            _ => true,
        }
    }

    fn raise_runtime_error(&mut self, code: &str, message: String) {
        self.pending.clear();
        self.scene_effects.clear();
        let error = RuntimeError {
            code: code.to_string(),
            scene: self.current_scene.clone(),
            message,
            resource: None,
            actual: None,
            limit: None,
        };
        self.last_error = Some(error.clone());
        self.pending.push_back(InternalEvent::Error(error));
        self.pending.push_back(InternalEvent::End);
        self.finished = true;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn internal_to_pending(event: &InternalEvent) -> PendingEvent {
    match event {
        InternalEvent::Scene(scene, effects) => PendingEvent::Event(
            SemanticEvent::SceneTransition(scene.clone()),
            effects.clone(),
        ),
        InternalEvent::Media(effect) => {
            PendingEvent::Event(SemanticEvent::Media(effect.clone()), Vec::new())
        }
        InternalEvent::Error(error) => {
            PendingEvent::Event(SemanticEvent::Error(error.clone()), Vec::new())
        }
        InternalEvent::Narration(text) => {
            PendingEvent::Event(SemanticEvent::Narration(text.clone()), Vec::new())
        }
        InternalEvent::Dialogue {
            actor_name,
            actor_id,
            emotion,
            position,
            portrait_path,
            text,
        } => PendingEvent::Event(
            SemanticEvent::Dialogue {
                actor_name: actor_name.clone(),
                actor_id: actor_id.clone(),
                emotion: emotion.clone(),
                position: position.clone(),
                portrait_path: portrait_path.clone(),
                text: text.clone(),
            },
            Vec::new(),
        ),
        InternalEvent::Choices(items) => PendingEvent::Event(
            SemanticEvent::Choices(
                items
                    .iter()
                    .map(|item| Choice {
                        text: item.text.clone(),
                        target_scene: item.target.clone(),
                    })
                    .collect(),
            ),
            Vec::new(),
        ),
        InternalEvent::Jump(target) => PendingEvent::Jump(target.clone()),
        InternalEvent::End => PendingEvent::Event(SemanticEvent::End, Vec::new()),
    }
}

fn pending_to_internal(event: PendingEvent) -> InternalEvent {
    match event {
        PendingEvent::Jump(target) => InternalEvent::Jump(target),
        PendingEvent::Event(SemanticEvent::SceneTransition(scene), effects) => {
            InternalEvent::Scene(scene, effects)
        }
        PendingEvent::Event(SemanticEvent::Narration(text), _) => InternalEvent::Narration(text),
        PendingEvent::Event(
            SemanticEvent::Dialogue {
                actor_name,
                actor_id,
                emotion,
                position,
                portrait_path,
                text,
            },
            _,
        ) => InternalEvent::Dialogue {
            actor_name,
            actor_id,
            emotion,
            position,
            portrait_path,
            text,
        },
        PendingEvent::Event(SemanticEvent::Choices(items), _) => InternalEvent::Choices(
            items
                .into_iter()
                .map(|item| ChoiceDisplay {
                    text: item.text,
                    target: item.target_scene,
                })
                .collect(),
        ),
        PendingEvent::Event(SemanticEvent::Media(effect), _) => InternalEvent::Media(effect),
        PendingEvent::Event(SemanticEvent::End, _) => InternalEvent::End,
        PendingEvent::Event(SemanticEvent::Error(error), _) => InternalEvent::Error(error),
    }
}

#[allow(dead_code)]
fn eval_init_expr(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
    rng: &mut SessionRng,
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
        Expr::Call { name, args, .. } => eval_init_call(name, args, vars, assignment_target, rng),
        Expr::ListLit { items, .. } => eval_init_array_literal(items, vars, assignment_target, rng),
        Expr::BinOp { left, op, right } => {
            let l = eval_init_expr(left, vars, assignment_target, rng)?;
            let r = eval_init_expr(right, vars, assignment_target, rng)?;
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn eval_init_array_literal(
    items: &[Expr],
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
    rng: &mut SessionRng,
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
            let value = eval_init_expr(item, vars, Some(element_type), rng)?;
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
        let value = eval_init_expr(item, vars, None, rng)?;
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

#[allow(dead_code)]
fn eval_init_array_argument(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target_hint: Option<VarType>,
    rng: &mut SessionRng,
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
            let value = eval_init_array_literal(items, vars, assignment_target_hint, rng)?;
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

#[allow(dead_code)]
fn eval_init_scalar_argument(
    expr: &Expr,
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
    rng: &mut SessionRng,
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

    let value = eval_init_expr(expr, vars, assignment_target, rng)?;
    if is_array_type(value_type(&value)) {
        return None;
    }

    Some(value)
}

#[allow(dead_code)]
fn eval_init_call(
    name: &str,
    args: &[Expr],
    vars: &HashMap<String, Value>,
    assignment_target: Option<VarType>,
    rng: &mut SessionRng,
) -> Option<Value> {
    match name {
        "abs" => {
            if args.len() != 1 {
                return None;
            }

            match eval_init_expr(&args[0], vars, assignment_target, rng)? {
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
                    VarType::Integer => Some(Value::Int(rng.sample::<i64>())),
                    VarType::Decimal => Decimal::from_f64(rng.range(0.0..=1.0)).map(Value::Decimal),
                    _ => None,
                },
                2 => match target {
                    VarType::Integer => {
                        let min = eval_init_expr(&args[0], vars, assignment_target, rng)?;
                        let max = eval_init_expr(&args[1], vars, assignment_target, rng)?;
                        let (min, max) = match (min, max) {
                            (Value::Int(min), Value::Int(max)) => (min, max),
                            _ => return None,
                        };
                        if min > max {
                            return None;
                        }
                        Some(Value::Int(rng.range(min..=max)))
                    }
                    VarType::Decimal => {
                        let min =
                            as_decimal(&eval_init_expr(&args[0], vars, assignment_target, rng)?)?;
                        let max =
                            as_decimal(&eval_init_expr(&args[1], vars, assignment_target, rng)?)?;

                        if min > max {
                            return None;
                        }

                        let min_f = min.to_f64()?;
                        let max_f = max.to_f64()?;
                        Decimal::from_f64(rng.range(min_f..=max_f)).map(Value::Decimal)
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        "pick" => {
            if args.len() == 1 {
                let (items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
                if items.is_empty() {
                    return None;
                }

                let index = rng.range(0..items.len());
                return Some(items[index].clone());
            }

            if args.len() != 2 {
                return None;
            }

            let count =
                match eval_init_scalar_argument(&args[0], vars, Some(VarType::Integer), rng)? {
                    Value::Int(n) if n >= 0 => n as usize,
                    _ => return None,
                };

            let hint = assignment_target.filter(|ty| is_array_type(*ty));
            let (items, element_type) = eval_init_array_argument(&args[1], vars, hint, rng)?;
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
                let random_index = rng.range(0..pool.len());
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

            let (mut items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            items.pop()
        }
        "array_contains" => {
            if args.len() != 2 {
                return None;
            }

            let (items, element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            let probe = eval_init_scalar_argument(&args[1], vars, Some(element_type), rng)?;
            let probe = coerce_value_for_type(probe, element_type)?;
            Some(Value::Bool(items.iter().any(|item| item == &probe)))
        }
        "array_size" => {
            if args.len() != 1 {
                return None;
            }

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            Some(Value::Int(items.len() as i64))
        }
        "array_join" => {
            if args.len() != 2 {
                return None;
            }

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            let separator =
                match eval_init_scalar_argument(&args[1], vars, Some(VarType::String), rng)? {
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

            let (items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            let index =
                match eval_init_scalar_argument(&args[1], vars, Some(VarType::Integer), rng)? {
                    Value::Int(n) if n >= 0 => n as usize,
                    _ => return None,
                };

            items.get(index).cloned()
        }
        "array_remove" => {
            if args.len() != 2 {
                return None;
            }

            let (mut items, _element_type) = eval_init_array_argument(&args[0], vars, None, rng)?;
            let index =
                match eval_init_scalar_argument(&args[1], vars, Some(VarType::Integer), rng)? {
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
