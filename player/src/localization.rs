//! Execution-owned immutable locale, bounded Fluent formatting, exact snapshots.
use crate::contract::{EventText, MessageSnapshot, PlayerLimits, RuntimeError};
use crate::{
    engine::Value,
    model::{LocalizationMetadata, StoryModel, StoryText, VarType},
};
use fluent::{FluentArgs, FluentResource, concurrent::FluentBundle};
use std::{collections::BTreeMap, sync::Arc};
use storyscript_bundle::localization::{Catalog, Expansion};

pub(crate) struct LocaleRuntime {
    pub locale: String,
    bundle: FluentBundle<FluentResource>,
    messages: BTreeMap<String, Expansion>,
}

pub fn negotiate(
    metadata: &LocalizationMetadata,
    requested: &[String],
) -> Result<String, RuntimeError> {
    let tags: Vec<icu_locale_core::Locale> = requested
        .iter()
        .map(|request| {
            if request.contains('_') {
                return Err(error(
                    "R_LOCALIZATION_LOCALE",
                    "",
                    "requested locale must use BCP-47 hyphens",
                ));
            }
            request.parse::<icu_locale_core::Locale>().map_err(|_| {
                error(
                    "R_LOCALIZATION_LOCALE",
                    "",
                    "invalid requested BCP-47 locale",
                )
            })
        })
        .collect::<Result<_, RuntimeError>>()?;
    for tag in tags {
        let canonical = tag.to_string();
        if metadata.supported_locales.contains(&canonical) {
            return Ok(canonical);
        }
        let language = tag.id.language.to_string();
        if metadata.supported_locales.contains(&language) {
            return Ok(language);
        }
    }
    Ok(metadata.default_locale.clone())
}

impl LocaleRuntime {
    pub(crate) fn from_catalog(catalog: Catalog) -> Result<Arc<Self>, RuntimeError> {
        let locale = catalog.locale;
        let parsed: icu_locale_core::Locale = locale
            .parse()
            .map_err(|_| error("R_LOCALIZATION_LOCALE", "", "invalid configured locale"))?;
        // Catalog identity keeps the complete canonical tag. Fluent consumes
        // its language identifier; extensions do not mutate numeric snapshots.
        let tag = parsed
            .id
            .to_string()
            .parse::<unic_langid::LanguageIdentifier>()
            .map_err(|_| {
                error(
                    "R_LOCALIZATION_LOCALE",
                    "",
                    "invalid Fluent language identifier",
                )
            })?;
        let resource = FluentResource::try_new(catalog.canonical).map_err(|_| {
            error(
                "R_LOCALIZATION_CATALOG",
                "",
                "invalid validated Fluent resource",
            )
        })?;
        let mut bundle = FluentBundle::new_concurrent(vec![tag]);
        bundle
            .add_resource(resource)
            .map_err(|_| error("R_LOCALIZATION_CATALOG", "", "conflicting catalog entries"))?;
        bundle.add_builtins().map_err(|_| {
            error(
                "R_LOCALIZATION_CATALOG",
                "",
                "could not register safe NUMBER function",
            )
        })?;
        // Fluent's default isolation is intentionally retained.
        Ok(Arc::new(Self {
            locale,
            bundle,
            messages: catalog.messages,
        }))
    }
    pub(crate) fn work(&self, id: &str) -> usize {
        self.messages
            .get(id)
            .map_or(1, |m| m.work.saturating_add(m.literal_bytes))
    }

    /// Charge conservative UTF-8 copy work as well as AST resolution. A small
    /// saved reference to a large pattern must not allocate gigabytes while
    /// rerendering thousands of current/pending/history references.
    pub(crate) fn snapshot_work(&self, snapshot: &MessageSnapshot) -> usize {
        snapshot
            .arguments
            .iter()
            .fold(self.work(&snapshot.id), |total, arg| {
                let occurrences = self
                    .messages
                    .get(&snapshot.id)
                    .and_then(|m| m.variables.get(&arg.name))
                    .copied()
                    .unwrap_or(1);
                let bytes = arg.name.len().saturating_add(match &arg.value {
                    Value::Str(v) => v.len(),
                    _ => 128,
                });
                total.saturating_add(occurrences.saturating_mul(bytes))
            })
    }
}

