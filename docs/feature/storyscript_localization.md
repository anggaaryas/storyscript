# StoryScript Story Localization

Audience: authors, translators and Rust SDK integrators. Phases 1–5 are the core
implementation; locale-aware Dart bridges, workspace editor intelligence and the
localized Flutter example are later phases, not shipped capabilities here.

## What is localized

Use `@"stable-message-id"` for narration, dialogue bodies and choice labels:

```text
* INIT {
    $name as string = "Ada";
    @actor A "Actor";
    @start main;
}
* main {
    #STORY
    @"welcome";
    A: @"actor-greeting";
    @choice { @"continue-choice" -> main; }
}
```

An ID has one declaration across root/includes. It is not translated prose, an
expression or an interpolation template. Plain strings retain `${variable}`
interpolation. Actor/project names, scene/actor identifiers, media paths,
diagnostics and string-variable data remain locale-neutral. The host localizes
its own menus/errors/accessibility copy; Flutter shell `gen_l10n` work is phase 9.

## Project layout

```text
StoryScript.toml
story/main.StoryScript
assets/
localization/en.ftl
localization/id.ftl
```

Pin `[project].compiler-version = "0.1.0-localization.1"` and add:

```toml
[localization]
default-locale = "en"
supported-locales = ["en", "id"]
root = "localization"
```

Locale tags and paths must be canonical; absolute/traversing paths, aliases and
symlink escapes fail. Every configured locale requires one FTL catalog. Catalog
messages and terms must all be used directly or through references; every source
ID must exist exactly once in every locale. Every locale agrees on its transitive
argument names, resolved against the site's visible scalar variables.

```ftl
# en.ftl
welcome = Welcome, { $name }.
actor-greeting = Hello!
continue-choice = Continue
```

```ftl
# id.ftl
welcome = Selamat datang, { $name }.
actor-greeting = Halo!
continue-choice = Lanjutkan
```

Terms, references and plural/select expressions are supported. `NUMBER` takes one
numeric variable/literal and bounded precision options. Attributes, `DATETIME`,
custom functions, cycles and unbounded/deep expansion are rejected. Parameterized
terms require complete named literal scalar bindings; term variables do not
silently inherit StoryScript globals. The catalog analyzer conservatively sums
all branches and bounds expanded depth/work/analysis memory.

## Author commands

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- init ./my-story \
  --id example.localized --name "My Story" --localized
cargo run --manifest-path bundle/rust/Cargo.toml -- localize extract --project ./my-story --json
cargo run --manifest-path bundle/rust/Cargo.toml -- localize sync --project ./my-story --json
cargo run --manifest-path bundle/rust/Cargo.toml -- localize check --project ./my-story --json
```

`extract` emits ID-sorted source-relative inventory with scene/site/spans, without
requiring complete catalogs. `sync` appends missing TODO stubs and preserves
existing translator text/comments; it reports obsolete records and conflicting
contracts without deleting or overwriting them. Stubs can carry known arguments.
Review and translate all TODOs: structural `check` cannot judge translation quality.
`check` is non-mutating and validates the strict catalog release gate. Export runs
the same validation before packaging canonical comment-free signed catalogs.
Minimal non-localized `init` remains available without `--localized`.

## Rust playback

```rust
use std::path::Path;
use storyscript_player::{SemanticPlayer, contract::HARD_LIMITS};

let preferences = vec!["id-ID".to_string(), "en".to_string()];
let player = SemanticPlayer::from_project(Path::new("my-story"), &preferences, HARD_LIMITS)?;
assert_eq!(player.resolved_locale(), Some("id"));
let save = player.export_save()?;
let english = SemanticPlayer::restore_project(
    Path::new("my-story"), &save, &["en".to_string()], HARD_LIMITS,
)?;
```

With `storybundle-runtime`, use `from_loaded_bundle_with_locales` and
`restore_loaded_bundle_with_locales` on a completely verified `LoadedBundle`.
Existing bundle constructors select the default. Negotiation processes each
requested BCP-47 locale tag in order: exact match, a configured
language-only match, then project default if no request matches. Malformed tags
and underscore spellings fail. Locale is immutable; change language by restoring
a save into a new candidate and swapping only after success.

Valid ICU locale extensions participate in exact catalog identity and language-only
fallback; Fluent formats using the tag's language identifier and its safe NUMBER
options, not arbitrary extension-based number-format customization.

Raw `from_source`/`from_file` and legacy `StoryPlayer` do not discover catalogs:
they display IDs and return `resolved_locale() == None` with
`has_unresolved_localization() == true`. This is deliberately not translated output.
Current Dart source APIs remain raw until phase 6; locale preferences for Dart
bundle players arrive in phase 7. Do not invent corresponding Dart methods yet.

## Exact progress and limits

Rust events contain a rendered cache plus an optional immutable message snapshot.
Hosts/bridge DTOs receive rendered strings. Keyed saves contain only message ID
and ordered exact scalar snapshots, including current/pending/choices/history;
neither selected locale nor rendered keyed prose is persisted. Restore validates
those contracts and rerenders without replaying PREP/STORY. Loop-generated events
retain distinct argument snapshots after iterators disappear.

Integer and decimal values must be exactly representable in Fluent's `f64`:
`0.5`, `0.125` and `2^53` are safe; `0.1`, `2^53 + 1`, and `i64::MAX` are not.
Some larger powers of two are safe, so this is not merely an integer-range check.
Decimals keep their original mantissa/scale in saves. Boolean selectors use stable
`true`/`false`; strings are opaque and retain Fluent isolation marks.

Limits: 64 locales, 100,000 message/term IDs, 16 MiB per catalog, 256 bytes per
message ID, 1 MiB rendered event; existing 100 MiB archive/total and 64 MiB entry
ceilings still apply. Conservative expansion preflight prevents oversized resolver
allocations; actual formatted output is also checked against host-lowered limits.
Formatting work is charged to interactions and restore. Resolver, numeric and
limit failures leave the prior stable checkpoint intact, never partial patterns.
The work charge includes conservative UTF-8 copy/argument bytes, so the 100,000
interaction operation budget can reject a large translation before its allocation,
even when the single-event 1 MiB ceiling would permit it.

Both v1 descriptors were replaced: old bundles/saves are rejected with **no
migration**. Re-export/re-sign bundles and restart incompatible progress. Saves
are **not encrypted**; argument snapshots can contain PII. See the
[contract](../contracts/storyscript_localization_v1.md),
[checklist](../onboarding/localized_story_project_checklist.md),
[recovery playbook](../playbook/localization_release_recovery.md) and
[QA matrix](../qa-docs/storyscript_localization_v1_qa.md).
