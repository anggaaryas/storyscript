use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use storyscript_player::{StepResult, StoryPlayer, Value};

static SESSION_ID: AtomicU64 = AtomicU64::new(1);
static SESSIONS: OnceLock<Mutex<HashMap<u64, StoryPlayer>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<u64, StoryPlayer>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
pub struct BridgeChoice {
    pub text: String,
    pub target: String,
}

#[derive(Clone)]
pub struct BridgeStep {
    pub kind: String,
    pub text: Option<String>,
    pub actor_name: Option<String>,
    pub actor_id: Option<String>,
    pub emotion: Option<String>,
    pub position: Option<String>,
    pub choices: Vec<BridgeChoice>,
}

#[derive(Clone)]
pub struct BridgeVariable {
    pub name: String,
    pub value: String,
}

#[derive(Clone)]
pub struct BridgeState {
    pub session_id: u64,
    pub script_name: String,
    pub scene: String,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pub finished: bool,
    pub variables: Vec<BridgeVariable>,
    pub current: Option<BridgeStep>,
    pub history: Vec<BridgeStep>,
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_open(path: String) -> Result<u64, String> {
    let player = StoryPlayer::from_file(&PathBuf::from(path))?;
    let id = SESSION_ID.fetch_add(1, Ordering::Relaxed);

    let mut guard = sessions()
        .lock()
        .map_err(|_| "Session store lock poisoned".to_string())?;
    guard.insert(id, player);

    Ok(id)
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_open_raw(source: String) -> Result<u64, String> {
    let player = StoryPlayer::from_source("inline", &source)?;
    let id = SESSION_ID.fetch_add(1, Ordering::Relaxed);

    let mut guard = sessions()
        .lock()
        .map_err(|_| "Session store lock poisoned".to_string())?;
    guard.insert(id, player);

    Ok(id)
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_close(session_id: u64) -> bool {
    if let Ok(mut guard) = sessions().lock() {
        return guard.remove(&session_id).is_some();
    }
    false
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_advance(session_id: u64) -> Result<BridgeState, String> {
    let mut guard = sessions()
        .lock()
        .map_err(|_| "Session store lock poisoned".to_string())?;
    let player = guard
        .get_mut(&session_id)
        .ok_or_else(|| format!("Unknown session id {}", session_id))?;

    player.advance();
    Ok(snapshot(session_id, player))
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_choose(session_id: u64, index: u32) -> Result<BridgeState, String> {
    let mut guard = sessions()
        .lock()
        .map_err(|_| "Session store lock poisoned".to_string())?;
    let player = guard
        .get_mut(&session_id)
        .ok_or_else(|| format!("Unknown session id {}", session_id))?;

    if !player.select_choice(index as usize) {
        return Err(format!("Invalid choice index {}", index));
    }

    Ok(snapshot(session_id, player))
}

#[flutter_rust_bridge::frb(sync)]
pub fn player_get_state(session_id: u64) -> Result<BridgeState, String> {
    let guard = sessions()
        .lock()
        .map_err(|_| "Session store lock poisoned".to_string())?;
    let player = guard
        .get(&session_id)
        .ok_or_else(|| format!("Unknown session id {}", session_id))?;

    Ok(snapshot(session_id, player))
}

fn snapshot(session_id: u64, player: &StoryPlayer) -> BridgeState {
    let mut variables = player
        .engine()
        .variables
        .iter()
        .map(|(name, value)| BridgeVariable {
            name: name.clone(),
            value: value_to_string(value),
        })
        .collect::<Vec<_>>();
    variables.sort_by(|a, b| a.name.cmp(&b.name));

    BridgeState {
        session_id,
        script_name: player.script_name().to_string(),
        scene: player.engine().current_scene.clone(),
        bg: player.engine().bg.clone(),
        bgm: player.engine().bgm.clone(),
        finished: player.engine().finished,
        variables,
        current: player.current().map(step_to_bridge),
        history: player.history().iter().map(step_to_bridge).collect(),
    }
}

fn step_to_bridge(step: &StepResult) -> BridgeStep {
    match step {
        StepResult::Narration(text) => BridgeStep {
            kind: "narration".to_string(),
            text: Some(text.clone()),
            actor_name: None,
            actor_id: None,
            emotion: None,
            position: None,
            choices: Vec::new(),
        },
        StepResult::Dialogue {
            actor_name,
            actor_id,
            emotion,
            position,
            text,
        } => BridgeStep {
            kind: "dialogue".to_string(),
            text: Some(text.clone()),
            actor_name: Some(actor_name.clone()),
            actor_id: Some(actor_id.clone()),
            emotion: emotion.clone(),
            position: position.clone(),
            choices: Vec::new(),
        },
        StepResult::Choices(options) => BridgeStep {
            kind: "choices".to_string(),
            text: None,
            actor_name: None,
            actor_id: None,
            emotion: None,
            position: None,
            choices: options
                .iter()
                .map(|choice| BridgeChoice {
                    text: choice.text.clone(),
                    target: choice.target.clone(),
                })
                .collect(),
        },
        StepResult::End => BridgeStep {
            kind: "end".to_string(),
            text: None,
            actor_name: None,
            actor_id: None,
            emotion: None,
            position: None,
            choices: Vec::new(),
        },
    }
}

fn value_to_string(value: &Value) -> String {
    value.to_string()
}