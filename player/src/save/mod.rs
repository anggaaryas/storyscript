//! Exact-match, progress-only save export and hostile-input restore.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use prost::Message;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};

use crate::contract::proto::storyplayer::v1 as w;
use crate::contract::{
    BundleSigner, Choice, HistoryEntry, MediaEffect, Origin, PlayerLimits, RuntimeError,
    SAVE_RUNTIME_VERSION, SAVE_SCHEMA_VERSION, SemanticEvent, SessionStatus, decode_save_contract,
};
use crate::engine::{ChoiceDisplay, Engine, EngineState, PendingEvent, Value};
use crate::history::HistoryBuffer;
use crate::model::{PrepStatement, StoryModel, VarType};
use crate::runtime::SemanticPlayer;
use crate::session_rng::RngState;

pub const RUNTIME_IDENTITY: &str = "storyscript-player/0.1.0:model-v1";
pub const PARSER_IDENTITY: &str = "storyscript-parser/0.1.0";
pub const COMPILER_IDENTITY: &str = "storyscript-compiler/0.1.0";

pub fn semantic_sha256(model: &StoryModel) -> String {
    // Every SourceSpan has a location-independent Debug implementation. The
    // runtime identity pins this representation for save v1.
    format!("{:x}", Sha256::digest(format!("{model:?}").as_bytes()))
}

pub fn source_origin(model: &StoryModel) -> Origin {
    Origin::Source {
        parser_version: PARSER_IDENTITY.into(),
        compiler_version: COMPILER_IDENTITY.into(),
        runtime_identity: RUNTIME_IDENTITY.into(),
        semantic_sha256: semantic_sha256(model),
    }
}

impl SemanticPlayer {
    pub fn export_save(&self) -> Result<Vec<u8>, RuntimeError> {
        let state = self.engine.snapshot();
        let mut save = w::PlayerSave {
            schema_version: SAVE_SCHEMA_VERSION,
            runtime_version: SAVE_RUNTIME_VERSION,
            origin: Some(origin_to_wire(&self.origin)),
            globals: variables_to_wire(&state.variables, &global_types(self.engine.model()))?,
            locals: variables_to_wire(&state.local_variables, &state.local_var_types)?,
            current_scene: state.current_scene,
            background: state.bg,
            bgm: state.bgm,
            current: Some(event_to_wire(&self.current.current, false)?),
            pending: state
                .pending
                .iter()
                .map(pending_to_wire)
                .collect::<Result<_, _>>()?,
            rng: Some(rng_to_wire(state.rng)),
            history: self
                .history
                .retained_entries()
                .iter()
                .map(history_to_wire)
                .collect::<Result<_, _>>()?,
            next_sequence: self.history.next_sequence(),
            first_retained_sequence: self.history.first_retained_sequence(),
            omitted_history_count: self.history.omitted_history_count(),
            status: status_to_wire(self.current.status) as i32,
            current_effects: self.current.effects.iter().map(effect_to_wire).collect(),
        };

        let mut bytes = save.encode_to_vec();
        while bytes.len() > self.limits.save_bytes && !save.history.is_empty() {
            save.history.remove(0);
            save.omitted_history_count = save.omitted_history_count.saturating_add(1);
            save.first_retained_sequence = save
                .history
                .first()
                .map_or(save.next_sequence, |entry| entry.sequence);
            bytes = save.encode_to_vec();
        }
        if bytes.len() > self.limits.save_bytes {
            return Err(save_error(
                "R_SAVE_LIMIT",
                &self.current.scene,
                "non-history save state exceeds byte limit",
            ));
        }
        Ok(bytes)
    }

    pub fn restore_source(
        source: &str,
        bytes: &[u8],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let compiled = storyscript_parser::compiler::compile_source(source);
        if compiled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
        {
            return Err(save_error(
                "R_SOURCE_COMPILE",
                "",
                "source failed to compile",
            ));
        }
        let script = compiled
            .script
            .ok_or_else(|| save_error("R_SOURCE_COMPILE", "", "compiler produced no story"))?;
        let model = crate::adapters::ast::adapt(&script);
        let origin = source_origin(&model);
        restore_model(model, origin, bytes, limits)
    }

