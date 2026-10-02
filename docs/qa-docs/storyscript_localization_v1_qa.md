# StoryScript Localization v1 QA Matrix

Scope: implemented phases 1–10; Flutter UI/platform acceptance is explicitly
human-owned. Audience: core/SDK maintainers and QA reviewers. No live service/network or
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
| Source project bridge | Ordered id-ID/en and en/id, unsupported fallback, cross-locale current/pending/history | Malformed tags, raw source/path open/restore ID/null locale/unresolved flag, unsafe numbers/no candidate or byte-identical prior checkpoint, lower-only limits, legacy lifecycle | `storyscript_player_core/rust/tests/player_save_bridge.rs` |
| Source Dart facade | Project forwarding, immutable preferences/resolution, copied saves, candidate restore | Structured failure keeps old checkpoint, raw unresolved flag, unchanged disposal | `storyscript_player_core/test/player_save_contract_test.dart` (injected bindings, not native FFI) |
| Bundle bridge | All six construction routes, en/id/fallback, current/pending/history rerender, default plain bundles | Catalog tamper in strict/development policies exposes no player; malformed preferences; child/parent disposal orders; no public catalog asset/read or eager compiled-prose transfer | `storyscript_bundle/rust/tests/bridge_loader.rs`, `bridge_player.rs` |
| Bundle Dart facade | All six locale-forwarding routes, rendered progression/history, immutable metadata, copied save/archive buffers | Stale candidate cleanup, failed candidate rollback, preflight archive on restore, explicit preferences on legacy bindings fail | `storyscript_bundle/test/player_api_test.dart`, `player_progression_test.dart`, `player_save_test.dart`, `player_lifecycle_test.dart`, `test_support.dart` (injected bindings) |
| Workspace editor | en/id multi-file AST index, completion, definitions/references, symbols, atomic rename and append-only sync | Malformed/profile/drift/scope/array/cycle diagnostics, canonical tags, traversal/symlinks, duplicate IDs, stale versions/disk changes, include changes, missing catalog creation, legacy symbols | `tool/vscode-storyscript/server/test/localization.test.ts`; `npm test`, `npm run compile`, `npm run package` |
| Reference controller | Same-checkpoint rerender, requested/resolved state, same verified lease, unchanged artwork/no media reads | Failed-switch rollback preserves player/locale/delta/artwork; busy guards and disposal | `storyscript_bundle/example/test/game_controller_locale_test.dart` (non-UI/injected bindings) |
| Signed Station Nine | All five branches in en/id/fallback, count select, stability snapshots, source/bundle parity, cross-locale choices and byte-identical saves | English branch/asset regressions, separate Inspector signature, no rendered keyed save prose | `player/tests/bundle_runtime.rs`; CLI strict check/verify and deterministic repeat export |
| Release/static gates | Required docs/phrases, pinned Fluent in all owning locks, descriptors, complete shell ARB keys/placeholders, separate trust identities | Stale feature claims, missing contracts, dependency drift or obsolete vulnerable optional rkyv lock entries | `tests/localization_gates.py`, Cargo audit/deny, npm audit, workflow validation |

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
`flutter test test/protobuf_contract_test.dart test/story_bundle_loader_test.dart test/player_api_test.dart test/player_progression_test.dart test/player_save_test.dart test/player_lifecycle_test.dart`.
From `storyscript_player_core/`: `flutter analyze && flutter test test/player_save_contract_test.dart`.
Keep pinned Protobuf generation byte-stable on regeneration. Inspect `git diff
--check`, scoped dependency locks, docs and public-key-only fixtures. Compile the
extension with `npm --prefix tool/vscode-storyscript test && npm --prefix tool/vscode-storyscript run compile`;
package with `npm --prefix tool/vscode-storyscript run package` and audit its lockfile.

## Bridge review checklist

- [ ] Compare initial resolution with request order, language-only and default fallback;
  distinguish raw IDs from localized output and null plain-story metadata.
- [ ] Confirm generated source FRB 2.12.0 and bundle FRB 2.13.0 outputs are
  byte-identical on a second regeneration; rebuild native/Web binaries separately.
- [ ] Keep catalogs behind verified leases, not assets, model bodies or Dart parsers.
- [ ] Verify save/restore candidate sequence/history/pending choices and retain the
  old session on failure; never mutate locale or replay PREP/STORY.
- [ ] Preserve copied buffers, lower-only limits, structured codes, idempotent
  disposal and stale-result cleanup without touching human-owned UI tests.

### 2026-10-02 Phases 6–7 execution record

- Source Rust bridge: 5 tests passed; source non-UI Dart: 4 passed.
- Bundle Rust bridge/loader: 10 tests passed; focused player Dart: 14 passed,
  or 20 with Protobuf and generic-loader regression tests.
- Both packages' `flutter analyze` reported no issues. Full shared player
  `--no-default-features --features storybundle-runtime`: 47 tests passed;
  `schema check` passed with the existing rewritten descriptor.
