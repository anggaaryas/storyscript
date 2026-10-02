use std::path::Path;

use storyscript_parser::ast::Script;

use crate::contract::{
    EventDelta, HistoryPage, Origin, PlayerLimits, RuntimeError, SemanticEvent, SessionStatus,
};
use crate::engine::{ChoiceDisplay, Engine, StepResult};
use crate::history::HistoryBuffer;
use crate::model::StoryModel;

/// Compact semantic session shared by source and verified StoryBundle adapters.
/// It owns bounded history, transactional progression, and exact-origin saves.
#[derive(Clone)]
pub struct SemanticPlayer {
    pub(crate) engine: Engine,
    pub(crate) current: EventDelta,
    pub(crate) history: HistoryBuffer,
    pub(crate) origin: Origin,
    pub(crate) limits: PlayerLimits,
}

impl std::fmt::Debug for SemanticPlayer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SemanticPlayer")
            .field("current", &self.current)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl SemanticPlayer {
    /// `true` means keyed text is displayed as IDs, not translated output.
    pub fn has_unresolved_localization(&self) -> bool {
        self.engine.model().has_keyed_text() && self.resolved_locale().is_none()
    }
    pub fn resolved_locale(&self) -> Option<&str> {
        self.engine.locale_runtime().map(|r| r.locale.as_str())
    }

    pub fn from_project(
        root: &Path,
        requested: &[String],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        Self::from_project_seeded(root, requested, rand::random(), limits)
    }

    pub fn from_project_seeded(
        root: &Path,
        requested: &[String],
        seed: [u8; 32],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let (model, localization) = project_model(root, requested)?;
        let origin = crate::save::source_origin(&model);
        Self::from_localized_model_seeded(model, origin, seed, limits, localization)
    }

    pub fn restore_project(
        root: &Path,
        bytes: &[u8],
        requested: &[String],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let (model, localization) = project_model(root, requested)?;
        let origin = crate::save::source_origin(&model);
        crate::save::restore_localized_model(model, origin, bytes, limits, localization)
    }

    pub fn from_source(source: &str, limits: PlayerLimits) -> Result<Self, String> {
        let compiled = storyscript_parser::compiler::compile_source(source);
        if compiled.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                "Compile errors in source",
                &compiled
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }
        let script = compiled.script.ok_or("Compile failed to produce script")?;
        Self::new_seeded(&script, rand::random(), limits)
            .map_err(|e| format!("[{}] {}", e.code, e.message))
    }

    pub fn from_file(path: &Path, limits: PlayerLimits) -> Result<Self, String> {
        let compiled = storyscript_parser::compiler::compile_file(path)
            .map_err(|e| format!("Failed to compile {}: {}", display_path(path), e))?;
        if compiled.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                &format!("Compile errors in {}", display_path(path)),
                &compiled
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }
        let script = compiled.script.ok_or("Compile failed to produce script")?;
        Self::new_seeded(&script, rand::random(), limits)
            .map_err(|e| format!("[{}] {}", e.code, e.message))
    }

    pub fn new_seeded(
        script: &Script,
        seed: [u8; 32],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let model = crate::adapters::ast::adapt(script);
        let origin = crate::save::source_origin(&model);
        Self::from_model_seeded(model, origin, seed, limits)
    }

    pub fn from_model_seeded(
        model: StoryModel,
        origin: Origin,
        seed: [u8; 32],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        Self::from_localized_model_seeded(model, origin, seed, limits, None)
    }

    fn from_localized_model_seeded(
        model: StoryModel,
        origin: Origin,
        seed: [u8; 32],
        limits: PlayerLimits,
        localization: Option<std::sync::Arc<crate::localization::LocaleRuntime>>,
    ) -> Result<Self, RuntimeError> {
        let mut engine = Engine::open_localized_checked(&model, seed, limits, localization)?;
        let history = HistoryBuffer::new(limits).map_err(|invalid| RuntimeError {
            code: "R_LIMIT_CONFIGURATION".into(),
            scene: model.init.start.clone(),
            message: format!("invalid limit: {}", invalid.resource),
            resource: Some(invalid.resource.into()),
            actual: Some(invalid.requested as u64),
            limit: Some(invalid.hard_maximum as u64),
        })?;
        let (event, effects) = engine.advance_checked()?.ok_or_else(|| RuntimeError {
            code: "R_INVALID_STATE".into(),
            scene: engine.current_scene.clone(),
            message: "opening a scene produced no event".into(),
            resource: None,
            actual: None,
            limit: None,
        })?;
        let current = EventDelta {
            status: status(&event),
            scene: engine.current_scene.clone(),
            current: event,
            effects,
            sequence: 0,
            first_retained_sequence: 0,
            omitted_history_count: 0,
        };
        Ok(Self {
            engine,
            current,
            history,
            origin,
            limits,
        })
    }

    pub fn current(&self) -> &EventDelta {
        &self.current
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    #[cfg(feature = "storybundle-runtime")]
    pub fn from_loaded_bundle(
        bundle: &storyscript_bundle::loader::LoadedBundle,
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        Self::from_loaded_bundle_with_locales(bundle, &[], limits)
    }

    #[cfg(feature = "storybundle-runtime")]
    pub fn from_loaded_bundle_with_locales(
        bundle: &storyscript_bundle::loader::LoadedBundle,
        requested: &[String],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let (model, origin) = crate::adapters::bundle::adapt_verified(bundle)?;
        let localization = bundle_locale(bundle, &model, requested)?;
        Self::from_localized_model_seeded(model, origin, rand::random(), limits, localization)
    }

    #[cfg(feature = "storybundle-runtime")]
    pub fn restore_loaded_bundle(
        bundle: &storyscript_bundle::loader::LoadedBundle,
        bytes: &[u8],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        Self::restore_loaded_bundle_with_locales(bundle, bytes, &[], limits)
    }

    #[cfg(feature = "storybundle-runtime")]
    pub fn restore_loaded_bundle_with_locales(
        bundle: &storyscript_bundle::loader::LoadedBundle,
        bytes: &[u8],
        requested: &[String],
        limits: PlayerLimits,
    ) -> Result<Self, RuntimeError> {
        let (model, origin) = crate::adapters::bundle::adapt_verified(bundle)?;
        let localization = bundle_locale(bundle, &model, requested)?;
        crate::save::restore_localized_model(model, origin, bytes, limits, localization)
    }

    pub fn history_page(&self, start_sequence: u64, maximum: usize) -> HistoryPage {
        self.history.page(start_sequence, maximum)
    }

    pub fn advance(&mut self) -> Result<&EventDelta, RuntimeError> {
        if self.current.status != SessionStatus::Active {
            return Err(self.action_error("R_INVALID_ACTION", "session is no longer active"));
        }
        if matches!(self.current.current, SemanticEvent::Choices(_)) {
            return Err(self.action_error("R_INVALID_ACTION", "a choice is required"));
        }
        let mut candidate = self.clone();
        let (event, effects) = candidate
            .engine
            .advance_checked()?
            .ok_or_else(|| self.action_error("R_INVALID_STATE", "scene produced no next event"))?;
        candidate.commit_event(event, effects)?;
        *self = candidate;
        Ok(&self.current)
    }

    pub fn choose(&mut self, index: usize) -> Result<&EventDelta, RuntimeError> {
        let choices = match &self.current.current {
            SemanticEvent::Choices(choices) => choices,
            _ => {
                return Err(
                    self.action_error("R_INVALID_ACTION", "session is not waiting for a choice")
                );
            }
        };
        let choice = choices
            .get(index)
            .ok_or_else(|| self.action_error("R_INVALID_CHOICE", "choice index out of range"))?;
        let choice = ChoiceDisplay {
            text: choice.text.clone(),
            target: choice.target_scene.clone(),
        };
        let mut candidate = self.clone();
        let (event, effects) = candidate
            .engine
            .choose_checked(&choice)?
            .ok_or_else(|| self.action_error("R_INVALID_STATE", "choice produced no event"))?;
        candidate.commit_event(event, effects)?;
        *self = candidate;
        Ok(&self.current)
    }

    fn commit_event(
        &mut self,
        event: SemanticEvent,
        effects: Vec<crate::contract::MediaEffect>,
    ) -> Result<(), RuntimeError> {
        self.history
            .append(
                self.current.current.clone(),
                self.current.effects.clone(),
                self.current.scene.clone(),
            )
            .map_err(|message| self.action_error("R_EXECUTION_LIMIT", message))?;
        self.current = EventDelta {
            scene: self.engine.current_scene.clone(),
            status: status(&event),
            current: event,
            effects,
            sequence: self.history.next_sequence(),
            first_retained_sequence: self.history.first_retained_sequence(),
            omitted_history_count: self.history.omitted_history_count(),
        };
        Ok(())
    }

    fn action_error(&self, code: &str, message: &str) -> RuntimeError {
        RuntimeError {
            code: code.into(),
            scene: self.current.scene.clone(),
            message: message.into(),
            resource: None,
            actual: None,
            limit: None,
        }
    }
}