    pub fn restore_file(
        path: &Path,
        bytes: &[u8],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let compiled = storyscript_parser::compiler::compile_file(path)
            .map_err(|_| save_error("R_SOURCE_COMPILE", "", "source path failed to compile"))?;
        if compiled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
        {
            return Err(save_error(
                "R_SOURCE_COMPILE",
                "",
                "source path failed to compile",
            ));
        }
        let script = compiled
            .script
            .ok_or_else(|| save_error("R_SOURCE_COMPILE", "", "compiler produced no story"))?;
        let model = crate::adapters::ast::adapt(&script);
        let origin = source_origin(&model);
        restore_model(model, origin, bytes, limits)
    }
}

pub(crate) fn restore_model(
    model: StoryModel,
    origin: Origin,
    bytes: &[u8],
    limits: PlayerLimits,
) -> Result<SemanticPlayer, RuntimeError> {
    let limits = limits.lowered().map_err(|invalid| RuntimeError {
        code: "R_LIMIT_CONFIGURATION".into(),
        scene: model.init.start.clone(),
        message: format!("invalid limit: {}", invalid.resource),
        resource: Some(invalid.resource.into()),
        actual: Some(invalid.requested as u64),
        limit: Some(invalid.hard_maximum as u64),
    })?;
    let save = decode_save_contract(bytes, limits)
        .map_err(|error| save_error(error.code, "", error.message))?;
    if save.origin.as_ref() != Some(&origin_to_wire(&origin)) {
        return Err(save_error(
            "R_SAVE_INCOMPATIBLE",
            &save.current_scene,
            "save origin or semantic identity does not match story",
        ));
    }
    validate_story_state(&model, &save, limits)?;

    let global_types = global_types(&model);
    let variables = variables_from_wire(&save.globals, &global_types)?;
    let allowed_locals = local_types_for_scene(&model, &save.current_scene);
    let local_variables = variables_from_wire(&save.locals, &allowed_locals)?;
    let local_var_types = save
        .locals
        .iter()
        .map(|item| Ok((item.name.clone(), type_from_wire(item.declared_type)?)))
        .collect::<Result<HashMap<_, _>, RuntimeError>>()?;
    let current = event_from_wire(
        save.current.as_ref().expect("contract requires current"),
        false,
    )?;
    let current_effects = save
        .current_effects
        .iter()
        .map(effect_from_wire)
        .collect::<Result<Vec<_>, _>>()?;
    let status = status_from_wire(save.status)?;
    let active_choices = match &current {
        SemanticEvent::Choices(items) => Some(
            items
                .iter()
                .map(|item| ChoiceDisplay {
                    text: item.text.clone(),
                    target: item.target_scene.clone(),
                })
                .collect(),
        ),
        _ => None,
    };
    let pending = save
        .pending
        .iter()
        .map(pending_from_wire)
        .collect::<Result<Vec<_>, _>>()?;
    let history_entries = save
        .history
        .iter()
        .map(history_from_wire)
        .collect::<Result<Vec<_>, _>>()?;
    let history = HistoryBuffer::restore(
        limits,
        history_entries,
        save.next_sequence,
        save.omitted_history_count,
    )
    .map_err(|message| save_error("R_SAVE_STATE_CORRUPT", &save.current_scene, message))?;
    if history.first_retained_sequence() != save.first_retained_sequence {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "history first-retained sequence mismatch",
        ));
    }
    if save.omitted_history_count != save.first_retained_sequence {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "history omitted count does not match absolute sequence",
        ));
    }

    let state = EngineState {
        variables,
        local_variables,
        local_var_types,
        current_scene: save.current_scene.clone(),
        bg: save.background,
        bgm: save.bgm,
        pending,
        finished: status != SessionStatus::Active,
        rng: rng_from_wire(save.rng.as_ref().expect("contract requires rng")),
        active_choices,
    };
    let engine = Engine::restore_model(&model, state, limits)?;
    Ok(SemanticPlayer {
        engine,
        current: crate::contract::EventDelta {
            current,
            effects: current_effects,
            scene: save.current_scene,
            status,
            sequence: save.next_sequence,
            first_retained_sequence: save.first_retained_sequence,
            omitted_history_count: save.omitted_history_count,
        },
        history,
        origin,
        limits,
    })
}