- Pinned source FRB 2.12.0 and bundle FRB 2.13.0 outputs were byte-identical
  on repeat generation. Bridge Rust suites also passed after regeneration.
- Supplemental `cargo clippy --all-targets -- -D warnings` is **not green**:
  each bridge has five existing `result_large_err` diagnostics in unchanged
  action closures, lock-result and limit-conversion helpers. No suppression or
  unrelated shared error-type refactor was introduced. Required phase checkpoints
  do not include this supplemental lint gate.
- No widget/golden/integration test was modified or run. Native/Web FFI artifact
  builds and platform acceptance remain human-owned, not implied by these results.

## Manual/release checklist

### 2026-10-02 Phases 8–10 execution record

- Full required Rust matrix passed: parser 64, archive/toolchain 65, player 48,
  source bridge 5 and bundle bridge 10 tests (**192 total**); CLI help also passed.
- Editor: `npm ci`, ten deterministic tests, compile and VSIX package passed;
  `npm audit` reports zero vulnerabilities. The English/Indonesian root/include
  fixture also passed the authoritative Rust catalog check (2 messages).
- Source Dart: 4 focused tests; bundle Dart: 20 focused tests plus 10 other non-UI
  loader/trust/assets/Web-destination regressions; reference controller: 3 passed.
  All three Flutter analyses and `gen-l10n` passed. No UI runner was invoked.
- Station Nine: strict catalog check (86 messages), repeat signed-byte equality and
  strict verification passed. Game SHA-256 is
  `94f40965c83071dd5e13eb5718fd0aa40e08eb9f5f865a4edc316da3000e60e6`.
  Its external ephemeral private key was deleted. Inspector independently verified
  with its existing Phase 4 rewritten-v1 artifact/key; no additional rebuild needed.
- Both pinned FRB generators and Dart Protobuf regenerated with no owned diff;
  both descriptor contracts/schema check passed. Repeated ARB generation was
  byte-identical. Parser formatting debt from Phase 2 was normalized without
  semantic changes; parser/bundle fmt and bundle Clippy `-D warnings` passed.
- Installer smoke tests, fuzz compile, static localization assertions,
  `git diff --check` and actionlint 1.7.12 workflow validation passed.
- Audits for all five owning Rust locks exit successfully; bundle cargo-deny
  advisories/bans/licenses/sources pass. Expanded auditing initially detected
  RUSTSEC-2026-0235 in inactive optional `rkyv 0.7.46` entries of player/source
  locks. A targeted compatible `rust_decimal 1.41.0 → 1.43.0` lock update removed
  those entries, matching the archive toolchain; regressions were rerun.
- **Not warning-free:** existing audit warnings cover unmaintained/yanked and
  unsoundness advisories (`paste`, `adler`, `anyhow`, `tokio`, `lru`, `chacha20`,
  `futures-util`, depending on the owning lock). Cargo-deny retains duplicate/
  unused-license-allowance warnings; VSIX reports an unbundled-file-count warning.
  No advisory/warning suppression was added. Existing bridge-only supplemental
  `result_large_err` Clippy debt from Phases 6–7 remains outside required gates.
- Hosted CI/release dispatch and all Flutter widget/golden/integration/platform
  work remain unexecuted and human-owned. Existing UI tests are byte-unchanged.

- [ ] Review English/Indonesian translations; structural TODO stubs are not proof of quality.
- [ ] Verify unsupported locale fallback is distinguishable from raw unresolved IDs.
- [ ] Confirm Fluent FSI/PDI marks survive host plain-text rendering.
- [ ] Exercise strict catalog tamper rejection before exposing any player/model.
- [ ] Export/re-sign both separately trusted fixtures; compare repeated bytes and discard private keys.
- [ ] Confirm old-v1 rejection/re-export/restart recovery and save PII policies.

## Human-deferred acceptance

Phases 6–7 cover project Dart loaders, requested/resolved locale DTOs and all bundle
routes through real Rust and injected non-UI Dart tests. Phase 8 tests workspace
completion/definition/references/rename/sync/diagnostics, including incomplete
resources and version/disk freshness. Phase 9 localizes Station Nine and Flutter
shell copy; phase 10 gates contracts/catalogs/editor/dependencies and non-UI tests
in CI/release. These implementation results do not imply hosted/platform acceptance.

Humans alone create/modify/run game/Inspector widget, route, golden and integration
tests. Acceptance includes candidate save/restore language-switch rollback, no
duplicated PREP/media, both story and shell languages, unsupported device locale,
long Indonesian choices, empty/loading/error/fault states, 200% text, narrow/wide
layout, keyboard/screen reader, 48x48 targets, Inspector metadata, native/Web FFI
and Android/iOS/macOS/Linux/Windows/Web builds. These are not passing core-phase
claims and were not executed by the implementing agent.
