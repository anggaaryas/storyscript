use std::sync::{Arc, Mutex, TryLockError};

use storyscript_bundle_core::loader::LoadedBundle;
use storyscript_player::contract::{
    EventDelta, HistoryEntry, HistoryPage, MediaEffect, PlayerLimits, RuntimeError, SemanticEvent,
    SessionStatus, HARD_LIMITS,
};
use storyscript_player::SemanticPlayer;

use super::bundle::{
    bundle_open_bytes, bundle_open_path, BridgeError, BridgeLimits, BridgeTrustKey,
    BridgeVerificationPolicy, BundleResource,
};

#[derive(Debug, Clone, Copy)]
pub struct BridgeRuntimeLimits {
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
pub struct BridgeRuntimeError {
    pub code: String,
    pub scene: String,
    pub message: String,
    pub resource: Option<String>,
    pub actual: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerChoice {
    pub text: String,
    pub target_scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerEvent {
    pub kind: String,
    pub text: Option<String>,
    pub scene: Option<String>,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub emotion: Option<String>,
    pub position: Option<String>,
    pub portrait_path: Option<String>,
    pub choices: Vec<BridgePlayerChoice>,
    pub error: Option<BridgeRuntimeError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerEffect {
    pub kind: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerDelta {
    pub event: BridgePlayerEvent,
    pub effects: Vec<BridgePlayerEffect>,
    pub scene: String,
    pub status: String,
    pub sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerHistoryEntry {
    pub sequence: u64,
    pub event: BridgePlayerEvent,
    pub effects: Vec<BridgePlayerEffect>,
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlayerHistoryPage {
    pub entries: Vec<BridgePlayerHistoryEntry>,
    pub next_sequence: u64,
    pub first_retained_sequence: u64,
    pub omitted_history_count: u64,
}

struct PlayerLease {
    player: SemanticPlayer,
    bundle: Arc<LoadedBundle>,
}

#[derive(Clone)]
pub struct BundlePlayerResource {
    lease: Arc<Mutex<Option<PlayerLease>>>,
}

pub struct BridgeBundlePlayerOpened {
    pub resource: BundlePlayerResource,
    pub current: BridgePlayerDelta,
}
pub struct BridgeBundlePlayerOpenResult {
    pub opened: Option<BridgeBundlePlayerOpened>,
    pub error: Option<BridgeRuntimeError>,
}
pub struct BridgeBundlePlayerActionResult {
    pub delta: Option<BridgePlayerDelta>,
    pub error: Option<BridgeRuntimeError>,
}
pub struct BridgeBundlePlayerHistoryResult {
    pub page: Option<BridgePlayerHistoryPage>,
    pub error: Option<BridgeRuntimeError>,
}
pub struct BridgeBundlePlayerSaveResult {
    pub bytes: Option<Vec<u8>>,
    pub error: Option<BridgeRuntimeError>,
}
pub struct BridgeBundlePlayerAssetResult {
    pub bytes: Option<Vec<u8>>,
    pub error: Option<BridgeRuntimeError>,
}
pub struct BridgeBundlePlayerDisposeResult {
    pub released: bool,
    pub error: Option<BridgeRuntimeError>,
}

pub fn bundle_player_hard_limits() -> BridgeRuntimeLimits {
    HARD_LIMITS.into()
}

pub fn bundle_player_open_from_bundle(
    bundle: &BundleResource,
    limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let bundle = match bundle.lease() {
        Ok(value) => value,
        Err(error) => return BridgeBundlePlayerOpenResult::failure(error.into()),
    };
    open_bundle(bundle, None, limits)
}

pub fn bundle_player_restore_from_bundle(
    bundle: &BundleResource,
    save: Vec<u8>,
    limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let bundle = match bundle.lease() {
        Ok(value) => value,
        Err(error) => return BridgeBundlePlayerOpenResult::failure(error.into()),
    };
    open_bundle(bundle, Some(save), limits)
}

pub fn bundle_player_open_bytes(
    bytes: Vec<u8>,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    bundle_limits: BridgeLimits,
    player_limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let result = bundle_open_bytes(bytes, trust_keys, policy, bundle_limits);
    let Some(opened) = result.opened else {
        return BridgeBundlePlayerOpenResult::failure(
            result
                .error
                .unwrap_or_else(|| {
                    BridgeError::new("B_BRIDGE_CONTRACT", "bundle open returned no result")
                })
                .into(),
        );
    };
    match opened.resource.lease() {
        Ok(bundle) => open_bundle(bundle, None, player_limits),
        Err(error) => BridgeBundlePlayerOpenResult::failure(error.into()),
    }
}

pub fn bundle_player_restore_bytes(
    bytes: Vec<u8>,
    save: Vec<u8>,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    bundle_limits: BridgeLimits,
    player_limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let result = bundle_open_bytes(bytes, trust_keys, policy, bundle_limits);
    let Some(opened) = result.opened else {
        return BridgeBundlePlayerOpenResult::failure(
            result
                .error
                .unwrap_or_else(|| {
                    BridgeError::new("B_BRIDGE_CONTRACT", "bundle open returned no result")
                })
                .into(),
        );
    };
    match opened.resource.lease() {
        Ok(bundle) => open_bundle(bundle, Some(save), player_limits),
        Err(error) => BridgeBundlePlayerOpenResult::failure(error.into()),
    }
}

pub fn bundle_player_open_path(
    path: String,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    bundle_limits: BridgeLimits,
    player_limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let result = bundle_open_path(path, trust_keys, policy, bundle_limits);
    let Some(opened) = result.opened else {
        return BridgeBundlePlayerOpenResult::failure(
            result
                .error
                .unwrap_or_else(|| {
                    BridgeError::new("B_BRIDGE_CONTRACT", "bundle open returned no result")
                })
                .into(),
        );
    };
    match opened.resource.lease() {
        Ok(bundle) => open_bundle(bundle, None, player_limits),
        Err(error) => BridgeBundlePlayerOpenResult::failure(error.into()),
    }
}

pub fn bundle_player_restore_path(
    path: String,
    save: Vec<u8>,
    trust_keys: Vec<BridgeTrustKey>,
    policy: BridgeVerificationPolicy,
    bundle_limits: BridgeLimits,
    player_limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let result = bundle_open_path(path, trust_keys, policy, bundle_limits);
    let Some(opened) = result.opened else {
        return BridgeBundlePlayerOpenResult::failure(
            result
                .error
                .unwrap_or_else(|| {
                    BridgeError::new("B_BRIDGE_CONTRACT", "bundle open returned no result")
                })
                .into(),
        );
    };
    match opened.resource.lease() {
        Ok(bundle) => open_bundle(bundle, Some(save), player_limits),
        Err(error) => BridgeBundlePlayerOpenResult::failure(error.into()),
    }
}

pub fn bundle_player_current(resource: &BundlePlayerResource) -> BridgeBundlePlayerActionResult {
    with_player(resource, |player| Ok(player.current().clone()))
}
pub fn bundle_player_advance(resource: &BundlePlayerResource) -> BridgeBundlePlayerActionResult {
    with_player(resource, |player| player.advance().cloned())
}
pub fn bundle_player_choose(
    resource: &BundlePlayerResource,
    index: u32,
) -> BridgeBundlePlayerActionResult {
    with_player(resource, |player| player.choose(index as usize).cloned())
}

pub fn bundle_player_history(
    resource: &BundlePlayerResource,
    start_sequence: u64,
    maximum: u32,
) -> BridgeBundlePlayerHistoryResult {
    match lock_lease(resource) {
        Ok(guard) => BridgeBundlePlayerHistoryResult {
            page: Some(history_to_bridge(
                guard
                    .as_ref()
                    .expect("checked")
                    .player
                    .history_page(start_sequence, maximum as usize),
            )),
            error: None,
        },
        Err(error) => BridgeBundlePlayerHistoryResult {
            page: None,
            error: Some(error),
        },
    }
}

pub fn bundle_player_export_save(resource: &BundlePlayerResource) -> BridgeBundlePlayerSaveResult {
    match lock_lease(resource) {
        Ok(guard) => match guard.as_ref().expect("checked").player.export_save() {
            Ok(bytes) => BridgeBundlePlayerSaveResult {
                bytes: Some(bytes),
                error: None,
            },
            Err(error) => BridgeBundlePlayerSaveResult {
                bytes: None,
                error: Some(error.into()),
            },
        },
        Err(error) => BridgeBundlePlayerSaveResult {
            bytes: None,
            error: Some(error),
        },
    }
}

pub fn bundle_player_read_asset(
    resource: &BundlePlayerResource,
    logical_path: String,
    maximum_bytes: u64,
) -> BridgeBundlePlayerAssetResult {
    match lock_lease(resource) {
        Ok(guard) => match guard
            .as_ref()
            .expect("checked")
            .bundle
            .read_asset(&logical_path, maximum_bytes)
        {
            Ok(bytes) => BridgeBundlePlayerAssetResult {
                bytes: Some(bytes),
                error: None,
            },
            Err(error) => BridgeBundlePlayerAssetResult {
                bytes: None,
                error: Some(BridgeRuntimeError::simple(
                    error.code().to_string(),
                    error.to_string(),
                )),
            },
        },
        Err(error) => BridgeBundlePlayerAssetResult {
            bytes: None,
            error: Some(error),
        },
    }
}

pub fn bundle_player_dispose(resource: &BundlePlayerResource) -> BridgeBundlePlayerDisposeResult {
    match resource.lease.try_lock() {
        Ok(mut guard) => BridgeBundlePlayerDisposeResult {
            released: guard.take().is_some(),
            error: None,
        },
        Err(TryLockError::WouldBlock) => BridgeBundlePlayerDisposeResult {
            released: false,
            error: Some(BridgeRuntimeError::busy()),
        },
        Err(TryLockError::Poisoned(_)) => BridgeBundlePlayerDisposeResult {
            released: false,
            error: Some(BridgeRuntimeError::simple(
                "R_RESOURCE_STATE",
                "player resource lock is poisoned",
            )),
        },
    }
}

fn open_bundle(
    bundle: Arc<LoadedBundle>,
    save: Option<Vec<u8>>,
    limits: BridgeRuntimeLimits,
) -> BridgeBundlePlayerOpenResult {
    let limits = match PlayerLimits::try_from(limits) {
        Ok(value) => value,
        Err(error) => return BridgeBundlePlayerOpenResult::failure(error),
    };
    let result = match save {
        Some(save) => SemanticPlayer::restore_loaded_bundle(&bundle, &save, limits),
        None => SemanticPlayer::from_loaded_bundle(&bundle, limits),
    };
    match result {
        Ok(player) => BridgeBundlePlayerOpenResult::success(player, bundle),
        Err(error) => BridgeBundlePlayerOpenResult::failure(error.into()),
    }
}

fn with_player<F>(resource: &BundlePlayerResource, operation: F) -> BridgeBundlePlayerActionResult
where
    F: FnOnce(&mut SemanticPlayer) -> Result<EventDelta, RuntimeError>,
{
    let mut guard = match lock_lease(resource) {
        Ok(value) => value,
        Err(error) => {
            return BridgeBundlePlayerActionResult {
                delta: None,
                error: Some(error),
            };
        }
    };
    match operation(&mut guard.as_mut().expect("checked").player) {
        Ok(delta) => BridgeBundlePlayerActionResult {
            delta: Some(delta_to_bridge(delta)),
            error: None,
        },
        Err(error) => BridgeBundlePlayerActionResult {
            delta: None,
            error: Some(error.into()),
        },
    }
}

fn lock_lease(
    resource: &BundlePlayerResource,
) -> Result<std::sync::MutexGuard<'_, Option<PlayerLease>>, BridgeRuntimeError> {
    match resource.lease.try_lock() {
        Ok(guard) if guard.is_some() => Ok(guard),
        Ok(_) => Err(BridgeRuntimeError::simple(
            "R_PLAYER_DISPOSED",
            "player resource has been disposed",
        )),
        Err(TryLockError::WouldBlock) => Err(BridgeRuntimeError::busy()),
        Err(TryLockError::Poisoned(_)) => Err(BridgeRuntimeError::simple(
            "R_RESOURCE_STATE",
            "player resource lock is poisoned",
        )),
    }
}

impl BridgeBundlePlayerOpenResult {
    fn success(player: SemanticPlayer, bundle: Arc<LoadedBundle>) -> Self {
        let current = delta_to_bridge(player.current().clone());
        Self {
            opened: Some(BridgeBundlePlayerOpened {
                resource: BundlePlayerResource {
                    lease: Arc::new(Mutex::new(Some(PlayerLease { player, bundle }))),
                },
                current,
            }),
            error: None,
        }
    }
    fn failure(error: BridgeRuntimeError) -> Self {
        Self {
            opened: None,
            error: Some(error),
        }
    }
}

impl BridgeRuntimeError {
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
    fn busy() -> Self {
        Self::simple("R_PLAYER_BUSY", "player operation is already in progress")
    }
}

impl From<RuntimeError> for BridgeRuntimeError {
    fn from(value: RuntimeError) -> Self {
        Self {
            code: value.code,
            scene: value.scene,
            message: value.message,
            resource: value.resource,
            actual: value.actual,
            limit: value.limit,
        }
    }
}
impl From<BridgeError> for BridgeRuntimeError {
    fn from(value: BridgeError) -> Self {
        Self::simple(value.code, value.message)
    }
}
impl From<PlayerLimits> for BridgeRuntimeLimits {
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
impl TryFrom<BridgeRuntimeLimits> for PlayerLimits {
    type Error = BridgeRuntimeError;
    fn try_from(value: BridgeRuntimeLimits) -> Result<Self, Self::Error> {
        let convert = |value| {
            usize::try_from(value).map_err(|_| {
                BridgeRuntimeError::simple("R_LIMIT_CONFIGURATION", "limit cannot be represented")
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
        .map_err(|invalid| BridgeRuntimeError {
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
fn history_to_bridge(value: HistoryPage) -> BridgePlayerHistoryPage {
    BridgePlayerHistoryPage {
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
fn history_entry_to_bridge(value: HistoryEntry) -> BridgePlayerHistoryEntry {
    BridgePlayerHistoryEntry {
        sequence: value.sequence,
        event: event_to_bridge(value.event),
        effects: value.effects.into_iter().map(effect_to_bridge).collect(),
        scene: value.scene,
    }
}
fn event_to_bridge(value: SemanticEvent) -> BridgePlayerEvent {
    let mut output = BridgePlayerEvent {
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
                .map(|item| BridgePlayerChoice {
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
fn effect_to_bridge(value: MediaEffect) -> BridgePlayerEffect {
    match value {
        MediaEffect::Background(path) => BridgePlayerEffect {
            kind: "background".into(),
            path: Some(path),
        },
        MediaEffect::Bgm(path) => BridgePlayerEffect {
            kind: "bgm".into(),
            path: Some(path),
        },
        MediaEffect::BgmStop => BridgePlayerEffect {
            kind: "bgm_stop".into(),
            path: None,
        },
        MediaEffect::Sfx(path) => BridgePlayerEffect {
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
