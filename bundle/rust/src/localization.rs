//! Bounded Fluent profile, strict project coverage and source-independent contracts.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::Localization;
use crate::limits::{MAX_CATALOG_BYTES, MAX_MESSAGE_IDS};
use crate::proto::storybundle::v1 as pb;
use crate::{BundleError, Result};
use fluent_syntax::{ast as f, parser, serializer};
use storyscript_parser::{ast::VarType, localization::MessageSite};

const MAX_WORK: usize = 100_000;
const MAX_DEPTH: usize = 128;

#[derive(Debug, Clone, Default)]
pub struct Expansion {
    pub variables: BTreeMap<String, usize>,
    pub numeric_variables: BTreeSet<String>,
    pub work: usize,
    pub literal_bytes: usize,
    pub depth: usize,
}
#[derive(Debug, Clone)]
pub struct Catalog {
    pub locale: String,
    pub canonical: String,
    pub messages: BTreeMap<String, Expansion>,
    pub records: BTreeSet<String>,
    dependencies: BTreeMap<String, Vec<String>>,
}

/// Source projects retain canonical bytes, not every locale's dependency index.
#[derive(Debug, Clone)]
pub struct CatalogResource {
    pub locale: String,
    pub canonical: String,
}

pub struct ValidatedCatalogs {
    pub resources: Vec<CatalogResource>,
    pub arguments: BTreeMap<String, BTreeSet<String>>,
}
#[derive(Default)]
struct Record {
    own: Expansion,
    edges: Vec<Dependency>,
}

struct Dependency {
    target: String,
    bound: BTreeSet<String>,
    numeric: BTreeSet<String>,
    binding_bytes: BTreeMap<String, usize>,
}

#[derive(Default)]
struct AnalysisBudget {
    work: usize,
    bytes: usize,
}
fn invalid(location: &str, message: impl std::fmt::Display) -> BundleError {
    BundleError::Contract(format!("{location}: {message}"))
}

pub fn parse_catalog(locale: &str, source: &str, location: &str) -> Result<Catalog> {
    if source.len() as u64 > MAX_CATALOG_BYTES {
        return Err(BundleError::Limit(format!(
            "{location}: catalog exceeds 16 MiB"
        )));
    }
    // Conservative brace preflight bounds parser recursion before constructing ASTs.
    let mut nesting = 0usize;
    for byte in source.bytes() {
        if byte == b'{' {
            nesting += 1;
            if nesting > MAX_DEPTH {
                return Err(invalid(location, "FTL nesting exceeds 128"));
            }
        }
        if byte == b'}' {
            nesting = nesting.saturating_sub(1);
        }
    }
    let mut resource = parser::parse(source.to_string()).map_err(|(_, errors)| {
        let error = &errors[0];
        let line = source.as_bytes()[..error.pos.start.min(source.len())]
            .iter()
            .filter(|b| **b == b'\n')
            .count()
            + 1;
        invalid(&format!("{location}:{line}"), "malformed FTL")
    })?;
    let mut records = BTreeMap::new();
    resource
        .body
        .retain(|entry| matches!(entry, f::Entry::Message(_) | f::Entry::Term(_)));
    for entry in &mut resource.body {
        let (key, pattern) = match entry {
            f::Entry::Message(v) => {
                if !v.attributes.is_empty() {
                    return Err(invalid(location, "message attributes are unsupported"));
                }
                v.comment = None;
                (
                    v.id.name.clone(),
                    v.value
                        .as_ref()
                        .ok_or_else(|| invalid(location, "message value is required"))?,
                )
            }
            f::Entry::Term(v) => {
                if !v.attributes.is_empty() {
                    return Err(invalid(location, "term attributes are unsupported"));
                }
                v.comment = None;
                (format!("-{}", v.id.name), &v.value)
            }
            _ => unreachable!(),
        };
        if !crate::contract::valid_message_id(key.trim_start_matches('-')) {
            return Err(invalid(location, "invalid or oversized message/term ID"));
        }
        let mut record = Record::default();
        pattern_analysis(pattern, &mut record, location, 1)?;
        if records.insert(key.clone(), record).is_some() {
            return Err(invalid(location, format!("duplicate catalog ID '{key}'")));
        }
        if records.len() > MAX_MESSAGE_IDS {
            return Err(BundleError::Limit(format!(
                "{location}: too many message/term IDs"
            )));
        }
    }
    let mut expanded = BTreeMap::new();
    let mut budget = AnalysisBudget::default();
    for key in records.keys() {
        expand(
            key,
            &records,
            &mut expanded,
            &mut BTreeSet::new(),
            location,
            1,
            &mut budget,
        )?;
    }
    let dependencies = records
        .iter()
        .map(|(key, v)| {
            (
                key.clone(),
                v.edges.iter().map(|edge| edge.target.clone()).collect(),
            )
        })
        .collect();
    resource.body.sort_by_key(|entry| match entry {
        f::Entry::Message(v) => v.id.name.clone(),
        f::Entry::Term(v) => format!("-{}", v.id.name),
        _ => unreachable!(),
    });
    let canonical = serializer::serialize(&resource);
    if canonical.len() as u64 > MAX_CATALOG_BYTES {
        return Err(BundleError::Limit(format!(
            "{location}: canonical catalog exceeds 16 MiB"
        )));
    }
    Ok(Catalog {
        locale: locale.into(),
        canonical,
        messages: expanded
            .into_iter()
            .filter(|(key, _)| !key.starts_with('-'))
            .collect(),
        records: records.into_keys().collect(),
        dependencies,
    })
}

