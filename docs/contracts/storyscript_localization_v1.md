# StoryScript Localization v1 Contract

Status: implemented rewritten-v1 Rust core (phases 1–5); checkpoints are tracked in
`docs/plan/20261002001_storyscript_localization_plan.md`. This contract does not
claim that later bridge/editor/UI phases have shipped.

## Ownership and syntax

StoryScript owns narration, dialogue bodies and choice labels. The host owns shell
copy. Actor/project names, IDs, scenes, assets, diagnostics and stored strings are
locale-neutral. Plain quoted text keeps ordered `${variable}` interpolation.
`@"fluent-message-id"` is an ID-only keyed literal at the three story sites, never
an expression or directive. IDs match `[A-Za-z][A-Za-z0-9_-]*`, are at most 256 UTF-8
bytes, and have one declaration across merged root/includes. Empty IDs, escapes
producing invalid IDs, interpolation and excluded sites fail compilation.

## Catalog profile

Pin `fluent = 0.17.0`, `fluent-syntax = 0.12.0`, `unic-langid = 0.9.6` and
`icu_locale_core = 2.1.1` in the
owning Cargo locks. Support messages, terms, opaque variables, message/term
references, select/plural expressions and safe `NUMBER` formatting. Reject custom
functions, `DATETIME`, attributes, malformed syntax, duplicate IDs, unresolved
references, cycles and constructs that cannot be deterministically validated or
bounded. Every source ID exists exactly once in every declared locale; every
catalog record must be used directly or transitively. Every locale agrees on the
transitive argument-name set. Resolve names against each site's visible globals
and scene/loop locals; record immutable scalar type. Reject arrays.
`NUMBER` has one numeric positional variable/literal, optional integer precision
settings bounded at 20, and optional literal `useGrouping` string `true`/`false`.
Reject other options and nonnumeric declared types. Parameterized terms require
complete named literal scalar bindings, including numeric bindings for numeric
term parameters; positional or implicit inherited term-variable bindings are not
supported. Validate both syntax depth and memoized expanded dependency depth at
128, aggregate/per-message analysis work at 100,000, and cached analysis name bytes
at 16 MiB. Conservative all-branch output estimates bound formatting allocation;
the actual output must also fit the host-lowered event cap.

Catalogs live at `<localization-root>/<canonical-locale>.ftl` inside the project.
Absolute/traversing paths, symlink escape, case/Unicode aliases, duplicate canonical
tags and a missing default are invalid. English is the reference default and
Indonesian the reference translation, not a restriction on project languages.

## Signed wire boundary

Compiled story text is exactly one plain ordered interpolation or message reference
(ID plus name-sorted scalar type contracts). Localization metadata holds canonical
default and complete supported locales, never catalog bodies. Semantic identity
includes localization metadata and message contracts. Source spans never enter IR.

One canonical semantic FTL entry per locale lives at
`localization/<canonical-locale>.ftl`. Remove comments and source-only metadata;
sort semantic messages/terms and locale paths deterministically. Index each entry
as manifest type `catalog` with its sizes and SHA-256 under the existing Ed25519
signature. Unknown/unlisted/duplicate/aliased/missing/tampered entries fail closed.
No catalog or player is exposed before complete archive verification, including
catalog validation. Do not extract files. Generic Dart loading receives locale
metadata, not all catalog bodies; Rust players acquire bounded verified leases.

| Resource | Hard maximum |
|---|---:|
| Declared locales | 64 |
| Message IDs | 100,000 |
| Canonical catalog | 16 MiB |
| Message ID UTF-8 bytes | 256 |
| Rendered event | 1 MiB |
| Whole archive / total payload | 100 MiB |
| Any entry | 64 MiB |

The stricter archive, entry and host-lowered limits always apply. Bound parsing,
dependency walks, resolver work and output; do not fetch translations remotely.
Project validation streams one locale's analysis at a time, retains at most 100 MiB
of canonical catalog resources, and shares source lexical scope frames. Runtime
construction retains the selected catalog's index, not every locale's index.

## Execution, locale and exactness

Canonical BCP-47 locale tags (including extensions supported by ICU locale parsing)
use ordered requested preferences: exact
supported match, configured language-only match, then project default. Return the
resolved locale; it is immutable for that session.

ICU locale extensions participate in catalog identity; Fluent receives the language
identifier projection. Extensions do not relax the exact-number guard or provide
custom number-format functions. All requested tags are syntax-validated first.
A raw source/path session has no catalogs and no resolved locale; keyed text
displays its ID verbatim, explicitly
unresolved. It must not claim to have selected a locale.

Capture ordered immutable argument snapshots when flattening each event. Strings
are opaque, booleans select stable `true`/`false` keys. Integers and decimals become
Fluent numbers only when round-tripping through `f64` preserves the exact original
numeric value; retain exact `i64` and decimal mantissa/scale in snapshots and saves.
Reject unsafe numbers with structured `R_LOCALIZATION_NUMBER`. Preserve Fluent
variable isolation. Resolver errors are failures, not partially rendered patterns.
Charge formatting against interaction operations and check the event cap after
formatting. Failure retains the previous stable checkpoint transactionally.
Formatting work includes conservative expanded UTF-8 copy bytes and snapshot
argument bytes/names, not only AST node counts. Restore charges the same aggregate
budget before formatting current/pending/history, preventing small saved references
from expanding into unbounded retained text. A structurally valid large pattern can
therefore fail the interaction work limit even below the 1 MiB event ceiling.

## Locale-neutral saves and replacement

Current/pending/choice/history text is plain text or message ID plus name-sorted
exact typed scalar snapshots. No selected locale or rendered keyed text appears in
save bytes. Restore validates references, names/types/values, known targets and
bounds against loaded resources, negotiates new preferences and rerenders without
PREP/STORY replay. Saves are not encrypted or authenticated; arguments may contain
PII, and hosts own confidential storage, retention and deletion.

Both v1 schemas are deliberately replaced in place. Compiler identity
`0.1.0-localization.1`, runtime identity
`storyscript-player/0.1.0:model-v1-localization`, and pinned descriptor hashes reject
old artifacts. There is no migration: re-export/re-sign/redeploy matching bundles
and restart incompatible progress. Roll back only a complete matching toolchain,
artifact and trust-store set.

## Error boundaries

Syntax/site/duplicate-ID errors are compile-time diagnostics. Project/catalog
validation fails before export; machine diagnostics use project-relative locations,
never host paths. Bundle errors retain existing structured trust/contract/limit
taxonomy. Restore incompatibility differs from corrupt state. Runtime localization
number/resolver/output-limit errors fail atomically. Existing strict signature,
bounded reads, lifecycle leases and no-replay guarantees remain mandatory.