pub(crate) fn render_snapshot(
    runtime: Option<&LocaleRuntime>,
    snapshot: MessageSnapshot,
    limits: PlayerLimits,
    scene: &str,
) -> Result<EventText, RuntimeError> {
    let rendered = if let Some(runtime) = runtime {
        let expansion = runtime
            .messages
            .get(&snapshot.id)
            .ok_or_else(|| error("R_LOCALIZATION_MESSAGE", scene, "unknown message ID"))?;
        let mut bound = expansion.literal_bytes;
        let mut args = FluentArgs::new();
        for argument in &snapshot.arguments {
            let occurrences = expansion.variables.get(&argument.name).ok_or_else(|| {
                error("R_LOCALIZATION_ARGUMENT", scene, "unknown message argument")
            })?;
            let bytes = match &argument.value {
                Value::Str(v) => v.len(),
                _ => 128,
            };
            bound = bound.saturating_add(occurrences.saturating_mul(bytes));
            match (&argument.value, argument.var_type) {
                (Value::Str(v), VarType::String) => args.set(&argument.name, v.as_str()),
                (Value::Bool(v), VarType::Boolean) => {
                    args.set(&argument.name, if *v { "true" } else { "false" })
                }
                (Value::Int(v), VarType::Integer) => {
                    let n = storyscript_bundle::localization::exact_decimal_number(
                        rust_decimal::Decimal::from(*v),
                    )
                    .ok_or_else(|| {
                        error(
                            "R_LOCALIZATION_NUMBER",
                            scene,
                            "integer cannot round-trip exactly through Fluent f64",
                        )
                    })?;
                    args.set(&argument.name, n);
                }
                (Value::Decimal(v), VarType::Decimal) => {
                    let n = storyscript_bundle::localization::exact_decimal_number(*v).ok_or_else(
                        || {
                            error(
                                "R_LOCALIZATION_NUMBER",
                                scene,
                                "decimal cannot round-trip exactly through Fluent f64",
                            )
                        },
                    )?;
                    args.set(&argument.name, n);
                }
                _ => {
                    return Err(error(
                        "R_LOCALIZATION_ARGUMENT",
                        scene,
                        "invalid scalar argument type",
                    ));
                }
            }
        }
        if snapshot.arguments.len() != expansion.variables.len() {
            return Err(error(
                "R_LOCALIZATION_ARGUMENT",
                scene,
                "missing message arguments",
            ));
        }
        // Protect allocation before the resolver; apply the host's exact event
        // limit after formatting, not to this conservative all-branch estimate.
        if bound > crate::contract::HARD_LIMITS.rendered_bytes {
            return Err(output_limit(
                scene,
                bound,
                crate::contract::HARD_LIMITS.rendered_bytes,
            ));
        }
        let message = runtime
            .bundle
            .get_message(&snapshot.id)
            .ok_or_else(|| error("R_LOCALIZATION_MESSAGE", scene, "message is missing"))?;
        let pattern = message
            .value()
            .ok_or_else(|| error("R_LOCALIZATION_MESSAGE", scene, "message has no pattern"))?;
        let mut errors = Vec::new();
        let value = runtime
            .bundle
            .format_pattern(pattern, Some(&args), &mut errors);
        if !errors.is_empty() {
            return Err(error(
                "R_LOCALIZATION_FORMAT",
                scene,
                "Fluent resolver failed",
            ));
        }
        value.into_owned()
    } else {
        if !snapshot.arguments.is_empty() {
            return Err(error(
                "R_LOCALIZATION_ARGUMENT",
                scene,
                "raw message must not contain project arguments",
            ));
        }
        snapshot.id.clone()
    };
    if rendered.len() > limits.rendered_bytes {
        return Err(output_limit(scene, rendered.len(), limits.rendered_bytes));
    }
    Ok(EventText {
        rendered,
        message: Some(snapshot),
    })
}

pub(crate) fn contracts(model: &StoryModel) -> BTreeMap<String, Vec<(String, VarType)>> {
    let mut result = BTreeMap::new();
    visit_model_texts(&mut model.clone(), &mut |text| {
        if let StoryText::Message { id, arguments } = text {
            result.insert(id.clone(), arguments.clone());
        }
    });
    result
}

pub(crate) fn visit_model_texts(model: &mut StoryModel, f: &mut impl FnMut(&mut StoryText)) {
    use crate::model::{ChoiceEntry as C, StoryStatement as S};
    fn choices(entries: &mut [crate::model::ChoiceEntry], f: &mut impl FnMut(&mut StoryText)) {
        for entry in entries {
            match entry {
                C::Option(v) => f(&mut v.text),
                C::If(v) => choices(&mut v.body, f),
                C::Repeat(v) => choices(&mut v.body, f),
                C::ForSnapshot(v) => choices(&mut v.body, f),
            }
        }
    }
    fn statements(stmts: &mut [crate::model::StoryStatement], f: &mut impl FnMut(&mut StoryText)) {
        for stmt in stmts {
            match stmt {
                S::Narration { text, .. } => f(text),
                S::Dialogue(v) => f(&mut v.text),
                S::Choice(v) => choices(&mut v.entries, f),
                S::IfElse(v) => {
                    statements(&mut v.then_branch, f);
                    if let Some(b) = &mut v.else_branch {
                        statements(b, f);
                    }
                }
                S::Repeat(v) => statements(&mut v.body, f),
                S::ForSnapshot(v) => statements(&mut v.body, f),
                _ => {}
            }
        }
    }
    for scene in &mut model.scenes {
        statements(&mut scene.story, f);
    }
}

pub(crate) fn error(code: &str, scene: &str, message: &str) -> RuntimeError {
    RuntimeError {
        code: code.into(),
        scene: scene.into(),
        message: message.into(),
        resource: None,
        actual: None,
        limit: None,
    }
}
fn output_limit(scene: &str, actual: usize, limit: usize) -> RuntimeError {
    RuntimeError {
        code: "R_EXECUTION_LIMIT".into(),
        scene: scene.into(),
        message: "localized output exceeds event limit".into(),
        resource: Some("rendered_bytes".into()),
        actual: Some(actual as u64),
        limit: Some(limit as u64),
    }
}