fn pattern_analysis(
    pattern: &f::Pattern<String>,
    record: &mut Record,
    location: &str,
    depth: usize,
) -> Result<()> {
    record.own.depth = record.own.depth.max(depth);
    if depth > MAX_DEPTH {
        return Err(invalid(location, "FTL expression depth exceeds 128"));
    }
    for element in &pattern.elements {
        record.own.work += 1;
        match element {
            f::PatternElement::TextElement { value } => record.own.literal_bytes += value.len(),
            f::PatternElement::Placeable { expression } => {
                // Reserve isolation marks and bounded numeric formatting overhead.
                record.own.literal_bytes += 128;
                expression_analysis(expression, record, location, depth + 1)?;
            }
        }
        bounded(&record.own, location)?;
    }
    Ok(())
}
fn expression_analysis(
    expression: &f::Expression<String>,
    record: &mut Record,
    location: &str,
    depth: usize,
) -> Result<()> {
    record.own.depth = record.own.depth.max(depth);
    if depth > MAX_DEPTH {
        return Err(invalid(location, "FTL expression depth exceeds 128"));
    }
    match expression {
        f::Expression::Inline(v) => inline_analysis(v, record, location, depth + 1),
        f::Expression::Select { selector, variants } => {
            inline_analysis(selector, record, location, depth + 1)?;
            let mut keys = BTreeSet::new();
            for variant in variants {
                let key = match &variant.key {
                    f::VariantKey::Identifier { name } => name.clone(),
                    f::VariantKey::NumberLiteral { value } => {
                        safe_literal_number(value, location)?;
                        value.clone()
                    }
                };
                if !keys.insert(key) {
                    return Err(invalid(location, "duplicate select variant"));
                }
                pattern_analysis(&variant.value, record, location, depth + 1)?;
            }
            Ok(())
        }
    }
}
fn inline_analysis(
    value: &f::InlineExpression<String>,
    record: &mut Record,
    location: &str,
    depth: usize,
) -> Result<()> {
    record.own.depth = record.own.depth.max(depth);
    if depth > MAX_DEPTH {
        return Err(invalid(location, "FTL expression depth exceeds 128"));
    }
    record.own.work += 1;
    use f::InlineExpression as I;
    match value {
        I::StringLiteral { value } => record.own.literal_bytes += value.len(),
        I::NumberLiteral { value } => {
            safe_literal_number(value, location)?;
            record.own.literal_bytes += 128;
        }
        I::VariableReference { id } => {
            *record.own.variables.entry(id.name.clone()).or_default() += 1;
        }
        I::MessageReference {
            id,
            attribute: None,
        } => record.edges.push(Dependency {
            target: id.name.clone(),
            bound: BTreeSet::new(),
            numeric: BTreeSet::new(),
            binding_bytes: BTreeMap::new(),
        }),
        I::TermReference {
            id,
            attribute: None,
            arguments,
        } => {
            let mut names = BTreeSet::new();
            let mut numeric = BTreeSet::new();
            let mut binding_bytes = BTreeMap::new();
            if let Some(arguments) = arguments {
                if !arguments.positional.is_empty() {
                    return Err(invalid(
                        location,
                        "positional term arguments are unsupported",
                    ));
                }
                for arg in &arguments.named {
                    if !names.insert(arg.name.name.clone()) {
                        return Err(invalid(location, "duplicate term argument"));
                    }
                    if matches!(arg.value, I::NumberLiteral { .. }) {
                        numeric.insert(arg.name.name.clone());
                    }
                    if !matches!(arg.value, I::StringLiteral { .. } | I::NumberLiteral { .. }) {
                        return Err(invalid(location, "term parameters must be literal scalars"));
                    }
                    binding_bytes.insert(
                        arg.name.name.clone(),
                        match &arg.value {
                            I::StringLiteral { value } => value.len(),
                            _ => 128,
                        },
                    );
                    inline_analysis(&arg.value, record, location, depth + 1)?;
                }
            }
            record.edges.push(Dependency {
                target: format!("-{}", id.name),
                bound: names,
                numeric,
                binding_bytes,
            });
        }
        I::FunctionReference { id, arguments } if id.name == "NUMBER" => {
            if arguments.positional.len() != 1
                || !matches!(
                    arguments.positional[0],
                    I::VariableReference { .. } | I::NumberLiteral { .. }
                )
            {
                return Err(invalid(location, "NUMBER requires one numeric argument"));
            }
            inline_analysis(&arguments.positional[0], record, location, depth + 1)?;
            if let I::VariableReference { id } = &arguments.positional[0] {
                record.own.numeric_variables.insert(id.name.clone());
            }
            let mut names = BTreeSet::new();
            for arg in &arguments.named {
                if !names.insert(&arg.name.name) {
                    return Err(invalid(location, "duplicate NUMBER option"));
                }
                match (arg.name.name.as_str(), &arg.value) {
                    (
                        "minimumIntegerDigits"
                        | "minimumFractionDigits"
                        | "maximumFractionDigits"
                        | "minimumSignificantDigits"
                        | "maximumSignificantDigits",
                        I::NumberLiteral { value },
                    ) => {
                        let n = value
                            .parse::<u8>()
                            .map_err(|_| invalid(location, "invalid NUMBER precision"))?;
                        if n > 20 {
                            return Err(invalid(location, "NUMBER precision exceeds 20"));
                        }
                    }
                    ("useGrouping", I::StringLiteral { value })
                        if value == "true" || value == "false" => {}
                    _ => return Err(invalid(location, "unsupported NUMBER option")),
                }
            }
        }
        I::Placeable { expression } => {
            expression_analysis(expression, record, location, depth + 1)?
        }
        _ => {
            return Err(invalid(
                location,
                "unsupported function or attribute reference",
            ));
        }
    }
    bounded(&record.own, location)
}
fn safe_literal_number(value: &str, location: &str) -> Result<()> {
    use rust_decimal::Decimal;
    let exact =
        Decimal::from_str_exact(value).map_err(|_| invalid(location, "invalid numeric literal"))?;
    if exact_decimal_number(exact).is_none() {
        return Err(invalid(
            location,
            "numeric literal does not round-trip exactly through f64",
        ));
    }
    Ok(())
}