fn status(event: &SemanticEvent) -> SessionStatus {
    match event {
        SemanticEvent::End => SessionStatus::Finished,
        SemanticEvent::Error(_) => SessionStatus::Faulted,
        _ => SessionStatus::Active,
    }
}

fn project_model(
    root: &Path,
    requested: &[String],
) -> Result<
    (
        StoryModel,
        Option<std::sync::Arc<crate::localization::LocaleRuntime>>,
    ),
    RuntimeError,
> {
    let project = storyscript_bundle::project::compile(root).map_err(|_| {
        crate::localization::error(
            "R_SOURCE_COMPILE",
            "",
            "project or catalogs failed validation",
        )
    })?;
    let model = crate::adapters::bundle::adapt_story(&project.story)?;
    let localization = if let Some(metadata) = &model.localization {
        let locale = crate::localization::negotiate(metadata, requested)?;
        let catalog = project
            .catalogs
            .into_iter()
            .find(|c| c.locale == locale)
            .expect("validated supported catalog");
        let catalog = storyscript_bundle::localization::parse_catalog(
            &locale,
            &catalog.canonical,
            "validated project catalog",
        )
        .map_err(|_| {
            crate::localization::error(
                "R_LOCALIZATION_CATALOG",
                "",
                "validated project catalog analysis failed",
            )
        })?;
        Some(crate::localization::LocaleRuntime::from_catalog(catalog)?)
    } else {
        None
    };
    Ok((model, localization))
}