fn validate_story_state(
    model: &StoryModel,
    save: &w::PlayerSave,
    limits: PlayerLimits,
) -> Result<(), RuntimeError> {
    let scenes: HashSet<&str> = model
        .scenes
        .iter()
        .map(|scene| scene.label.as_str())
        .collect();
    if !scenes.contains(save.current_scene.as_str()) {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "unknown current scene",
        ));
    }
    let globals = global_types(model);
    if save.globals.len() != globals.len()
        || save
            .globals
            .iter()
            .any(|item| globals.get(&item.name).copied() != type_from_wire(item.declared_type).ok())
    {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "global declarations do not match story",
        ));
    }
    let locals = local_types_for_scene(model, &save.current_scene);
    if save
        .locals
        .iter()
        .any(|item| locals.get(&item.name).copied() != type_from_wire(item.declared_type).ok())
    {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "local declaration does not match current scene",
        ));
    }
    validate_event_targets(
        save.current.as_ref().expect("contract requires current"),
        &scenes,
        false,
        limits,
        &save.current_scene,
    )?;
    for event in &save.pending {
        validate_event_targets(event, &scenes, true, limits, &save.current_scene)?;
    }
    for effect in &save.current_effects {
        validate_effect_path(effect, &save.current_scene)?;
    }
    for entry in &save.history {
        for effect in &entry.effects {
            validate_effect_path(effect, &save.current_scene)?;
        }
    }
    for path in save.background.iter().chain(save.bgm.iter()) {
        validate_media_path(path, &save.current_scene)?;
    }
    let status = status_from_wire(save.status)?;
    if status == SessionStatus::Active
        && matches!(
            save.current.as_ref().and_then(|event| event.kind.as_ref()),
            Some(w::semantic_event::Kind::End(_) | w::semantic_event::Kind::Error(_))
        )
    {
        return Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            &save.current_scene,
            "active status has terminal current event",
        ));
    }
    Ok(())
}

fn validate_event_targets(
    event: &w::SemanticEvent,
    scenes: &HashSet<&str>,
    pending: bool,
    _limits: PlayerLimits,
    scene: &str,
) -> Result<(), RuntimeError> {
    use w::semantic_event::Kind;
    match event.kind.as_ref() {
        Some(Kind::Scene(value)) if !scenes.contains(value.scene.as_str()) => Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            scene,
            "unknown scene transition",
        )),
        Some(Kind::Choices(value))
            if value
                .items
                .iter()
                .any(|choice| !scenes.contains(choice.target_scene.as_str())) =>
        {
            Err(save_error(
                "R_SAVE_STATE_CORRUPT",
                scene,
                "unknown choice target",
            ))
        }
        Some(Kind::Jump(target)) if !pending || !scenes.contains(target.as_str()) => {
            Err(save_error(
                "R_SAVE_STATE_CORRUPT",
                scene,
                "unknown or public jump event",
            ))
        }
        Some(Kind::Media(effect)) => validate_effect_path(effect, scene),
        Some(Kind::Dialogue(value)) => {
            if let Some(path) = &value.portrait_path {
                validate_media_path(path, scene)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_effect_path(effect: &w::MediaEffect, scene: &str) -> Result<(), RuntimeError> {
    use w::media_effect::Kind;
    match effect.kind.as_ref() {
        Some(Kind::BackgroundPath(path) | Kind::BgmPath(path) | Kind::SfxPath(path)) => {
            validate_media_path(path, scene)
        }
        _ => Ok(()),
    }
}

fn validate_media_path(path: &str, scene: &str) -> Result<(), RuntimeError> {
    if path.is_empty() || path.contains('\0') {
        Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            scene,
            "invalid media path",
        ))
    } else {
        Ok(())
    }
}

