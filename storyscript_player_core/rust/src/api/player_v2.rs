use std::path::PathBuf;
use std::sync::{Arc, Mutex, TryLockError};

use storyscript_player::contract::{
    EventDelta, HistoryEntry, HistoryPage, MediaEffect, PlayerLimits, RuntimeError, SemanticEvent,
    SessionStatus, HARD_LIMITS,
};
use storyscript_player::SemanticPlayer;

#[derive(Debug, Clone, Copy)]
pub struct BridgePlayerLimits {
    pub operations_per_interaction: u64,
    pub logic_depth: u64,
    pub pending_events_per_scene: u64,
    pub array_elements: u64,
    pub rendered_bytes: u64,
    pub history_entries: u64,
    pub history_bytes: u64,
    pub save_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerError {
    pub code: String,
    pub scene: String,
    pub message: String,
    pub resource: Option<String>,
    pub actual: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSemanticChoice {
    pub text: String,
    pub target_scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSemanticEvent {
    pub kind: String,
    pub text: Option<String>,
    pub scene: Option<String>,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub emotion: Option<String>,
    pub position: Option<String>,
    pub portrait_path: Option<String>,
    pub choices: Vec<BridgeSemanticChoice>,
    pub error: Option<BridgePlayerError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMediaEffect {
    pub kind: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerDelta {
    pub event: BridgeSemanticEvent,
    pub effects: Vec<BridgeMediaEffect>,
    pub scene: String,
    pub status: String,
    pub sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoryEntry {
    pub sequence: u64,
    pub event: BridgeSemanticEvent,
    pub effects: Vec<BridgeMediaEffect>,
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoryPage {
    pub entries: Vec<BridgeHistoryEntry>,
    pub next_sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

#[derive(Clone)]
pub struct SourcePlayerResource {
    player: Arc<Mutex<Option<SemanticPlayer>>>,
}

pub struct BridgeSourcePlayerOpened {
    pub resource: SourcePlayerResource,
    pub current: BridgePlayerDelta,
}

pub struct BridgeSourcePlayerOpenResult {
    pub opened: Option<BridgeSourcePlayerOpened>,
    pub error: Option<BridgePlayerError>,
}

pub struct BridgePlayerActionResult {
    pub delta: Option<BridgePlayerDelta>,
    pub error: Option<BridgePlayerError>,
}

pub struct BridgePlayerHistoryResult {
    pub page: Option<BridgeHistoryPage>,
    pub error: Option<BridgePlayerError>,
}

pub struct BridgePlayerSaveResult {
    pub bytes: Option<Vec<u8>>,
    pub error: Option<BridgePlayerError>,
}

pub struct BridgePlayerDisposeResult {
    pub released: bool,
    pub error: Option<BridgePlayerError>,
}

pub fn source_player_hard_limits() -> BridgePlayerLimits {
    HARD_LIMITS.into()
}

pub fn source_player_open_raw(
    source: String,
    limits: BridgePlayerLimits,
) -> BridgeSourcePlayerOpenResult {
    let limits = match PlayerLimits::try_from(limits) {
        Ok(value) => value,
        Err(error) => return BridgeSourcePlayerOpenResult::failure(error),
    };
    match SemanticPlayer::from_source(&source, limits) {
        Ok(player) => BridgeSourcePlayerOpenResult::success(player),
        Err(message) => BridgeSourcePlayerOpenResult::failure(BridgePlayerError::simple(
            "R_SOURCE_COMPILE",
            message,
        )),
    }
}

pub fn source_player_open_path(
    path: String,
    limits: BridgePlayerLimits,
) -> BridgeSourcePlayerOpenResult {
    let limits = match PlayerLimits::try_from(limits) {
        Ok(value) => value,
        Err(error) => return BridgeSourcePlayerOpenResult::failure(error),
    };
    match SemanticPlayer::from_file(&PathBuf::from(path), limits) {
        Ok(player) => BridgeSourcePlayerOpenResult::success(player),
        Err(message) => BridgeSourcePlayerOpenResult::failure(BridgePlayerError::simple(
            "R_SOURCE_COMPILE",
            message,
        )),
    }
}

pub fn source_player_restore_raw(
    source: String,
    save: Vec<u8>,
    limits: BridgePlayerLimits,
) -> BridgeSourcePlayerOpenResult {
    let limits = match PlayerLimits::try_from(limits) {
        Ok(value) => value,
        Err(error) => return BridgeSourcePlayerOpenResult::failure(error),
    };
    match SemanticPlayer::restore_source(&source, &save, limits) {
        Ok(player) => BridgeSourcePlayerOpenResult::success(player),
        Err(error) => BridgeSourcePlayerOpenResult::failure(error.into()),
    }
}

pub fn source_player_restore_path(
    path: String,
    save: Vec<u8>,
    limits: BridgePlayerLimits,
) -> BridgeSourcePlayerOpenResult {
    let limits = match PlayerLimits::try_from(limits) {
        Ok(value) => value,
        Err(error) => return BridgeSourcePlayerOpenResult::failure(error),
    };
    match SemanticPlayer::restore_file(&PathBuf::from(path), &save, limits) {
        Ok(player) => BridgeSourcePlayerOpenResult::success(player),
        Err(error) => BridgeSourcePlayerOpenResult::failure(error.into()),
    }
}

pub fn source_player_current(resource: &SourcePlayerResource) -> BridgePlayerActionResult {
    with_player(resource, |player| Ok(player.current().clone()))
}

pub fn source_player_advance(resource: &SourcePlayerResource) -> BridgePlayerActionResult {
    with_player(resource, |player| player.advance().cloned())
}

pub fn source_player_choose(
    resource: &SourcePlayerResource,
    index: u32,
) -> BridgePlayerActionResult {
    with_player(resource, |player| player.choose(index as usize).cloned())
}

pub fn source_player_history(
    resource: &SourcePlayerResource,
    start_sequence: u64,
    maximum: u32,
) -> BridgePlayerHistoryResult {
    match lock_player(resource) {
        Ok(guard) => BridgePlayerHistoryResult {
            page: Some(history_page_to_bridge(
                guard
                    .as_ref()
                    .expect("checked")
                    .history_page(start_sequence, maximum as usize),
            )),
            error: None,
        },
        Err(error) => BridgePlayerHistoryResult {
            page: None,
            error: Some(error),
        },
    }
}

pub fn source_player_export_save(resource: &SourcePlayerResource) -> BridgePlayerSaveResult {
    match lock_player(resource) {
        Ok(guard) => match guard.as_ref().expect("checked").export_save() {
            Ok(bytes) => BridgePlayerSaveResult {
                bytes: Some(bytes),
                error: None,
            },
            Err(error) => BridgePlayerSaveResult {
                bytes: None,
                error: Some(error.into()),
            },
        },
        Err(error) => BridgePlayerSaveResult {
            bytes: None,
            error: Some(error),
        },
    }
}

pub fn source_player_dispose(resource: &SourcePlayerResource) -> BridgePlayerDisposeResult {
    match resource.player.try_lock() {
        Ok(mut guard) => BridgePlayerDisposeResult {
            released: guard.take().is_some(),
            error: None,
        },
        Err(TryLockError::WouldBlock) => BridgePlayerDisposeResult {
            released: false,
            error: Some(BridgePlayerError::simple(
                "R_PLAYER_BUSY",
                "player operation is already in progress",
            )),
        },
        Err(TryLockError::Poisoned(_)) => BridgePlayerDisposeResult {
            released: false,
            error: Some(BridgePlayerError::simple(
                "R_RESOURCE_STATE",
                "player resource lock is poisoned",
            )),
        },
    }
}

fn with_player<F>(resource: &SourcePlayerResource, operation: F) -> BridgePlayerActionResult
where
    F: FnOnce(&mut SemanticPlayer) -> Result<EventDelta, RuntimeError>,
{
    let mut guard = match lock_player(resource) {
        Ok(guard) => guard,
        Err(error) => {
            return BridgePlayerActionResult {
                delta: None,
                error: Some(error),
            };
        }
    };
    match operation(guard.as_mut().expect("checked")) {
        Ok(delta) => BridgePlayerActionResult {
            delta: Some(delta_to_bridge(delta)),
            error: None,
        },
        Err(error) => BridgePlayerActionResult {
            delta: None,
            error: Some(error.into()),
        },
    }
}

fn lock_player(
    resource: &SourcePlayerResource,
) -> Result<std::sync::MutexGuard<'_, Option<SemanticPlayer>>, BridgePlayerError> {
    match resource.player.try_lock() {
        Ok(guard) if guard.is_some() => Ok(guard),
        Ok(_) => Err(BridgePlayerError::simple(
            "R_PLAYER_DISPOSED",
            "player resource has been disposed",
        )),
        Err(TryLockError::WouldBlock) => Err(BridgePlayerError::simple(
            "R_PLAYER_BUSY",
            "player operation is already in progress",
        )),
        Err(TryLockError::Poisoned(_)) => Err(BridgePlayerError::simple(
            "R_RESOURCE_STATE",
            "player resource lock is poisoned",
        )),
    }
}

impl BridgeSourcePlayerOpenResult {
    fn success(player: SemanticPlayer) -> Self {
        let current = delta_to_bridge(player.current().clone());
        Self {
            opened: Some(BridgeSourcePlayerOpened {
                resource: SourcePlayerResource {
                    player: Arc::new(Mutex::new(Some(player))),
                },
                current,
            }),
            error: None,
        }
    }
    fn failure(error: BridgePlayerError) -> Self {
        Self {
            opened: None,
            error: Some(error),
        }
    }
}

impl BridgePlayerError {
    fn simple(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            scene: String::new(),
            message: message.into(),
            resource: None,
            actual: None,
            limit: None,
        }
    }
}

impl From<RuntimeError> for BridgePlayerError {
    fn from(error: RuntimeError) -> Self {
        Self {
            code: error.code,
            scene: error.scene,
            message: error.message,
            resource: error.resource,
            actual: error.actual,
            limit: error.limit,
        }
    }
}

impl From<PlayerLimits> for BridgePlayerLimits {
    fn from(value: PlayerLimits) -> Self {
        Self {
            operations_per_interaction: value.operations_per_interaction as u64,
            logic_depth: value.logic_depth as u64,
            pending_events_per_scene: value.pending_events_per_scene as u64,
            array_elements: value.array_elements as u64,
            rendered_bytes: value.rendered_bytes as u64,
            history_entries: value.history_entries as u64,
            history_bytes: value.history_bytes as u64,
            save_bytes: value.save_bytes as u64,
        }
    }
}

impl TryFrom<BridgePlayerLimits> for PlayerLimits {
    type Error = BridgePlayerError;
    fn try_from(value: BridgePlayerLimits) -> Result<Self, Self::Error> {
        let convert = |value: u64| {
            usize::try_from(value).map_err(|_| {
                BridgePlayerError::simple(
                    "R_LIMIT_CONFIGURATION",
                    "limit cannot be represented on this platform",
                )
            })
        };
        PlayerLimits {
            operations_per_interaction: convert(value.operations_per_interaction)?,
            logic_depth: convert(value.logic_depth)?,
            pending_events_per_scene: convert(value.pending_events_per_scene)?,
            array_elements: convert(value.array_elements)?,
            rendered_bytes: convert(value.rendered_bytes)?,
            history_entries: convert(value.history_entries)?,
            history_bytes: convert(value.history_bytes)?,
            save_bytes: convert(value.save_bytes)?,
        }
        .lowered()
        .map_err(|invalid| BridgePlayerError {
            code: "R_LIMIT_CONFIGURATION".into(),
            scene: String::new(),
            message: format!("invalid limit: {}", invalid.resource),
            resource: Some(invalid.resource.into()),
            actual: Some(invalid.requested as u64),
            limit: Some(invalid.hard_maximum as u64),
        })
    }
}

fn delta_to_bridge(value: EventDelta) -> BridgePlayerDelta {
    BridgePlayerDelta {
        event: event_to_bridge(value.current),
        effects: value.effects.into_iter().map(effect_to_bridge).collect(),
        scene: value.scene,
        status: status_name(value.status).into(),
        sequence: value.sequence,
        first_retained_sequence: value.first_retained_sequence,
        omitted_history_count: value.omitted_history_count,
    }
}

fn history_page_to_bridge(value: HistoryPage) -> BridgeHistoryPage {
    BridgeHistoryPage {
        entries: value
            .entries
            .into_iter()
            .map(history_entry_to_bridge)
            .collect(),
        next_sequence: value.next_sequence,
        first_retained_sequence: value.first_retained_sequence,
        omitted_history_count: value.omitted_history_count,
    }
}

fn history_entry_to_bridge(value: HistoryEntry) -> BridgeHistoryEntry {
    BridgeHistoryEntry {
        sequence: value.sequence,
        event: event_to_bridge(value.event),
        effects: value.effects.into_iter().map(effect_to_bridge).collect(),
        scene: value.scene,
    }
}

fn event_to_bridge(value: SemanticEvent) -> BridgeSemanticEvent {
    let mut output = BridgeSemanticEvent {
        kind: String::new(),
        text: None,
        scene: None,
        actor_id: None,
        actor_name: None,
        emotion: None,
        position: None,
        portrait_path: None,
        choices: Vec::new(),
        error: None,
    };
    match value {
        SemanticEvent::SceneTransition(scene) => {
            output.kind = "scene".into();
            output.scene = Some(scene);
        }
        SemanticEvent::Narration(text) => {
            output.kind = "narration".into();
            output.text = Some(text);
        }
        SemanticEvent::Dialogue {
            actor_id,
            actor_name,
            emotion,
            position,
            portrait_path,
            text,
        } => {
            output.kind = "dialogue".into();
            output.actor_id = Some(actor_id);
            output.actor_name = Some(actor_name);
            output.emotion = emotion;
            output.position = position;
            output.portrait_path = portrait_path;
            output.text = Some(text);
        }
        SemanticEvent::Choices(items) => {
            output.kind = "choices".into();
            output.choices = items
                .into_iter()
                .map(|item| BridgeSemanticChoice {
                    text: item.text,
                    target_scene: item.target_scene,
                })
                .collect();
        }
        SemanticEvent::Media(effect) => {
            output.kind = "media".into();
            output.text = effect_to_bridge(effect).path;
        }
        SemanticEvent::End => output.kind = "end".into(),
        SemanticEvent::Error(error) => {
            output.kind = "error".into();
            output.error = Some(error.into());
        }
    }
    output
}

fn effect_to_bridge(value: MediaEffect) -> BridgeMediaEffect {
    match value {
        MediaEffect::Background(path) => BridgeMediaEffect {
            kind: "background".into(),
            path: Some(path),
        },
        MediaEffect::Bgm(path) => BridgeMediaEffect {
            kind: "bgm".into(),
            path: Some(path),
        },
        MediaEffect::BgmStop => BridgeMediaEffect {
            kind: "bgm_stop".into(),
            path: None,
        },
        MediaEffect::Sfx(path) => BridgeMediaEffect {
            kind: "sfx".into(),
            path: Some(path),
        },
    }
}

fn status_name(value: SessionStatus) -> &'static str {
    match value {
        SessionStatus::Active => "active",
        SessionStatus::Finished => "finished",
        SessionStatus::Faulted => "faulted",
    }
}
