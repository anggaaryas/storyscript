//! Source-only message inventory. Paths and coordinates never enter compiled IR.
use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::token::{Spanned, Token};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

pub fn valid_message_id(id: &str) -> bool {
    id.len() <= 256
        && id.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[derive(Debug, Clone)]
pub struct MessageSite {
    pub text: LocalizedText,
    pub scene: String,
    pub kind: &'static str,
    pub visible_variables: Arc<VariableScope>,
}

/// Persistent lexical frames avoid copying globals into every message or loop.
#[derive(Debug)]
pub struct VariableScope {
    variables: BTreeMap<String, VarType>,
    parent: Option<Arc<VariableScope>>,
}
impl VariableScope {
    pub fn get(&self, name: &str) -> Option<&VarType> {
        let mut frame = self;
        loop {
            if let Some(ty) = frame.variables.get(name) {
                return Some(ty);
            }
            frame = frame.parent.as_deref()?;
        }
    }
}
impl std::ops::Index<&str> for VariableScope {
    type Output = VarType;
    fn index(&self, name: &str) -> &VarType {
        self.get(name).expect("validated visible variable")
    }
}

pub fn inventory(script: &Script) -> Vec<MessageSite> {
    let globals = Arc::new(VariableScope {
        variables: script
            .init
            .variables
            .iter()
            .map(|v| (v.name.clone(), v.var_type))
            .collect(),
        parent: None,
    });
    let mut sites = Vec::new();
    for scene in &script.scenes {
        let mut locals = BTreeMap::new();
        if let Some(prep) = &scene.prep {
            prep_scope(&prep.statements, &mut locals);
        }
        let scope = if locals.is_empty() {
            globals.clone()
        } else {
            Arc::new(VariableScope {
                variables: locals,
                parent: Some(globals.clone()),
            })
        };
        story_sites(&scene.story.statements, &scope, &scene.label, &mut sites);
    }
    sites
}

fn prep_scope(statements: &[PrepStatement], scope: &mut BTreeMap<String, VarType>) {
    for stmt in statements {
        match stmt {
            PrepStatement::VarDecl(v) => {
                scope.insert(v.name.clone(), v.var_type);
            }
            PrepStatement::IfElse(v) => {
                prep_scope(&v.then_branch, scope);
                if let Some(branch) = &v.else_branch {
                    prep_scope(branch, scope);
                }
            }
            PrepStatement::Repeat(v) => prep_scope(&v.body, scope),
            PrepStatement::ForSnapshot(v) => prep_scope(&v.body, scope),
            _ => {}
        }
    }
}

fn loop_scope(scope: &Arc<VariableScope>, item: &str, array: &str) -> Arc<VariableScope> {
    let ty = match scope.get(array) {
        Some(VarType::ArrayInteger) => Some(VarType::Integer),
        Some(VarType::ArrayString) => Some(VarType::String),
        Some(VarType::ArrayBoolean) => Some(VarType::Boolean),
        Some(VarType::ArrayDecimal) => Some(VarType::Decimal),
        _ => None,
    };
    if let Some(ty) = ty {
        Arc::new(VariableScope {
            variables: BTreeMap::from([(item.into(), ty)]),
            parent: Some(scope.clone()),
        })
    } else {
        scope.clone()
    }
}

fn site(
    text: &StoryText,
    scope: &Arc<VariableScope>,
    scene: &str,
    kind: &'static str,
    out: &mut Vec<MessageSite>,
) {
    if let StoryText::Localized(text) = text {
        out.push(MessageSite {
            text: text.clone(),
            scene: scene.into(),
            kind,
            visible_variables: scope.clone(),
        });
    }
}
fn story_sites(
    statements: &[StoryStatement],
    scope: &Arc<VariableScope>,
    scene: &str,
    out: &mut Vec<MessageSite>,
) {
    for stmt in statements {
        match stmt {
            StoryStatement::Narration { text, .. } => site(text, scope, scene, "narration", out),
            StoryStatement::Dialogue(v) => site(&v.text, scope, scene, "dialogue", out),
            StoryStatement::Choice(v) => choice_sites(&v.entries, scope, scene, out),
            StoryStatement::IfElse(v) => {
                story_sites(&v.then_branch, scope, scene, out);
                if let Some(branch) = &v.else_branch {
                    story_sites(branch, scope, scene, out);
                }
            }
            StoryStatement::Repeat(v) => story_sites(&v.body, scope, scene, out),
            StoryStatement::ForSnapshot(v) => story_sites(
                &v.body,
                &loop_scope(scope, &v.item_name, &v.array_name),
                scene,
                out,
            ),
            _ => {}
        }
    }
}
fn choice_sites(
    entries: &[ChoiceEntry],
    scope: &Arc<VariableScope>,
    scene: &str,
    out: &mut Vec<MessageSite>,
) {
    for entry in entries {
        match entry {
            ChoiceEntry::Option(v) => site(&v.text, scope, scene, "choice", out),
            ChoiceEntry::If(v) => choice_sites(&v.body, scope, scene, out),
            ChoiceEntry::Repeat(v) => choice_sites(&v.body, scope, scene, out),
            ChoiceEntry::ForSnapshot(v) => choice_sites(
                &v.body,
                &loop_scope(scope, &v.item_name, &v.array_name),
                scene,
                out,
            ),
        }
    }
}

pub fn validate_ids(script: &Script) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut diagnostics = Vec::new();
    for site in inventory(script) {
        let code = if !valid_message_id(&site.text.id) {
            Some(DiagnosticCode::ELocalizationIdInvalid)
        } else if !seen.insert(site.text.id.clone()) {
            Some(DiagnosticCode::ELocalizationIdDuplicate)
        } else {
            None
        };
        if let Some(code) = code {
            diagnostics.push(Diagnostic::new(
                code,
                format!(
                    "{}:{}:{}: Invalid or duplicate message ID '{}'",
                    site.text.source, site.text.line, site.text.column, site.text.id
                ),
                Phase::Validation,
                site.scene,
                site.text.line,
                site.text.column,
            ));
        }
    }
    diagnostics
}