fn global_types(model: &StoryModel) -> HashMap<String, VarType> {
    model
        .init
        .variables
        .iter()
        .map(|item| (item.name.clone(), item.var_type))
        .collect()
}

fn local_types_for_scene(model: &StoryModel, scene: &str) -> HashMap<String, VarType> {
    let mut output = HashMap::new();
    if let Some(scene) = model.scenes.iter().find(|item| item.label == scene) {
        collect_local_types(&scene.prep, &mut output);
    }
    output
}

fn collect_local_types(statements: &[PrepStatement], output: &mut HashMap<String, VarType>) {
    for statement in statements {
        match statement {
            PrepStatement::VarDecl(value) => {
                output.insert(value.name.clone(), value.var_type);
            }
            PrepStatement::IfElse(value) => {
                collect_local_types(&value.then_branch, output);
                if let Some(branch) = &value.else_branch {
                    collect_local_types(branch, output);
                }
            }
            PrepStatement::ForSnapshot(value) => collect_local_types(&value.body, output),
            PrepStatement::Repeat(value) => collect_local_types(&value.body, output),
            _ => {}
        }
    }
}

fn variables_to_wire(
    values: &HashMap<String, Value>,
    types: &HashMap<String, VarType>,
) -> Result<Vec<w::Variable>, RuntimeError> {
    let mut names: Vec<_> = values.keys().collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let declared = types.get(name).copied().ok_or_else(|| {
                save_error("R_INVALID_STATE", "", "variable has no declared type")
            })?;
            Ok(w::Variable {
                name: name.clone(),
                declared_type: type_to_wire(declared) as i32,
                value: Some(value_to_wire(values.get(name).expect("key exists"))),
            })
        })
        .collect()
}

fn variables_from_wire(
    values: &[w::Variable],
    types: &HashMap<String, VarType>,
) -> Result<HashMap<String, Value>, RuntimeError> {
    values
        .iter()
        .map(|item| {
            let expected = types
                .get(&item.name)
                .copied()
                .ok_or_else(|| save_error("R_SAVE_STATE_CORRUPT", "", "unknown variable"))?;
            if type_from_wire(item.declared_type)? != expected {
                return Err(save_error(
                    "R_SAVE_STATE_CORRUPT",
                    "",
                    "variable type mismatch",
                ));
            }
            Ok((
                item.name.clone(),
                value_from_wire(
                    item.value.as_ref().expect("contract requires value"),
                    expected,
                )?,
            ))
        })
        .collect()
}

fn type_to_wire(value: VarType) -> w::ValueType {
    match value {
        VarType::Integer => w::ValueType::Integer,
        VarType::Decimal => w::ValueType::Decimal,
        VarType::Boolean => w::ValueType::Boolean,
        VarType::String => w::ValueType::String,
        VarType::ArrayInteger => w::ValueType::ArrayInteger,
        VarType::ArrayDecimal => w::ValueType::ArrayDecimal,
        VarType::ArrayBoolean => w::ValueType::ArrayBoolean,
        VarType::ArrayString => w::ValueType::ArrayString,
    }
}

fn type_from_wire(value: i32) -> Result<VarType, RuntimeError> {
    match w::ValueType::try_from(value) {
        Ok(w::ValueType::Integer) => Ok(VarType::Integer),
        Ok(w::ValueType::Decimal) => Ok(VarType::Decimal),
        Ok(w::ValueType::Boolean) => Ok(VarType::Boolean),
        Ok(w::ValueType::String) => Ok(VarType::String),
        Ok(w::ValueType::ArrayInteger) => Ok(VarType::ArrayInteger),
        Ok(w::ValueType::ArrayDecimal) => Ok(VarType::ArrayDecimal),
        Ok(w::ValueType::ArrayBoolean) => Ok(VarType::ArrayBoolean),
        Ok(w::ValueType::ArrayString) => Ok(VarType::ArrayString),
        _ => Err(save_error("R_SAVE_STATE_CORRUPT", "", "unknown value type")),
    }
}

