# StoryPlayer Save v1 QA Checklist

Status: **agent-owned Rust and non-UI Dart checkpoints implemented**. Audience:
runtime and bridge QA reviewers. No live services or UI runner are required.

## Localization core update (2026-10-02)

Use [localization QA](storyscript_localization_v1_qa.md) for rewritten-v1 text
unions/exact scalar snapshots, numeric guards, immutable negotiation, current/
pending/choice/history cross-locale rerendering, hostile message IDs/arguments,
raw ID display, atomic limits/resolver failures, and no PREP/STORY replay.
Old-v1 saves/bundles are rejected without migration. Existing bridge compatibility
is covered; new locale-aware Dart/editor/UI tests are later plan phases and all
Flutter widget/golden/integration/platform work remains human-owned.

## Contract checkpoint

| Area | Happy path | Boundary or failure | Automated test |
| --- | --- | --- | --- |
| Descriptor | Pinned generated SHA-256, version 1 | Schema drift fails | `player/tests/save_contract.rs::descriptor_is_pinned_and_versions_are_explicit` |
| Layout | Every required progress/origin/event/RNG/effect field | No embedded source, CompiledStory, maps | `descriptor_requires_all_progress_fields_without_static_story` |
| Values | Signed i64, scaled decimal, typed bool array, ordered globals/locals | No lossy JSON or reordered records | `ordered_records_and_exact_scalars_roundtrip` |
| Value variants | Four scalar types and four typed array families | Scalar and array preservation through strict decode | `all_scalar_and_array_value_types_survive_wire_roundtrip` |
| Semantics | Each event, media, signed/development origin distinguished | Cross-origin saves never interchangeable | `every_event_effect_and_origin_has_a_distinct_variant` |
| Limits | Hard maxima and permitted lowered cap | Zero or raised caps rejected | `limits_are_lower_only_and_no_source_story_is_embedded` |
| Wire preflight | Valid canonical protobuf accepted | Oversize, truncated, unknown field/enum/oneof, duplicate variable/history, invalid decimal and noncanonical RNG position rejected | `contract_decoder_rejects_unknown_required_and_duplicate_records` |

Run `cargo test --manifest-path player/Cargo.toml --no-default-features --test save_contract`.
Wire tests verify schema-level invariants; `save_security.rs` adds story-dependent
validation and atomic candidate construction for external bytes.

## Shared runtime regression

| Unit | Happy path | Boundary/failure | Automated test |
| --- | --- | --- | --- |
| Session RNG | Same seed covers INIT/PREP and later scene transitions | Restored stream/position reproduces next samples; unsupported algorithm and out-of-range word position rejected | `player/tests/runtime_semantics.rs::seed_covers_init_prep_and_subsequent_scene_entries`, `rng_state_reconstructs_stream_and_rejects_unknown_version` |
| Engine semantic stream | Scene event with ordered PREP bg/BGM/SFX, STORY SFX, dialogue portrait | BGM STOP is distinct from no effect; no legacy scene-header narration or history payload in a semantic engine step | `player/tests/runtime_semantics.rs::semantic_stream_resolves_prep_effects_story_sfx_and_portrait_without_tui_headers`, `stopping_bgm_is_a_distinct_effect_not_a_missing_effect` |
| Legacy source player | Scene headers, choice markers, end and history | Invalid choice does not advance; structured error maps back to narration | `player/tests/runtime_compatibility.rs::scene_headers_choice_markers_history_and_end_are_still_legacy_events`, `structured_errors_are_formatted_only_for_legacy_callers` |
| Compact source session | Source compiler, ordered semantic scene/narration/choices, sequence-aware bounded page reads | Invalid choice/advance and end preserve current; history oldest-entry truncation surfaced | `player/tests/runtime_compatibility.rs::semantic_source_constructor_compiles_and_keeps_choices_structured`, `player/tests/runtime_semantics.rs::compact_session_pages_bounded_history_and_never_includes_transcript_in_delta`, `player/tests/execution_limits.rs::history_retains_a_bounded_suffix_with_absolute_sequences_and_pages` |
| Interaction budgets | Checked engine open/choice, operation, logic depth, pending events, rendered text, arrays, lowered history | No candidate escapes on failed open; failed choice preserves globals/scene/RNG; oversized entries and invalid limit configs fail/evict predictably | `player/tests/execution_limits.rs` |

Run `cargo test --manifest-path player/Cargo.toml --no-default-features --test runtime_semantics --test runtime_compatibility --test execution_limits` and `cargo test --manifest-path player/Cargo.toml --test cli_help` for the shared runtime checkpoint.

## Completed regression inventory

- [x] Source AST and verified bundle produce the same semantic event/effect stream.
- [x] INIT/PREP/STORY, logic/loops/collections/interpolation, choices, jumps,
      media/SFX, and end/errors retain correct ordering and legacy TUI output.
- [x] Stable boundary save/restore resumes pending events and RNG without
      rerunning current PREP/STORY; later scenes enter normally.
- [x] Unknown version/origin/fingerprint, malformed/oversized, duplicate,
      invalid target, array, decimal, enum, RNG, and history-sequence saves are
      rejected before exposing a player.
- [x] Lowered operation/depth/queue/array/render/history/save limits either
      roll back the interaction or evict oldest history with visible truncation.
- [x] Source and verified-bundle FFI return compact deltas independent of history
      length, support paginated history, and dispose repeated/parent-first safely.
- [ ] Human-owned native/Web Flutter integration and platform matrix remain
      deferred as specified in the governing plan; do not run widget/golden tests
       as a prerequisite for these Rust contract tests.

## Checkpoint commands

```bash
cargo test --manifest-path player/Cargo.toml --no-default-features --features storybundle-runtime
cargo test --manifest-path storyscript_player_core/rust/Cargo.toml
cargo test --manifest-path storyscript_bundle/rust/Cargo.toml
(cd storyscript_player_core && flutter analyze && flutter test test/player_save_contract_test.dart)
(cd storyscript_bundle && flutter analyze && flutter test \
  test/player_api_test.dart test/player_progression_test.dart \
  test/player_save_test.dart test/player_lifecycle_test.dart)
```

The Rust save tests cover exact source/bundle identity, no-PREP replay,
deterministic continuation, malformed input, declarations/types/targets/RNG,
history, limits, and atomic failure. Bridge tests cover compact DTOs, fused
verified loading, leases, disposal order, assets, and no player exposure after
verification failure. Dart tests map those contracts through injected bindings.
