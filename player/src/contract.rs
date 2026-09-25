//! V1 progress-save wire contract and UI-independent semantic playback contract.
//! The legacy `engine::StepResult` remains the active engine interface until the
//! shared runtime is introduced in the next checkpoint.
use prost::Message;
use sha2::{Digest, Sha256};

pub mod proto {
    pub mod storyplayer {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/storyplayer.v1.rs"));
        }
    }
}

pub const SAVE_SCHEMA_VERSION: u32 = 1;
pub const SAVE_RUNTIME_VERSION: u32 = 1;
pub const SAVE_DESCRIPTOR: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/storyplayer_descriptor.bin"));
pub const SAVE_SCHEMA_SHA256: &str = include_str!("../proto/storyplayer/v1/schema.sha256");

pub fn descriptor_sha256() -> String {
    format!("{:x}", Sha256::digest(SAVE_DESCRIPTOR))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Source {
        parser_version: String,
        compiler_version: String,
        runtime_identity: String,
        semantic_sha256: String,
    },
    Bundle {
        format_version: u32,
        compiler_version: String,
        schema_sha256: String,
        project_id: String,
        project_version: String,
        compiled_entry_sha256: String,
        signer: BundleSigner,
        runtime_identity: String,
        semantic_sha256: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleSigner {
    KeyId(String),
    UnsignedDevelopment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticEvent {
    SceneTransition(String),
    Narration(String),
    Dialogue {
        actor_id: String,
        actor_name: String,
        emotion: Option<String>,
        position: Option<String>,
        portrait_path: Option<String>,
        text: String,
    },
    Choices(Vec<Choice>),
    Media(MediaEffect),
    End,
    Error(RuntimeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub text: String,
    pub target_scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaEffect {
    Background(String),
    Bgm(String),
    BgmStop,
    Sfx(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: String,
    pub scene: String,
    pub message: String,
    pub resource: Option<String>,
    pub actual: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Finished,
    Faulted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDelta {
    pub current: SemanticEvent,
    pub effects: Vec<MediaEffect>,
    pub scene: String,
    pub status: SessionStatus,
    pub sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub sequence: u64,
    pub event: SemanticEvent,
    pub effects: Vec<MediaEffect>,
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub next_sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

pub const HARD_LIMITS: PlayerLimits = PlayerLimits {
    operations_per_interaction: 100_000,
    logic_depth: 128,
    pending_events_per_scene: 16_384,
    array_elements: 16_384,
    rendered_bytes: 1024 * 1024,
    history_entries: 10_000,
    history_bytes: 8 * 1024 * 1024,
    save_bytes: 16 * 1024 * 1024,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerLimits {
    pub operations_per_interaction: usize,
    pub logic_depth: usize,
    pub pending_events_per_scene: usize,
    pub array_elements: usize,
    pub rendered_bytes: usize,
    pub history_entries: usize,
    pub history_bytes: usize,
    pub save_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitError {
    pub resource: &'static str,
    pub requested: usize,
    pub hard_maximum: usize,
}

impl PlayerLimits {
    pub fn lowered(self) -> Result<Self, LimitError> {
        for (resource, requested, hard_maximum) in [
            (
                "operations_per_interaction",
                self.operations_per_interaction,
                HARD_LIMITS.operations_per_interaction,
            ),
            ("logic_depth", self.logic_depth, HARD_LIMITS.logic_depth),
            (
                "pending_events_per_scene",
                self.pending_events_per_scene,
                HARD_LIMITS.pending_events_per_scene,
            ),
            (
                "array_elements",
                self.array_elements,
                HARD_LIMITS.array_elements,
            ),
            (
                "rendered_bytes",
                self.rendered_bytes,
                HARD_LIMITS.rendered_bytes,
            ),
            (
                "history_entries",
                self.history_entries,
                HARD_LIMITS.history_entries,
            ),
            (
                "history_bytes",
                self.history_bytes,
                HARD_LIMITS.history_bytes,
            ),
            ("save_bytes", self.save_bytes, HARD_LIMITS.save_bytes),
        ] {
            if requested == 0 || requested > hard_maximum {
                return Err(LimitError {
                    resource,
                    requested,
                    hard_maximum,
                });
            }
        }
        Ok(self)
    }
}

/// Wire-level guard for progress saves. Story-dependent checks (fingerprint,
/// declared variables, known scenes/paths) are performed by the restore codec.
/// Requiring canonical prost encoding prevents proto3 from silently discarding
/// unknown *required* oneof alternatives, unknown fields or repeated singulars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveContractError {
    pub code: &'static str,
    pub message: &'static str,
}

fn corrupt(message: &'static str) -> SaveContractError {
    SaveContractError {
        code: "R_SAVE_STATE_CORRUPT",
        message,
    }
}

pub fn decode_save_contract(
    bytes: &[u8],
    limits: PlayerLimits,
) -> Result<proto::storyplayer::v1::PlayerSave, SaveContractError> {
    use proto::storyplayer::v1 as w;
    let limits = limits
        .lowered()
        .map_err(|_| corrupt("invalid resource limits"))?;
    if bytes.len() > limits.save_bytes {
        return Err(corrupt("save exceeds byte limit"));
    }
    let save = w::PlayerSave::decode(bytes).map_err(|_| corrupt("invalid protobuf"))?;
    if save.schema_version != SAVE_SCHEMA_VERSION || save.runtime_version != SAVE_RUNTIME_VERSION {
        return Err(SaveContractError {
            code: "R_SAVE_INCOMPATIBLE",
            message: "unsupported save schema or runtime version",
        });
    }
    if save.encode_to_vec() != bytes {
        return Err(corrupt("noncanonical or unknown protobuf fields"));
    }
    let origin = save
        .origin
        .as_ref()
        .and_then(|o| o.kind.as_ref())
        .ok_or_else(|| corrupt("missing origin"))?;
    match origin {
        w::origin::Kind::Source(o)
            if !o.parser_version.is_empty()
                && !o.compiler_version.is_empty()
                && !o.runtime_identity.is_empty()
                && valid_sha256(&o.runtime_fingerprint_sha256) => {}
        w::origin::Kind::Bundle(o)
            if o.format_version != 0
                && !o.compiler_version.is_empty()
                && valid_sha256(&o.compiled_schema_sha256)
                && !o.project_id.is_empty()
                && !o.project_version.is_empty()
                && valid_sha256(&o.compiled_entry_sha256)
                && valid_sha256(&o.runtime_fingerprint_sha256)
                && !o.runtime_identity.is_empty()
                && (matches!(o.signer.as_ref(), Some(w::bundle_origin::Signer::SignerKeyId(id)) if valid_sha256(id))
                    || matches!(
                        o.signer,
                        Some(w::bundle_origin::Signer::UnsignedDevelopment(true))
                    )) => {}
        _ => return Err(corrupt("invalid origin")),
    }
    if save.current_scene.is_empty()
        || !matches!(
            w::SessionStatus::try_from(save.status),
            Ok(w::SessionStatus::Active | w::SessionStatus::Finished | w::SessionStatus::Faulted)
        )
    {
        return Err(corrupt("invalid scene or status"));
    }
    let rng = save.rng.as_ref().ok_or_else(|| corrupt("missing rng"))?;
    if rng.algorithm_version != 1
        || rng.seed.len() != 32
        || (u128::from(rng.word_position_high) << 64 | u128::from(rng.word_position_low))
            >= crate::session_rng::WORD_POSITION_END
    {
        return Err(corrupt("invalid rng state"));
    }
    for vars in [&save.globals, &save.locals] {
        let mut previous = None;
        for var in vars {
            if var.name.is_empty() || previous.is_some_and(|name: &str| name >= var.name.as_str()) {
                return Err(corrupt("duplicate or unordered variable names"));
            }
            previous = Some(var.name.as_str());
            let ty = w::ValueType::try_from(var.declared_type)
                .map_err(|_| corrupt("unknown variable type"))?;
            if ty == w::ValueType::Unspecified {
                return Err(corrupt("unspecified variable type"));
            }
            validate_value(
                var.value
                    .as_ref()
                    .ok_or_else(|| corrupt("missing variable value"))?,
                ty,
                limits,
            )?;
        }
    }
    if save.pending.len() > limits.pending_events_per_scene
        || save.history.len() > limits.history_entries
    {
        return Err(corrupt("too many pending or history events"));
    }
    let current = save
        .current
        .as_ref()
        .ok_or_else(|| corrupt("missing current event"))?;
    validate_event(current, false, limits)?;
    for event in &save.pending {
        validate_event(event, true, limits)?;
    }
    for effect in &save.current_effects {
        validate_effect(effect, limits)?;
    }
    let mut previous = None;
    let mut history_bytes = 0usize;
    for entry in &save.history {
        if previous.is_some_and(|n| n >= entry.sequence)
            || entry.sequence >= save.next_sequence
            || entry.scene.is_empty()
        {
            return Err(corrupt("invalid history sequence or scene"));
        }
        previous = Some(entry.sequence);
        validate_event(
            entry
                .event
                .as_ref()
                .ok_or_else(|| corrupt("missing history event"))?,
            false,
            limits,
        )?;
        for effect in &entry.effects {
            validate_effect(effect, limits)?;
        }
        history_bytes = history_bytes
            .checked_add(entry.encoded_len())
            .ok_or_else(|| corrupt("history size overflow"))?;
    }
    if history_bytes > limits.history_bytes
        || save.first_retained_sequence > save.next_sequence
        || save.omitted_history_count > save.first_retained_sequence
    {
        return Err(corrupt("invalid history bounds"));
    }
    if let Some(first) = save.history.first() {
        if first.sequence != save.first_retained_sequence {
            return Err(corrupt("invalid first retained sequence"));
        }
    }
    if matches!(
        w::SessionStatus::try_from(save.status),
        Ok(w::SessionStatus::Finished)
    ) && !matches!(current.kind, Some(w::semantic_event::Kind::End(true)))
    {
        return Err(corrupt("finished without end event"));
    }
    if matches!(
        w::SessionStatus::try_from(save.status),
        Ok(w::SessionStatus::Faulted)
    ) && !matches!(current.kind, Some(w::semantic_event::Kind::Error(_)))
    {
        return Err(corrupt("faulted without error event"));
    }
    Ok(save)
}

fn valid_sha256(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_text(text: &str, limits: PlayerLimits) -> Result<(), SaveContractError> {
    if text.len() > limits.rendered_bytes {
        Err(corrupt("rendered text exceeds limit"))
    } else {
        Ok(())
    }
}

fn validate_value(
    value: &proto::storyplayer::v1::Value,
    ty: proto::storyplayer::v1::ValueType,
    limits: PlayerLimits,
) -> Result<(), SaveContractError> {
    use proto::storyplayer::v1::{ValueType as T, value::Kind as K};
    match (ty, value.kind.as_ref()) {
        (T::Integer, Some(K::Integer(_))) | (T::Boolean, Some(K::Boolean(_))) => Ok(()),
        (T::String, Some(K::Text(s))) => valid_text(s, limits),
        (T::Decimal, Some(K::Decimal(d))) if d.scale <= 28 && canonical_mantissa(&d.mantissa) => {
            Ok(())
        }
        (
            T::ArrayInteger | T::ArrayDecimal | T::ArrayBoolean | T::ArrayString,
            Some(K::Array(a)),
        ) => {
            let element = match ty {
                T::ArrayInteger => T::Integer,
                T::ArrayDecimal => T::Decimal,
                T::ArrayBoolean => T::Boolean,
                _ => T::String,
            };
            if a.element_type != element as i32 || a.items.len() > limits.array_elements {
                return Err(corrupt("invalid array type or size"));
            }
            for item in &a.items {
                validate_value(item, element, limits)?;
            }
            Ok(())
        }
        _ => Err(corrupt("value type mismatch or missing value")),
    }
}

fn canonical_mantissa(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty()
        && (digits == "0"
            || (!digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit())))
        && text != "-0"
        && text
            .parse::<i128>()
            .ok()
            .and_then(i128::checked_abs)
            .is_some_and(|n| n <= (1i128 << 96) - 1)
}

fn validate_event(
    event: &proto::storyplayer::v1::SemanticEvent,
    pending: bool,
    limits: PlayerLimits,
) -> Result<(), SaveContractError> {
    use proto::storyplayer::v1::semantic_event::Kind as K;
    if event.encoded_len() > limits.rendered_bytes {
        return Err(corrupt("event exceeds rendered byte limit"));
    }
    match event
        .kind
        .as_ref()
        .ok_or_else(|| corrupt("missing event variant"))?
    {
        K::Scene(scene) if !scene.scene.is_empty() => Ok(()),
        K::Narration(s) => valid_text(s, limits),
        K::Dialogue(d) if !d.actor_id.is_empty() => {
            valid_text(&d.actor_name, limits)?;
            valid_text(&d.text, limits)?;
            if let Some(path) = &d.portrait_path {
                valid_text(path, limits)?;
            }
            Ok(())
        }
        K::Choices(c)
            if !c.items.is_empty() && c.items.len() <= limits.pending_events_per_scene =>
        {
            for choice in &c.items {
                if choice.target_scene.is_empty() {
                    return Err(corrupt("empty choice target"));
                }
                valid_text(&choice.text, limits)?;
            }
            Ok(())
        }
        K::Media(m) => validate_effect(m, limits),
        K::End(true) => Ok(()),
        K::Error(e) if !e.code.is_empty() && !e.scene.is_empty() => valid_text(&e.message, limits),
        K::Jump(target) if pending && !target.is_empty() => Ok(()),
        _ => Err(corrupt("invalid event variant")),
    }
}

fn validate_effect(
    effect: &proto::storyplayer::v1::MediaEffect,
    limits: PlayerLimits,
) -> Result<(), SaveContractError> {
    use proto::storyplayer::v1::media_effect::Kind as K;
    match effect
        .kind
        .as_ref()
        .ok_or_else(|| corrupt("missing media effect"))?
    {
        K::BackgroundPath(path) | K::BgmPath(path) | K::SfxPath(path) if !path.is_empty() => {
            valid_text(path, limits)
        }
        K::BgmStop(true) => Ok(()),
        _ => Err(corrupt("invalid media effect")),
    }
}