fn value_to_wire(value: &Value) -> w::Value {
    use w::value::Kind;
    let kind = match value {
        Value::Int(value) => Kind::Integer(*value),
        Value::Decimal(value) => Kind::Decimal(w::DecimalValue {
            mantissa: value.mantissa().to_string(),
            scale: value.scale(),
        }),
        Value::Bool(value) => Kind::Boolean(*value),
        Value::Str(value) => Kind::Text(value.clone()),
        Value::Array {
            items,
            element_type,
        } => Kind::Array(w::ScalarArray {
            element_type: type_to_wire(*element_type) as i32,
            items: items.iter().map(value_to_wire).collect(),
        }),
    };
    w::Value { kind: Some(kind) }
}

fn value_from_wire(value: &w::Value, declared: VarType) -> Result<Value, RuntimeError> {
    use w::value::Kind;
    match value.kind.as_ref() {
        Some(Kind::Integer(value)) => Ok(Value::Int(*value)),
        Some(Kind::Boolean(value)) => Ok(Value::Bool(*value)),
        Some(Kind::Text(value)) => Ok(Value::Str(value.clone())),
        Some(Kind::Decimal(value)) => {
            let mantissa = value
                .mantissa
                .parse::<i128>()
                .map_err(|_| save_error("R_SAVE_STATE_CORRUPT", "", "invalid decimal"))?;
            Ok(Value::Decimal(Decimal::from_i128_with_scale(
                mantissa,
                value.scale,
            )))
        }
        Some(Kind::Array(value)) => {
            let element = match declared {
                VarType::ArrayInteger => VarType::Integer,
                VarType::ArrayDecimal => VarType::Decimal,
                VarType::ArrayBoolean => VarType::Boolean,
                VarType::ArrayString => VarType::String,
                _ => {
                    return Err(save_error(
                        "R_SAVE_STATE_CORRUPT",
                        "",
                        "array for scalar declaration",
                    ));
                }
            };
            Ok(Value::Array {
                items: value
                    .items
                    .iter()
                    .map(|item| value_from_wire(item, element))
                    .collect::<Result<_, _>>()?,
                element_type: element,
            })
        }
        None => Err(save_error("R_SAVE_STATE_CORRUPT", "", "missing value")),
    }
}

fn origin_to_wire(origin: &Origin) -> w::Origin {
    use w::origin::Kind;
    let kind = match origin {
        Origin::Source {
            parser_version,
            compiler_version,
            runtime_identity,
            semantic_sha256,
        } => Kind::Source(w::SourceOrigin {
            parser_version: parser_version.clone(),
            compiler_version: compiler_version.clone(),
            runtime_fingerprint_sha256: semantic_sha256.clone(),
            runtime_identity: runtime_identity.clone(),
        }),
        Origin::Bundle {
            format_version,
            compiler_version,
            schema_sha256,
            project_id,
            project_version,
            compiled_entry_sha256,
            signer,
            runtime_identity,
            semantic_sha256,
        } => Kind::Bundle(w::BundleOrigin {
            format_version: *format_version,
            compiler_version: compiler_version.clone(),
            compiled_schema_sha256: schema_sha256.clone(),
            project_id: project_id.clone(),
            project_version: project_version.clone(),
            compiled_entry_sha256: compiled_entry_sha256.clone(),
            signer: Some(match signer {
                BundleSigner::KeyId(id) => w::bundle_origin::Signer::SignerKeyId(id.clone()),
                BundleSigner::UnsignedDevelopment => {
                    w::bundle_origin::Signer::UnsignedDevelopment(true)
                }
            }),
            runtime_fingerprint_sha256: semantic_sha256.clone(),
            runtime_identity: runtime_identity.clone(),
        }),
    };
    w::Origin { kind: Some(kind) }
}

