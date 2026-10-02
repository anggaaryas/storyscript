# StoryScript Story Localization

Audience: authors, translators and Rust/Dart SDK integrators. Phases 1–10 implement
the core, locale-aware source/bundle bridges, workspace editor intelligence and
localized Station Nine reference app. CI/release gates cover the same frozen
contracts; UI/platform acceptance remains human-owned.

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
its own menus/errors/accessibility copy; the example uses Flutter shell `gen_l10n`.

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

### Workspace editor

Open the project workspace in the StoryScript VS Code extension. It observes
source, `StoryScript.toml` and `.ftl` buffers/files, including closed includes and
catalogs. Use keyed-ID / catalog-variable completion, cross-file definition and
references, workspace symbols, atomic message rename and the synchronization
quick fix. Rename edits the unique source token, locale declarations and Fluent
references; it refuses collisions, malformed catalogs and stale disk snapshots.
Open-document edits carry versions. Sync appends TODO stubs (or creates a missing
catalog) without deleting or overwriting translator content. Review the preview.

Diagnostics are explicitly **advisory**. The bounded TypeScript index uses Fluent
AST spans and a lightweight source inventory, not the Rust compiler. Always run
the strict CLI `localize check` before signing; editor success is not release proof.

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

## Dart playback (UI-free)

Initialize the appropriate package's `RustLib` once. For native source projects,
import `package:storyscript_player_core/storyscript_player_core.dart` and use:

```dart
final loader = SourceStoryPlayerLoader();
final player = await loader.openProject('my-story',
  locales: StoryPlayerLocalePreferences(['id-ID', 'en']));
print(player.resolvedLocale); // id
final save = await player.exportSave();
final candidate = await loader.restoreProject('my-story', save,
  locales: StoryPlayerLocalePreferences(['en']));
// Publish candidate before disposing player. On restore failure, keep player.
await player.dispose();
```

`openSource`/`openPath` and their restore variants remain raw: they do not discover
FTL. `player.locale` is an immutable `StoryPlayerLocaleResolution`, with
`resolvedLocale` and `hasUnresolvedLocalization` also available as getters.
Project/path APIs require filesystem access; use bundle bytes on Web.

For verified archives, import
`package:storyscript_bundle/storyscript_bundle_player.dart` and use:

```dart
final loader = StoryBundlePlayerLoader(trustStore: trustStore);
final player = await loader.openBytes(bundleBytes,
  locales: StoryBundlePlayerLocalePreferences(['id-ID', 'en']));
final save = await player.exportSave();
final candidate = await loader.restoreBytes(bundleBytes, save,
  locales: StoryBundlePlayerLocalePreferences(['en']));
await player.dispose(); // only after candidate succeeds and is published
```

The same named `locales` parameter is available on `openPath`, `restorePath`,
`fromBundle` and `restoreFromBundle`. An empty/default preference list selects the
project default; non-localized stories have a null resolved locale. Ordered lists
are copied into unmodifiable preference models; Rust validates every tag and
performs exact/language-only/default negotiation. `player.locale` is an immutable
`StoryBundlePlayerLocaleResolution`. No locale setter or global locale exists.

Fused bundle routes return no compiled model or catalog bodies to Dart. Existing
verified-bundle routes retain their own Rust lease, so disposing the Inspector's
parent handle does not invalidate child players. All event/choice/history `text`
fields are rendered plain text; IDs and argument snapshots stay inside Rust and
opaque saves. Dart neither parses FTL nor reinterprets Fluent isolation marks.

Custom bundle bindings may opt into `LocaleAwareStoryBundlePlayerBindings`.
Legacy `StoryBundlePlayerBindings` implementations remain usable with default
preferences; explicit nonempty preferences fail with `R_LOCALIZATION_BINDINGS`
rather than being silently ignored. FFI bindings implement the locale capability.
Custom source bindings implement `openProject`/`restoreProject` as well as the
existing raw operations. Rebuild native/Web bridge artifacts with the generated
outputs: source FRB is pinned to 2.12.0 and bundle FRB to 2.13.0.

## Exact progress and limits

### Reference app

Station Nine ships 86 keyed sites, complete English/Indonesian catalogs, a resident
count select and stability interpolation in its separately signed fixture. The
Inspector fixture retains its distinct existing rewritten-v1 trust identity.
Flutter ARBs localize the shell of both routes, never the story delta text.
An app-owned process locale initializes from ordered platform preferences; the
selector commits shell locale only after `GameController` restores a saved
candidate using the same verified bundle. Failure retains event, progress, locale
and artwork. Successful restore preserves artwork without PREP/media replay.
No persistent preference or live session locale mutation is added.

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