/// A decimal is exactly binary-representable only if its reduced denominator
/// has no factor 5 and its normalized significand fits the 53-bit f64 mantissa.
/// A rounded Decimal::from_f64 alone is insufficient for tiny decimal values.
pub fn exact_decimal_number(value: rust_decimal::Decimal) -> Option<f64> {
    use rust_decimal::prelude::ToPrimitive;
    let mut mantissa = value.mantissa();
    for _ in 0..value.scale() {
        if mantissa % 5 != 0 {
            return None;
        }
        mantissa /= 5;
    }
    let mut magnitude = mantissa.unsigned_abs();
    while magnitude != 0 && magnitude & 1 == 0 {
        magnitude /= 2;
    }
    if magnitude > (1u128 << 53) - 1 {
        return None;
    }
    let number = value.to_f64()?;
    if !number.is_finite() || rust_decimal::Decimal::from_f64_retain(number)? != value {
        return None;
    }
    Some(number)
}
fn bounded(expansion: &Expansion, location: &str) -> Result<()> {
    if expansion.work > MAX_WORK
        || expansion.literal_bytes > 1 << 20
        || expansion.variables.values().any(|n| *n > MAX_WORK)
    {
        return Err(BundleError::Limit(format!(
            "{location}: Fluent expansion exceeds work/output bounds"
        )));
    }
    Ok(())
}
fn expand(
    key: &str,
    records: &BTreeMap<String, Record>,
    cache: &mut BTreeMap<String, Expansion>,
    visiting: &mut BTreeSet<String>,
    location: &str,
    depth: usize,
    budget: &mut AnalysisBudget,
) -> Result<Expansion> {
    if depth > MAX_DEPTH || !visiting.insert(key.into()) {
        return Err(invalid(
            location,
            "cyclic or overly deep message/term graph",
        ));
    }
    if let Some(value) = cache.get(key) {
        visiting.remove(key);
        return Ok(value.clone());
    }
    let record = records
        .get(key)
        .ok_or_else(|| invalid(location, format!("unknown message/term '{key}'")))?;
    let mut value = record.own.clone();
    for edge in &record.edges {
        let (target, bound, numeric) = (&edge.target, &edge.bound, &edge.numeric);
        let child = expand(
            target,
            records,
            cache,
            visiting,
            location,
            depth + 1,
            budget,
        )?;
        value.depth = value.depth.max(record.own.depth + child.depth);
        if value.depth > MAX_DEPTH {
            return Err(invalid(location, "expanded message/term depth exceeds 128"));
        }
        if target.starts_with('-')
            && child.variables.keys().cloned().collect::<BTreeSet<_>>() != *bound
        {
            return Err(invalid(
                location,
                "term variables require complete literal named bindings",
            ));
        }
        if target.starts_with('-') && !child.numeric_variables.is_subset(numeric) {
            return Err(invalid(
                location,
                "numeric term parameters require numeric literal bindings",
            ));
        }
        value.numeric_variables.extend(
            child
                .numeric_variables
                .into_iter()
                .filter(|name| !bound.contains(name)),
        );
        value.work += child.work;
        value.literal_bytes += child.literal_bytes;
        for (name, count) in child.variables {
            if let Some(bytes) = edge.binding_bytes.get(&name) {
                value.literal_bytes = value
                    .literal_bytes
                    .saturating_add(count.saturating_mul(*bytes));
            } else if !bound.contains(&name) {
                *value.variables.entry(name).or_default() += count;
            }
        }
        bounded(&value, location)?;
    }
    visiting.remove(key);
    budget.work += value.work;
    budget.bytes += key.len()
        + value.variables.keys().map(String::len).sum::<usize>()
        + value
            .numeric_variables
            .iter()
            .map(String::len)
            .sum::<usize>();
    if budget.work > MAX_WORK || budget.bytes as u64 > MAX_CATALOG_BYTES {
        return Err(BundleError::Limit(format!(
            "{location}: aggregate Fluent analysis exceeds work/memory limits"
        )));
    }
    cache.insert(key.into(), value.clone());
    Ok(value)
}