pub(crate) fn mark_source(scenes: &mut [Scene], source: &str) {
    for scene in scenes {
        visit_story(&mut scene.story.statements, &mut |text| {
            if let StoryText::Localized(text) = text {
                text.source = source.into();
            }
        });
    }
}
fn visit_story(statements: &mut [StoryStatement], f: &mut impl FnMut(&mut StoryText)) {
    for stmt in statements {
        match stmt {
            StoryStatement::Narration { text, .. } => f(text),
            StoryStatement::Dialogue(v) => f(&mut v.text),
            StoryStatement::Choice(v) => visit_choices(&mut v.entries, f),
            StoryStatement::IfElse(v) => {
                visit_story(&mut v.then_branch, f);
                if let Some(v) = &mut v.else_branch {
                    visit_story(v, f);
                }
            }
            StoryStatement::Repeat(v) => visit_story(&mut v.body, f),
            StoryStatement::ForSnapshot(v) => visit_story(&mut v.body, f),
            _ => {}
        }
    }
}
fn visit_choices(entries: &mut [ChoiceEntry], f: &mut impl FnMut(&mut StoryText)) {
    for entry in entries {
        match entry {
            ChoiceEntry::Option(v) => f(&mut v.text),
            ChoiceEntry::If(v) => visit_choices(&mut v.body, f),
            ChoiceEntry::Repeat(v) => visit_choices(&mut v.body, f),
            ChoiceEntry::ForSnapshot(v) => visit_choices(&mut v.body, f),
        }
    }
}

// Any keyed token not represented by an allowed parsed text node is an excluded
// site, even if parser recovery skipped it. This keeps one stable site diagnostic.
pub(crate) fn validate_token_sites(tokens: &[Spanned], scenes: &[Scene]) -> Vec<Diagnostic> {
    let mut locations = HashSet::new();
    let mut scenes = scenes.to_vec();
    for scene in &mut scenes {
        visit_story(&mut scene.story.statements, &mut |text| {
            if let StoryText::Localized(v) = text {
                locations.insert((v.line, v.column));
            }
        });
    }
    tokens
        .iter()
        .filter_map(|token| {
            if matches!(token.token, Token::LocalizedText(_))
                && !locations.contains(&(token.line, token.column))
            {
                Some(Diagnostic::new(
                    DiagnosticCode::ELocalizationSiteForbidden,
                    "Keyed text is only allowed in narration, dialogue bodies and choice labels",
                    Phase::Parse,
                    "GLOBAL",
                    token.line,
                    token.column,
                ))
            } else {
                None
            }
        })
        .collect()
}