#[cfg(feature = "storybundle-runtime")]
fn bundle_locale(
    bundle: &storyscript_bundle::loader::LoadedBundle,
    model: &StoryModel,
    requested: &[String],
) -> Result<Option<std::sync::Arc<crate::localization::LocaleRuntime>>, RuntimeError> {
    if let Some(metadata) = &model.localization {
        let locale = crate::localization::negotiate(metadata, requested)?;
        let source = bundle
            .read_catalog(&locale, storyscript_bundle::limits::MAX_CATALOG_BYTES)
            .map_err(|_| {
                crate::localization::error(
                    "R_LOCALIZATION_CATALOG",
                    "",
                    "verified catalog read failed",
                )
            })?;
        let catalog =
            storyscript_bundle::localization::parse_catalog(&locale, &source, "verified catalog")
                .map_err(|_| {
                crate::localization::error(
                    "R_LOCALIZATION_CATALOG",
                    "",
                    "verified catalog validation failed",
                )
            })?;
        Ok(Some(crate::localization::LocaleRuntime::from_catalog(
            catalog,
        )?))
    } else {
        Ok(None)
    }
}

pub struct StoryPlayer {
    script_name: String,
    engine: Engine,
    history: Vec<StepResult>,
    current: Option<StepResult>,
}

impl StoryPlayer {
    pub fn has_unresolved_localization(&self) -> bool {
        self.engine.model().has_keyed_text()
    }
    pub fn resolved_locale(&self) -> Option<&str> {
        None
    }

    pub fn new(script_name: impl Into<String>, script: &Script) -> Self {
        Self::with_engine(script_name.into(), Engine::new(script))
    }

    /// Deterministic test/session construction; the seed also covers INIT.
    pub fn new_seeded(script_name: impl Into<String>, script: &Script, seed: [u8; 32]) -> Self {
        Self::with_engine(script_name.into(), Engine::new_seeded(script, seed))
    }

    fn with_engine(script_name: String, engine: Engine) -> Self {
        let mut player = Self {
            script_name,
            engine,
            history: Vec::new(),
            current: None,
        };
        player.advance();
        player
    }

    pub fn from_source(script_name: impl Into<String>, source: &str) -> Result<Self, String> {
        let script_name = script_name.into();
        let compile = storyscript_parser::compiler::compile_source(source);

        if compile.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                &format!("Compile errors in {}", script_name),
                &compile
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }

        let script = compile
            .script
            .ok_or_else(|| format!("Compile failed to produce script for {}", script_name))?;

        Ok(Self::new(script_name, &script))
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let compile = storyscript_parser::compiler::compile_file(path)
            .map_err(|e| format!("Failed to compile {}: {}", display_path(path), e))?;

        if compile.diagnostics.iter().any(|d| d.is_error()) {
            return Err(format_diagnostics(
                &format!("Compile errors in {}", display_path(path)),
                &compile
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ));
        }

        let script = compile.script.ok_or_else(|| {
            format!(
                "Compile failed to produce script for {}",
                display_path(path)
            )
        })?;

        Ok(Self::new(display_path(path), &script))
    }

    pub fn script_name(&self) -> &str {
        &self.script_name
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn history(&self) -> &[StepResult] {
        &self.history
    }

    pub fn current(&self) -> Option<&StepResult> {
        self.current.as_ref()
    }

    pub fn advance(&mut self) {
        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        match self.engine.step() {
            Some(result) => self.current = Some(result),
            None => self.current = Some(StepResult::End),
        }
    }

    pub fn select_choice(&mut self, index: usize) -> bool {
        let choices = match self.current.as_ref() {
            Some(StepResult::Choices(choices)) => choices,
            _ => return false,
        };

        if index >= choices.len() {
            return false;
        }

        let choice = choices[index].clone();

        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        self.history
            .push(StepResult::Narration(format!("▸ {}", choice.text)));
        self.engine.select_choice(&choice);

        match self.engine.step() {
            Some(result) => self.current = Some(result),
            None => self.current = Some(StepResult::End),
        }

        true
    }
}

fn format_diagnostics(title: &str, diags: &[String]) -> String {
    if diags.is_empty() {
        return title.to_string();
    }
    let details = diags
        .iter()
        .map(|diag| format!("- {}", diag))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n{}", title, details)
}

fn display_path(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&cwd) {
            return relative.display().to_string();
        }
    }
    path.display().to_string()
}
