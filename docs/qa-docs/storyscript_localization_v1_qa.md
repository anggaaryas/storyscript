# StoryScript Localization v1 QA Matrix

Scope: implemented phases 1–5; later bridge/editor/app/CI phases are explicitly
deferred. Audience: core maintainers and QA reviewers. No live service/network or
Flutter UI runner is required for the agent-owned core tests.

## Contract-mapped deterministic tests

| Unit / contract | Happy paths | Boundaries and failures | Automated source |
|---|---|---|---|
| Rewritten compiled schema | Locale metadata, plain/message union, ordered scalar contracts | Descriptor drift, prior hash, invalid tags, absent default, no maps/source paths | `bundle/rust/tests/contract_v1.rs` |
| Rewritten save schema | Exact i64/decimal records, locale-neutral message snapshots | Old bare string bytes, arrays, unknown/duplicate variants/arguments; no rendered keyed prose or selected locale | `player/tests/save_contract.rs` |
| Keyed parser | All three sites, legacy interpolation, include inventory/scopes | Empty/invalid/duplicate IDs, interpolation in IDs, excluded expression/actor/asset sites | Parser/validator inline tests; `parser/rust/tests/compiler_project.rs` |
| Strict projects | Complete en/id, plural/select, terms, typed args | Missing/unused/duplicate, scope escape, arrays, argument drift, unsafe functions, cycles/depth/expansion bomb, path aliases/symlink escape | `bundle/rust/tests/project_compile.rs` |
| Source-free IR | Typed refs and metadata, ordered plain segments | No catalogs/prose/source spans inside keyed IR | `ir_conversion.rs` |
| Author CLI | Optional localized init, sorted extract, non-mutating check, append-only/idempotent sync | Conflicts/obsolete entries retained; incomplete check fails; no absolute host paths in inventory | `cli_init.rs`, `cli_export.rs` |
| Canonical archive | Deterministic semantic FTL, signed digests, bounded read | Comment/path exclusion, tampered/missing/extra/aliased catalog, signed noncanonical content | `export_determinism.rs`, `loader_security.rs` |
| Trust/identity | Strict and explicit development validation parity | Old descriptor/compiler rejection, signature-before-identity, unsigned/tamper rejection | `signature_policy.rs`, `version_compatibility.rs` |
| Catalog limits | Lower-only archive ceilings, independent 16 MiB catalog cap | 65 locales, 100,001 IDs, 257-byte ID, output/work/depth/memory bounds | `limits.rs`, `project_compile.rs`, `execution_limits.rs` |
| Immutable locale | en/id rendering, exact/language-only/default fallback, booleans/plurals/terms | Malformed preferences, raw ID/no-locale behavior | `player/tests/runtime_semantics.rs`, `runtime_compatibility.rs` |
| Exactness/atomicity | Safe integers/decimals, opaque isolated strings | `0.1`, tiny decimals, `2^53+1`, `i64::MAX`; resolver/placeable/work/output failures preserve identical save bytes | `execution_limits.rs`, `runtime_semantics.rs` |
| Memory amplification | Shared lexical frames and streaming catalog indexes | Memoized depth, repeated term-literal byte expansion, tiny saves referencing thousands of large translated patterns fail before formatting allocation | `compiler_project.rs`, `project_compile.rs`, `execution_limits.rs` |
| Locale-neutral progress | Current/pending/choices/history rerender, distinct repeated-site snapshots | Changed globals do not recompute snapshots; no current PREP/STORY replay; later PREP executes once | `save_roundtrip.rs` |
| Hostile restore | Valid typed references | Unknown IDs/names/types, arrays/duplicates, old runtime identity, targets/RNG/sequence bounds | `save_security.rs`, `save_contract.rs` |
| Source/bundle parity | Same rendered events in en/id/fallback, cross-locale bundle restore | Distinct origins; old source/bundle saves not interchangeable | `bundle_runtime.rs` |
| Generic Dart model | Generated locale/ref records and plain union | No eager catalog read during model open | `storyscript_bundle/test/protobuf_contract_test.dart`, `story_bundle_loader_test.dart` |
| Existing bridges/TUI | Rendered-string DTOs, verified leases, raw source/save compatibility | No new locale-aware Dart methods claimed; existing lifecycle/error guards preserved | Existing Rust bridge tests, player CLI check |

## Agent-owned commands

Run each plan checkpoint in phase order. Final regression:

```bash
cargo test --manifest-path parser/rust/Cargo.toml
cargo test --manifest-path bundle/rust/Cargo.toml
cargo run --manifest-path bundle/rust/Cargo.toml -- schema check
cargo test --manifest-path player/Cargo.toml --no-default-features --features storybundle-runtime
cargo test --manifest-path storyscript_player_core/rust/Cargo.toml --test player_save_bridge
cargo test --manifest-path storyscript_bundle/rust/Cargo.toml --test bridge_loader --test bridge_player
cargo check --manifest-path player/Cargo.toml
cargo check --manifest-path bundle/rust/fuzz/Cargo.toml
```

From `storyscript_bundle/`: `flutter analyze` and focused non-UI
`flutter test test/protobuf_contract_test.dart test/story_bundle_loader_test.dart`.
Keep pinned Protobuf generation byte-stable on regeneration. Inspect `git diff
--check`, scoped dependency locks, docs and public-key-only fixtures. Compile the
extension and validate its grammar JSON; semantic localization editor tests do not
exist until phase 8.

## Manual/release checklist

- [ ] Review English/Indonesian translations; structural TODO stubs are not proof of quality.
- [ ] Verify unsupported locale fallback is distinguishable from raw unresolved IDs.
- [ ] Confirm Fluent FSI/PDI marks survive host plain-text rendering.
- [ ] Exercise strict catalog tamper rejection before exposing any player/model.
- [ ] Export/re-sign both separately trusted fixtures; compare repeated bytes and discard private keys.
- [ ] Confirm old-v1 rejection/re-export/restart recovery and save PII policies.

## Human-deferred / future-phase coverage

Phases 6–7 add requested/resolved locale DTOs, project Dart loaders and fused bundle
locale preferences, plus focused non-UI bridge tests. Phase 8 adds workspace
completion/definition/references/rename/sync/diagnostics tests. Phase 9 localizes
Station Nine and Flutter shell copy; phase 10 owns CI/release lockstep.

Humans alone create/modify/run game/Inspector widget, route, golden and integration
tests. Acceptance includes candidate save/restore language-switch rollback, no
duplicated PREP/media, both story and shell languages, unsupported device locale,
long Indonesian choices, empty/loading/error/fault states, 200% text, narrow/wide
layout, keyboard/screen reader, 48x48 targets, Inspector metadata, native/Web FFI
and Android/iOS/macOS/Linux/Windows/Web builds. These are not passing core-phase
claims and were not executed by the implementing agent.