pub fn project_catalogs(
    root: &Path,
    config: &Localization,
    sites: &[MessageSite],
) -> Result<ValidatedCatalogs> {
    if sites.len() > MAX_MESSAGE_IDS {
        return Err(BundleError::Limit("too many source message IDs".into()));
    }
    let directory = catalog_directory(root, &config.root)?;
    validate_layout(&directory, &config.supported_locales)?;
    let mut catalogs = Vec::new();
    let mut total_bytes = 0_u64;
    let mut canonical_paths = BTreeSet::new();
    let mut expected: Option<BTreeMap<String, BTreeSet<String>>> = None;
    for locale in &config.supported_locales {
        let location = format!("{}/{locale}.ftl", config.root);
        let canonical = directory
            .join(format!("{locale}.ftl"))
            .canonicalize()
            .map_err(|_| invalid(&location, "catalog is missing"))?;
        if !canonical_paths.insert(canonical) {
            return Err(invalid(&location, "catalog path aliases another locale"));
        }
        let source = read_catalog(root, &directory.join(format!("{locale}.ftl")), &location)?;
        let catalog = parse_catalog(locale, &source, &location)?;
        check_coverage(
            &catalog,
            sites.iter().map(|s| s.text.id.as_str()),
            &location,
        )?;
        let sets: BTreeMap<_, _> = sites
            .iter()
            .map(|s| {
                (
                    s.text.id.clone(),
                    catalog.messages[&s.text.id]
                        .variables
                        .keys()
                        .cloned()
                        .collect::<BTreeSet<_>>(),
                )
            })
            .collect();
        if expected.as_ref().is_some_and(|expected| expected != &sets) {
            return Err(invalid(&location, "translated argument-name sets differ"));
        }
        for site in sites {
            for name in catalog.messages[&site.text.id].variables.keys() {
                let ty = site.visible_variables.get(name).ok_or_else(|| {
                    invalid(
                        &format!(
                            "{}:{}:{}",
                            site.text.source, site.text.line, site.text.column
                        ),
                        format!(
                            "message '{}' argument '${name}' is out of scope ({location})",
                            site.text.id
                        ),
                    )
                })?;
                if matches!(
                    ty,
                    VarType::ArrayInteger
                        | VarType::ArrayString
                        | VarType::ArrayBoolean
                        | VarType::ArrayDecimal
                ) {
                    return Err(invalid(
                        &location,
                        format!(
                            "message '{}' argument '${name}' cannot be an array",
                            site.text.id
                        ),
                    ));
                }
                if catalog.messages[&site.text.id]
                    .numeric_variables
                    .contains(name)
                    && !matches!(ty, VarType::Integer | VarType::Decimal)
                {
                    return Err(invalid(
                        &location,
                        format!("NUMBER argument '${name}' must have a numeric StoryScript type"),
                    ));
                }
            }
        }
        if expected.is_none() {
            expected = Some(sets);
        }
        total_bytes += catalog.canonical.len() as u64;
        if total_bytes > crate::limits::ResourceLimits::HARD.max_total_uncompressed_bytes {
            return Err(BundleError::Limit(
                "canonical catalogs exceed the 100 MiB total payload ceiling".into(),
            ));
        }
        catalogs.push(CatalogResource {
            locale: catalog.locale,
            canonical: catalog.canonical,
        });
    }
    catalogs.sort_by(|a, b| a.locale.cmp(&b.locale));
    Ok(ValidatedCatalogs {
        resources: catalogs,
        arguments: expected.unwrap_or_default(),
    })
}
pub fn check_coverage<'a>(
    catalog: &Catalog,
    ids: impl IntoIterator<Item = &'a str>,
    location: &str,
) -> Result<()> {
    let mut requested = BTreeSet::new();
    for id in ids {
        if !catalog.messages.contains_key(id) {
            return Err(invalid(location, format!("missing message '{id}'")));
        }
        requested.insert(id.to_string());
    }
    let used = reachable_records(catalog, &requested);
    if used != catalog.records {
        return Err(invalid(
            location,
            format!(
                "unused catalog entries: {}",
                catalog
                    .records
                    .difference(&used)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    Ok(())
}

fn reachable_records(catalog: &Catalog, ids: &BTreeSet<String>) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    let mut queue: Vec<_> = ids
        .iter()
        .filter(|id| catalog.records.contains(*id))
        .cloned()
        .collect();
    while let Some(key) = queue.pop() {
        if used.insert(key.clone()) {
            queue.extend(catalog.dependencies[&key].iter().cloned());
        }
    }
    used
}

pub fn catalog_directory(root: &Path, logical: &str) -> Result<PathBuf> {
    let mut parent = root.to_path_buf();
    for component in Path::new(logical).components() {
        let name = component.as_os_str();
        let exact = fs::read_dir(&parent)
            .map_err(|_| invalid(logical, "localization root is unavailable"))?
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name() == name);
        if !exact {
            return Err(invalid(
                logical,
                "case or Unicode aliased localization root",
            ));
        }
        parent.push(name);
    }
    let path = root
        .join(logical)
        .canonicalize()
        .map_err(|_| invalid(logical, "localization root is missing"))?;
    if !path.starts_with(root) || !path.is_dir() {
        return Err(invalid(
            logical,
            "localization root escapes project or is not a directory",
        ));
    }
    Ok(path)
}
fn validate_layout(directory: &Path, locales: &[String]) -> Result<()> {
    let expected: BTreeSet<_> = locales.iter().map(|s| format!("{s}.ftl")).collect();
    for entry in
        fs::read_dir(directory).map_err(|_| invalid("localization", "cannot enumerate catalogs"))?
    {
        let entry = entry.map_err(|_| invalid("localization", "cannot enumerate catalogs"))?;
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| invalid("localization", "non UTF-8 catalog path"))?
            .to_string();
        if !expected.contains(&name) {
            return Err(invalid(
                "localization",
                format!("unknown or aliased catalog '{name}'"),
            ));
        }
    }
    Ok(())
}
pub fn read_catalog(root: &Path, candidate: &Path, location: &str) -> Result<String> {
    let canonical = candidate
        .canonicalize()
        .map_err(|_| invalid(location, "catalog is missing"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(invalid(
            location,
            "catalog escapes project or is not a file",
        ));
    }
    let file = File::open(canonical).map_err(|_| invalid(location, "catalog cannot be read"))?;
    let mut bytes = Vec::new();
    file.take(MAX_CATALOG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid(location, "catalog cannot be read"))?;
    if bytes.len() as u64 > MAX_CATALOG_BYTES {
        return Err(BundleError::Limit(format!(
            "{location}: catalog exceeds 16 MiB"
        )));
    }
    String::from_utf8(bytes).map_err(|_| invalid(location, "catalog is not UTF-8"))
}

pub fn bind_story(
    story: &mut pb::CompiledStory,
    sites: &[MessageSite],
    catalogs: &ValidatedCatalogs,
    metadata: pb::LocalizationMetadata,
) -> Result<()> {
    let mut contracts: BTreeMap<String, Vec<pb::MessageArgument>> = BTreeMap::new();
    for site in sites {
        let args = catalogs.arguments[&site.text.id]
            .iter()
            .map(|name| pb::MessageArgument {
                name: name.clone(),
                r#type: match site.visible_variables[name] {
                    VarType::Integer => pb::VariableType::Integer,
                    VarType::Decimal => pb::VariableType::Decimal,
                    VarType::String => pb::VariableType::String,
                    VarType::Boolean => pb::VariableType::Boolean,
                    _ => unreachable!("validated scalar"),
                } as i32,
            })
            .collect();
        contracts.insert(site.text.id.clone(), args);
    }
    visit_texts_mut(story, &mut |text| {
        if let Some(pb::story_text::Value::Message(message)) = &mut text.value {
            message.arguments = contracts[&message.id].clone();
        }
    });
    story.localization = Some(metadata);
    Ok(())
}

pub fn visit_texts_mut(story: &mut pb::CompiledStory, f: &mut impl FnMut(&mut pb::StoryText)) {
    fn choices(entries: &mut [pb::ChoiceEntry], f: &mut impl FnMut(&mut pb::StoryText)) {
        for entry in entries {
            match entry.value.as_mut() {
                Some(pb::choice_entry::Value::Option(v)) => {
                    if let Some(t) = &mut v.text {
                        f(t);
                    }
                }
                Some(pb::choice_entry::Value::IfEntry(v)) => {
                    if let Some(b) = &mut v.body {
                        choices(&mut b.entries, f);
                    }
                }
                Some(pb::choice_entry::Value::Repeat(v)) => {
                    if let Some(b) = &mut v.body {
                        choices(&mut b.entries, f);
                    }
                }
                Some(pb::choice_entry::Value::ForSnapshot(v)) => {
                    if let Some(b) = &mut v.body {
                        choices(&mut b.entries, f);
                    }
                }
                _ => {}
            }
        }
    }
    fn statements(stmts: &mut [pb::StoryStatement], f: &mut impl FnMut(&mut pb::StoryText)) {
        for stmt in stmts {
            match stmt.value.as_mut() {
                Some(pb::story_statement::Value::Narration(v)) => {
                    if let Some(t) = &mut v.text {
                        f(t);
                    }
                }
                Some(pb::story_statement::Value::Dialogue(v)) => {
                    if let Some(t) = &mut v.text {
                        f(t);
                    }
                }
                Some(pb::story_statement::Value::Choice(v)) => choices(&mut v.entries, f),
                Some(pb::story_statement::Value::IfElse(v)) => {
                    if let Some(b) = &mut v.then_branch {
                        statements(&mut b.statements, f);
                    }
                    if let Some(b) = &mut v.else_branch {
                        statements(&mut b.statements, f);
                    }
                }
                Some(pb::story_statement::Value::Repeat(v)) => statements(&mut v.body, f),
                Some(pb::story_statement::Value::ForSnapshot(v)) => statements(&mut v.body, f),
                _ => {}
            }
        }
    }
    for scene in &mut story.scenes {
        if let Some(block) = &mut scene.story {
            statements(&mut block.statements, f);
        }
    }
}

pub fn author_command(project_root: &Path, action: &str) -> Result<serde_json::Value> {
    let root = project_root
        .canonicalize()
        .map_err(|_| invalid("StoryScript.toml", "project is unavailable"))?;
    let config = crate::config::ProjectConfig::from_path(&root.join("StoryScript.toml"))?;
    let output =
        storyscript_parser::compiler::compile_project(&root, Path::new(&config.project.entry))
            .map_err(|_| invalid(&config.project.entry, "source cannot be read"))?;
    if output.diagnostics.iter().any(|d| d.is_error()) {
        return Err(BundleError::Compile(
            output
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let mut sites = output.message_sites();
    sites.sort_by(|a, b| a.text.id.cmp(&b.text.id));
    let inventory = sites.iter().map(|site| serde_json::json!({ "id": site.text.id, "source": site.text.source, "line": site.text.line, "column": site.text.column, "scene": site.scene, "kind": site.kind })).collect::<Vec<_>>();
    match action {
        "extract" => Ok(serde_json::json!({ "status": "ok", "messages": inventory })),
        "check" => {
            if let Some(localization) = &config.localization {
                project_catalogs(&root, localization, &sites)?;
            } else if !sites.is_empty() {
                return Err(invalid(
                    "StoryScript.toml",
                    "keyed text requires localization config",
                ));
            }
            Ok(serde_json::json!({ "status": "ok", "message_count": sites.len() }))
        }
        "sync" => {
            use std::io::Write;
            let config = config
                .localization
                .as_ref()
                .ok_or_else(|| invalid("StoryScript.toml", "sync requires localization config"))?;
            let directory = catalog_directory(&root, &config.root)?;
            validate_layout(&directory, &config.supported_locales)?;
            let used: BTreeSet<_> = sites.iter().map(|s| s.text.id.clone()).collect();
            // Build every edit before mutating; conflicts cannot overwrite translations.
            let mut edits = Vec::new();
            let mut reports = Vec::new();
            let mut inputs = Vec::new();
            let mut input_bytes = 0_u64;
            let mut canonical_paths = BTreeSet::new();
            let mut known_arguments: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            let mut conflicts = BTreeSet::new();
            for locale in &config.supported_locales {
                let location = format!("{}/{locale}.ftl", config.root);
                let path = directory.join(format!("{locale}.ftl"));
                let exists = fs::symlink_metadata(&path).is_ok();
                if exists
                    && !canonical_paths.insert(
                        path.canonicalize()
                            .map_err(|_| invalid(&location, "catalog cannot be resolved"))?,
                    )
                {
                    return Err(invalid(&location, "catalog aliases another locale"));
                }
                let source = if exists {
                    read_catalog(&root, &path, &location)?
                } else {
                    String::new()
                };
                let catalog = parse_catalog(locale, &source, &location)?;
                input_bytes += source.len() as u64;
                if input_bytes > crate::limits::ResourceLimits::HARD.max_total_uncompressed_bytes {
                    return Err(BundleError::Limit(
                        "catalog synchronization input exceeds 100 MiB".into(),
                    ));
                }
                for site in &sites {
                    if let Some(message) = catalog.messages.get(&site.text.id) {
                        let args: BTreeSet<_> = message.variables.keys().cloned().collect();
                        if known_arguments
                            .get(&site.text.id)
                            .is_some_and(|prior| prior != &args)
                        {
                            conflicts.insert(site.text.id.clone());
                        }
                        for name in &args {
                            if !matches!(
                                site.visible_variables.get(name),
                                Some(
                                    VarType::Integer
                                        | VarType::Decimal
                                        | VarType::String
                                        | VarType::Boolean
                                )
                            ) {
                                conflicts.insert(site.text.id.clone());
                            }
                            if message.numeric_variables.contains(name)
                                && !matches!(
                                    site.visible_variables.get(name),
                                    Some(VarType::Integer | VarType::Decimal)
                                )
                            {
                                conflicts.insert(site.text.id.clone());
                            }
                        }
                        known_arguments.entry(site.text.id.clone()).or_insert(args);
                    }
                }
                let existing: BTreeSet<_> = catalog.messages.keys().cloned().collect();
                let missing: Vec<_> = used.difference(&existing).cloned().collect();
                let obsolete: Vec<_> = catalog
                    .records
                    .difference(&reachable_records(&catalog, &used))
                    .cloned()
                    .collect();
                inputs.push((location, path, exists, source, missing, obsolete));
            }
            let mut output_bytes = 0_u64;
            for (location, path, exists, source, missing, obsolete) in inputs {
                let mut updated = source.clone();
                if !missing.is_empty() && !updated.is_empty() && !updated.ends_with('\n') {
                    updated.push('\n');
                }
                for id in &missing {
                    let args = known_arguments
                        .get(id)
                        .map(|args| {
                            args.iter()
                                .map(|name| format!(" {{ ${name} }}"))
                                .collect::<String>()
                        })
                        .unwrap_or_default();
                    updated.push_str(&format!("\n# TODO translate {id}\n{id} = {id}{args}\n"));
                    if updated.len() as u64 > MAX_CATALOG_BYTES {
                        return Err(BundleError::Limit(format!(
                            "{location}: synced catalog exceeds 16 MiB"
                        )));
                    }
                }
                if updated.len() as u64 > MAX_CATALOG_BYTES {
                    return Err(BundleError::Limit(format!(
                        "{location}: synced catalog exceeds 16 MiB"
                    )));
                }
                output_bytes += updated.len() as u64;
                if output_bytes > crate::limits::ResourceLimits::HARD.max_total_uncompressed_bytes {
                    return Err(BundleError::Limit(
                        "catalog synchronization output exceeds 100 MiB".into(),
                    ));
                }
                if updated != source {
                    edits.push((path, exists, updated));
                }
                reports.push(serde_json::json!({ "catalog": location, "added": missing, "obsolete": obsolete }));
            }
            for (path, exists, updated) in edits {
                let mut options = fs::OpenOptions::new();
                options.write(true);
                if exists {
                    options.truncate(true);
                } else {
                    options.create_new(true);
                }
                let mut file = options
                    .open(&path)
                    .map_err(|_| invalid("localization", "catalog sync write failed"))?;
                file.write_all(updated.as_bytes())
                    .map_err(|_| invalid("localization", "catalog sync write failed"))?;
            }
            Ok(serde_json::json!({ "status": "ok", "catalogs": reports, "conflicts": conflicts }))
        }
        _ => Err(invalid("localization", "unknown author command")),
    }
}

pub fn catalog_locale(path: &str) -> Result<String> {
    let locale = path
        .strip_prefix(crate::manifest::CATALOG_PREFIX)
        .and_then(|s| s.strip_suffix(".ftl"))
        .ok_or_else(|| invalid(path, "invalid catalog path"))?;
    let tag = locale
        .parse::<icu_locale_core::Locale>()
        .map_err(|_| invalid(path, "invalid catalog locale"))?;
    if tag.to_string().as_str() != locale {
        return Err(invalid(path, "noncanonical catalog locale"));
    }
    Ok(locale.into())
}

pub fn message_contracts(
    story: &pb::CompiledStory,
) -> Result<BTreeMap<String, Vec<pb::MessageArgument>>> {
    validate_message_scopes(story)?;
    let mut contracts = BTreeMap::new();
    let mut invalid_id = false;
    visit_texts_mut(&mut story.clone(), &mut |text| {
        if let Some(pb::story_text::Value::Message(m)) = &text.value {
            invalid_id |= contracts
                .insert(m.id.clone(), m.arguments.clone())
                .is_some();
        }
    });
    if invalid_id || contracts.len() > MAX_MESSAGE_IDS {
        return Err(invalid(
            "compiled/story.pb",
            "duplicate or excessive message IDs",
        ));
    }
    if !contracts.is_empty() && story.localization.is_none() {
        return Err(invalid(
            "compiled/story.pb",
            "message references require localization metadata",
        ));
    }
    Ok(contracts)
}

pub fn validate_catalog_contract(
    catalog: &Catalog,
    contracts: &BTreeMap<String, Vec<pb::MessageArgument>>,
    location: &str,
) -> Result<()> {
    check_coverage(catalog, contracts.keys().map(String::as_str), location)?;
    for (id, args) in contracts {
        let variables: Vec<_> = catalog.messages[id]
            .variables
            .keys()
            .map(String::as_str)
            .collect();
        if variables != args.iter().map(|a| a.name.as_str()).collect::<Vec<_>>() {
            return Err(invalid(
                location,
                format!("message '{id}' variables differ from compiled contract"),
            ));
        }
        for arg in args {
            if catalog.messages[id].numeric_variables.contains(&arg.name)
                && !matches!(
                    pb::VariableType::try_from(arg.r#type),
                    Ok(pb::VariableType::Integer | pb::VariableType::Decimal)
                )
            {
                return Err(invalid(location, "NUMBER argument is not numeric"));
            }
        }
    }
    Ok(())
}

fn validate_message_scopes(story: &pb::CompiledStory) -> Result<()> {
    type Scope = BTreeMap<String, i32>;
    fn prep(stmts: &[pb::PrepStatement], scope: &mut Scope) {
        for stmt in stmts {
            match stmt.value.as_ref() {
                Some(pb::prep_statement::Value::VariableDefinition(v)) => {
                    scope.insert(v.name.clone(), v.r#type);
                }
                Some(pb::prep_statement::Value::IfElse(v)) => {
                    if let Some(b) = &v.then_branch {
                        prep(&b.statements, scope);
                    }
                    if let Some(b) = &v.else_branch {
                        prep(&b.statements, scope);
                    }
                }
                Some(pb::prep_statement::Value::Repeat(v)) => prep(&v.body, scope),
                Some(pb::prep_statement::Value::ForSnapshot(v)) => prep(&v.body, scope),
                _ => {}
            }
        }
    }
    fn loop_scope(scope: &Scope, item: &str, array: &str) -> Result<Scope> {
        let element = match scope
            .get(array)
            .and_then(|t| pb::VariableType::try_from(*t).ok())
        {
            Some(pb::VariableType::ArrayInteger) => pb::VariableType::Integer,
            Some(pb::VariableType::ArrayDecimal) => pb::VariableType::Decimal,
            Some(pb::VariableType::ArrayString) => pb::VariableType::String,
            Some(pb::VariableType::ArrayBoolean) => pb::VariableType::Boolean,
            _ => {
                return Err(invalid(
                    "compiled/story.pb",
                    "invalid snapshot loop variable",
                ));
            }
        };
        let mut inner = scope.clone();
        inner.insert(item.into(), element as i32);
        Ok(inner)
    }
    fn text(text: Option<&pb::StoryText>, scope: &Scope) -> Result<()> {
        if let Some(pb::story_text::Value::Message(message)) = text.and_then(|t| t.value.as_ref()) {
            for arg in &message.arguments {
                if scope.get(&arg.name) != Some(&arg.r#type) {
                    return Err(invalid(
                        "compiled/story.pb",
                        "message argument type/scope differs from declarations",
                    ));
                }
            }
        }
        Ok(())
    }
    fn choices(entries: &[pb::ChoiceEntry], scope: &Scope) -> Result<()> {
        for entry in entries {
            match entry.value.as_ref() {
                Some(pb::choice_entry::Value::Option(v)) => text(v.text.as_ref(), scope)?,
                Some(pb::choice_entry::Value::IfEntry(v)) => {
                    if let Some(b) = &v.body {
                        choices(&b.entries, scope)?;
                    }
                }
                Some(pb::choice_entry::Value::Repeat(v)) => {
                    if let Some(b) = &v.body {
                        choices(&b.entries, scope)?;
                    }
                }
                Some(pb::choice_entry::Value::ForSnapshot(v)) => {
                    if let Some(b) = &v.body {
                        choices(&b.entries, &loop_scope(scope, &v.item_name, &v.array_name)?)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn statements(stmts: &[pb::StoryStatement], scope: &Scope) -> Result<()> {
        for stmt in stmts {
            match stmt.value.as_ref() {
                Some(pb::story_statement::Value::Narration(v)) => text(v.text.as_ref(), scope)?,
                Some(pb::story_statement::Value::Dialogue(v)) => text(v.text.as_ref(), scope)?,
                Some(pb::story_statement::Value::Choice(v)) => choices(&v.entries, scope)?,
                Some(pb::story_statement::Value::IfElse(v)) => {
                    if let Some(b) = &v.then_branch {
                        statements(&b.statements, scope)?;
                    }
                    if let Some(b) = &v.else_branch {
                        statements(&b.statements, scope)?;
                    }
                }
                Some(pb::story_statement::Value::Repeat(v)) => statements(&v.body, scope)?,
                Some(pb::story_statement::Value::ForSnapshot(v)) => {
                    statements(&v.body, &loop_scope(scope, &v.item_name, &v.array_name)?)?
                }
                _ => {}
            }
        }
        Ok(())
    }
    let global: Scope = story
        .initialization
        .as_ref()
        .map(|init| {
            init.variables
                .iter()
                .map(|v| (v.name.clone(), v.r#type))
                .collect()
        })
        .unwrap_or_default();
    for scene in &story.scenes {
        let mut scope = global.clone();
        if let Some(b) = &scene.prep {
            prep(&b.statements, &mut scope);
        }
        if let Some(b) = &scene.story {
            statements(&b.statements, &scope)?;
        }
    }
    Ok(())
}