fn event_to_wire(event: &SemanticEvent, pending: bool) -> Result<w::SemanticEvent, RuntimeError> {
    use w::semantic_event::Kind;
    let kind = match event {
        SemanticEvent::SceneTransition(scene) => Kind::Scene(w::SceneTransition {
            scene: scene.clone(),
        }),
        SemanticEvent::Narration(value) => Kind::Narration(value.clone()),
        SemanticEvent::Dialogue {
            actor_id,
            actor_name,
            emotion,
            position,
            portrait_path,
            text,
        } => Kind::Dialogue(w::Dialogue {
            actor_id: actor_id.clone(),
            actor_name: actor_name.clone(),
            emotion: emotion.clone(),
            position: position.clone(),
            portrait_path: portrait_path.clone(),
            text: text.clone(),
        }),
        SemanticEvent::Choices(items) => Kind::Choices(w::Choices {
            items: items
                .iter()
                .map(|item| w::Choice {
                    text: item.text.clone(),
                    target_scene: item.target_scene.clone(),
                })
                .collect(),
        }),
        SemanticEvent::Media(value) => Kind::Media(effect_to_wire(value)),
        SemanticEvent::End => Kind::End(true),
        SemanticEvent::Error(value) => Kind::Error(error_to_wire(value)),
    };
    if !pending && matches!(kind, Kind::Jump(_)) {
        return Err(save_error(
            "R_INVALID_STATE",
            "",
            "public event cannot be jump",
        ));
    }
    Ok(w::SemanticEvent { kind: Some(kind) })
}

fn event_from_wire(event: &w::SemanticEvent, pending: bool) -> Result<SemanticEvent, RuntimeError> {
    use w::semantic_event::Kind;
    match event.kind.as_ref().expect("contract requires event kind") {
        Kind::Scene(value) => Ok(SemanticEvent::SceneTransition(value.scene.clone())),
        Kind::Narration(value) => Ok(SemanticEvent::Narration(value.clone())),
        Kind::Dialogue(value) => Ok(SemanticEvent::Dialogue {
            actor_id: value.actor_id.clone(),
            actor_name: value.actor_name.clone(),
            emotion: value.emotion.clone(),
            position: value.position.clone(),
            portrait_path: value.portrait_path.clone(),
            text: value.text.clone(),
        }),
        Kind::Choices(value) => Ok(SemanticEvent::Choices(
            value
                .items
                .iter()
                .map(|item| Choice {
                    text: item.text.clone(),
                    target_scene: item.target_scene.clone(),
                })
                .collect(),
        )),
        Kind::Media(value) => Ok(SemanticEvent::Media(effect_from_wire(value)?)),
        Kind::End(_) => Ok(SemanticEvent::End),
        Kind::Error(value) => Ok(SemanticEvent::Error(error_from_wire(value))),
        Kind::Jump(_) if pending => Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            "",
            "internal jump must be decoded as pending state",
        )),
        Kind::Jump(_) => Err(save_error("R_SAVE_STATE_CORRUPT", "", "public jump event")),
    }
}

fn pending_to_wire(event: &PendingEvent) -> Result<w::SemanticEvent, RuntimeError> {
    match event {
        PendingEvent::Jump(target) => Ok(w::SemanticEvent {
            kind: Some(w::semantic_event::Kind::Jump(target.clone())),
        }),
        PendingEvent::Event(event, effects) if effects.is_empty() => event_to_wire(event, true),
        PendingEvent::Event(_, _) => Err(save_error(
            "R_INVALID_STATE",
            "",
            "pending scene effects are not at a stable boundary",
        )),
    }
}

fn pending_from_wire(event: &w::SemanticEvent) -> Result<PendingEvent, RuntimeError> {
    match event.kind.as_ref() {
        Some(w::semantic_event::Kind::Jump(target)) => Ok(PendingEvent::Jump(target.clone())),
        _ => Ok(PendingEvent::Event(
            event_from_wire(event, true)?,
            Vec::new(),
        )),
    }
}

fn effect_to_wire(effect: &MediaEffect) -> w::MediaEffect {
    use w::media_effect::Kind;
    w::MediaEffect {
        kind: Some(match effect {
            MediaEffect::Background(path) => Kind::BackgroundPath(path.clone()),
            MediaEffect::Bgm(path) => Kind::BgmPath(path.clone()),
            MediaEffect::BgmStop => Kind::BgmStop(true),
            MediaEffect::Sfx(path) => Kind::SfxPath(path.clone()),
        }),
    }
}

