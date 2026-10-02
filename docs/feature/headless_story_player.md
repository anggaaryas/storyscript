# Headless Story Player

Audience: application developers integrating StoryScript without adopting a
provided UI, router, state manager, or persistence layer.

## Architecture

One Rust execution model powers source stories and Rust-verified StoryBundles
(strictly authenticated in production or explicitly unsigned for development).
Source AST and verified `CompiledStory` adapters convert into that immutable
model; INIT, PREP, STORY, expressions, choices, jumps, media, limits, history,
RNG, and saves then use the same engine. The legacy source player remains an
adapter that formats scene headers, selected choices, and errors for the TUI.

Semantic responses are compact. An advance or choice returns only the current
event, ordered effects, scene/status, absolute sequence, and history-truncation
metadata. Fetch transcript pages, save bytes, and bundle assets explicitly.
Rust narration/dialogue/choice text now contains a rendered cache plus a plain or
keyed message identity and immutable exact argument snapshots. Bridge DTOs still
carry rendered strings; the host must not parse Fluent or localize story copy itself.

## Source flow

Import `storyscript_player_core.dart`, initialize `RustLib`, then use
`SourceStoryPlayerLoader.openSource`/`openPath`. Restore creates a new player
from the same source/path plus opaque save bytes. Existing generated numeric
session calls remain available for legacy applications.
These Dart source APIs remain raw: keyed text displays IDs with no resolved locale.
Rust catalog-aware callers use `SemanticPlayer::from_project`/`restore_project`
with a project root and ordered preferences. `resolved_locale()` reports the
immutable selected locale. Locale-aware source Dart APIs are deferred to phase 6.

## StoryBundle flow

Import `storyscript_bundle_player.dart`. `StoryBundlePlayerLoader` supports:

- fused bytes/path verification and player construction without transferring a
  `CompiledStory` to Dart;
- fused restore from archive plus opaque save;
- construction or restore from an existing `LoadedStoryBundle` capability;
- multiple isolated players sharing one verified Rust bundle lease.

Rust `_with_locales` bundle open/restore methods negotiate a verified selected
catalog. Existing bundle constructors choose the default locale; Dart preference
DTOs are phase 7. Generic Inspector loading receives default/supported locale
metadata in the generated model but never eagerly copies all catalog bodies.

Disposing a `LoadedStoryBundle` releases that caller's handle. Existing child
players retain the verified archive/model until each player is disposed. The
ordinary `storyscript_bundle.dart` loader and Bundle Inspector still expose the
typed model and do not execute it.

## Playable Flutter example

`storyscript_bundle/example` starts on an illustrated, signed interactive
light-novel chapter. Station Nine's Dot/Neri story has several substantial
routes and endings, conditional choices, illustrated locations, and varied
portrait expressions. The scrollable chapter reader resets to the top on each
new semantic event; it does not store reading position across sessions.
Its `game/story/main.StoryScript` is compiled into a separate fixture and loaded
through the strict `StoryBundlePlayerLoader.openBytes` path; the widget consumes
compact deltas, calls `advance()` or `choose(index)`, lazily reads verified SVG
background/portrait assets, and disposes its player when replaced or removed.
It neither interprets StoryScript in Dart nor falls back to unsigned loading.
The original Bundle Inspector is available from the toolbar with its own
separately trusted fixture. Both fixtures were re-exported for the rewritten v1;
the chapter/UI remain plain English until localization phase 9. See the example
README for run and regeneration instructions.
The sample omits persistent saves and audio; those remain host responsibilities.

## Events, effects, and saves

Events cover scene transition, narration, dialogue, choices, standalone media,
end, and structured error. Scene transitions carry ordered PREP effects:
background, BGM path/stop, and SFX. STORY SFX is emitted as a media event.

A save is allowed only at a stable public boundary after open, advance, choice,
or restore. Restore injects the saved current/pending state: it does **not** run
the current scene's PREP again or re-flatten its STORY. Entering a later scene
works normally. The serializable ChaCha RNG state provides deterministic
continuation, not cryptographic randomness.
Keyed current/pending/choice/history save text stores locale-neutral message IDs
and exact arguments, not rendered prose or a selected locale. Restoring with a
different requested locale rerenders from snapshots without replaying PREP/STORY.
Raw restores retain unresolved ID display. Session locale is never mutable.

Saves require exact schema/runtime/origin and semantic fingerprint identity.
Source and bundle saves are deliberately incompatible. Save bytes are validated
as hostile input but are **not encrypted, signed, authenticated, or anti-cheat**.
The localization rewrite also rejects prior v1 bytes without migration.

## Ownership and exclusions

The host owns UI, plain-text rendering, application-shell localization, storage,
save-slot retention, backup, deletion,
cloud synchronization, authorization, and privacy controls. The SDK supplies no
widgets, routes, storage adapter, migration, registry publication, or latency
guarantee. See `docs/contracts/storyplayer_save_v1.md`, the onboarding checklist,
recovery playbook, and QA plan.
See [story localization](storyscript_localization.md) for project-aware source APIs,
verified catalog negotiation, exact-number guards and locale-neutral save behavior.