fn effect_from_wire(effect: &w::MediaEffect) -> Result<MediaEffect, RuntimeError> {
    use w::media_effect::Kind;
    match effect.kind.as_ref().expect("contract requires effect kind") {
        Kind::BackgroundPath(path) => Ok(MediaEffect::Background(path.clone())),
        Kind::BgmPath(path) => Ok(MediaEffect::Bgm(path.clone())),
        Kind::BgmStop(_) => Ok(MediaEffect::BgmStop),
        Kind::SfxPath(path) => Ok(MediaEffect::Sfx(path.clone())),
    }
}

fn error_to_wire(error: &RuntimeError) -> w::RuntimeError {
    w::RuntimeError {
        code: error.code.clone(),
        scene: error.scene.clone(),
        message: error.message.clone(),
        resource: error.resource.clone(),
        actual: error.actual,
        limit: error.limit,
    }
}

fn error_from_wire(error: &w::RuntimeError) -> RuntimeError {
    RuntimeError {
        code: error.code.clone(),
        scene: error.scene.clone(),
        message: error.message.clone(),
        resource: error.resource.clone(),
        actual: error.actual,
        limit: error.limit,
    }
}

fn rng_to_wire(rng: RngState) -> w::SessionRng {
    w::SessionRng {
        algorithm_version: rng.algorithm_version,
        seed: rng.seed.to_vec(),
        stream: rng.stream,
        word_position_low: rng.word_position as u64,
        word_position_high: (rng.word_position >> 64) as u64,
    }
}

fn rng_from_wire(rng: &w::SessionRng) -> RngState {
    let mut seed = [0; 32];
    seed.copy_from_slice(&rng.seed);
    RngState {
        algorithm_version: rng.algorithm_version,
        seed,
        stream: rng.stream,
        word_position: u128::from(rng.word_position_low)
            | (u128::from(rng.word_position_high) << 64),
    }
}

fn history_to_wire(entry: &HistoryEntry) -> Result<w::HistoryEntry, RuntimeError> {
    Ok(w::HistoryEntry {
        sequence: entry.sequence,
        event: Some(event_to_wire(&entry.event, false)?),
        effects: entry.effects.iter().map(effect_to_wire).collect(),
        scene: entry.scene.clone(),
    })
}

fn history_from_wire(entry: &w::HistoryEntry) -> Result<HistoryEntry, RuntimeError> {
    Ok(HistoryEntry {
        sequence: entry.sequence,
        event: event_from_wire(
            entry
                .event
                .as_ref()
                .expect("contract requires history event"),
            false,
        )?,
        effects: entry
            .effects
            .iter()
            .map(effect_from_wire)
            .collect::<Result<_, _>>()?,
        scene: entry.scene.clone(),
    })
}

fn status_to_wire(status: SessionStatus) -> w::SessionStatus {
    match status {
        SessionStatus::Active => w::SessionStatus::Active,
        SessionStatus::Finished => w::SessionStatus::Finished,
        SessionStatus::Faulted => w::SessionStatus::Faulted,
    }
}

fn status_from_wire(status: i32) -> Result<SessionStatus, RuntimeError> {
    match w::SessionStatus::try_from(status) {
        Ok(w::SessionStatus::Active) => Ok(SessionStatus::Active),
        Ok(w::SessionStatus::Finished) => Ok(SessionStatus::Finished),
        Ok(w::SessionStatus::Faulted) => Ok(SessionStatus::Faulted),
        _ => Err(save_error(
            "R_SAVE_STATE_CORRUPT",
            "",
            "invalid session status",
        )),
    }
}

fn save_error(code: &str, scene: &str, message: &str) -> RuntimeError {
    RuntimeError {
        code: code.into(),
        scene: scene.into(),
        message: message.into(),
        resource: None,
        actual: None,
        limit: None,
    }
}
