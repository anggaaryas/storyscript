# StoryBundle v1 QA Plan

Scope: parser safety, Rust format/export/load, Flutter FFI/public API, Bundle
Inspector, Web, and cross-platform packaging (Phases 1–9).

## Localization core update (2026-10-02)

Both v1 descriptors are replaced; prior dated execution records below are historical.
Use [localization QA](storyscript_localization_v1_qa.md) for the current contract:
complete en/id catalogs, canonical signed entries, tamper/missing/extra/alias/limit
rejection, generic Dart metadata without eager body transfer, source/bundle locale
parity and cross-locale saves. Both independent signed fixtures were re-exported.
Locale-aware Dart/editor/UI phases are deferred; all widget/golden/integration and
platform rows remain human-owned and are not part of this core run.

## Automated checklist

| Area | Scenarios | Test source |
| --- | --- | --- |
| Parser boundary | Include order; absolute/parent/symlink/UTF-8/case rejection; INIT diagnostics; numeric overflow | `parser/rust/tests/compiler_project.rs`, lexer unit tests |
| Contract | Descriptor hash; exact compiler pin; required oneofs; decimal scale; AST families; no source fields | `bundle/rust/tests/contract_v1.rs` |
| Project/IR | Root and child compile; interpolation segments; advanced AST conversion; source metadata stripped | `project_compile.rs`, `ir_conversion.rs` |
| Assets | Literal/dynamic closure; sorted output; missing/unmatched; traversal/case/symlink failures | `asset_discovery.rs` |
| Export | Byte determinism; fixed ZIP metadata; complete manifest; no source; extension/key safeguards | `export_determinism.rs`, `signing.rs`, `cli_export.rs` |
| Loader | Valid strict round-trip; traversal/absolute/duplicate/case/unknown entries; malformed manifest/Protobuf/semantics | `loader_security.rs` |
| Trust/tamper | Unknown signer; wrong signature; manifest/asset tamper; strict default; explicit unsigned status | `signature_policy.rs` |
| Compatibility | Authenticated format/compiler/schema mismatch; signature checked before identity | `version_compatibility.rs` |
| Limits | Host-lower-only; archive/entry/profile/semantic-depth boundaries | `limits.rs` |
| CLI initialization | Minimal project compiles; required metadata; TOML escaping; JSON success; existing file/directory/symlink rejection | `cli_init.rs` |
| CLI verification | Untrusted inspect; strict verify; wrong key; stable JSON/non-zero failures | `cli_verify.rs` |
| Fuzz compile gate | Bounded archive envelope, loader, and Protobuf decode entry points | `bundle/rust/fuzz/fuzz_targets/archive_and_protobuf.rs` |
| FFI lifecycle | Strict/development policy; invalid trust bytes; archive preflight; opaque ownership; bounded read; repeated disposal | `storyscript_bundle/rust/tests/bridge_loader.rs` |
| Verified player handoff | Fused/existing-resource creation; parent/child lease order; compact progression; save/restore; bounded assets; no player after verification failure | `storyscript_bundle/rust/tests/bridge_player.rs` |
| Dart bridge/API | DTO mapping; strict default; full Protobuf decode; stale completion; exception families; lower limits | `storyscript_bundle/test/*_test.dart` |
| Dart assets | Normalized/missing paths; lazy copies; caller limits; repeated reads; disposal | `storyscript_bundle/test/asset_access_test.dart` |
| Dart headless player | Fused API, compact events, explicit history, opaque saves, stale cleanup, parent-first lifecycle | `storyscript_bundle/test/player_*_test.dart` |
| Inspector | Empty/loading/verified/failure/disposed; stale-content clearing; compact/wide; semantics; keyboard; 48px controls; preview | `storyscript_bundle/example/test/bundle_inspector_screen_test.dart` |
| Game widget | Branching navigation, image reads, retry after verification failure, restart/resource disposal, removal during pending open, inspector navigation | `storyscript_bundle/example/test/game_screen_test.dart` |
| Game runtime | Signed archive verifies; patch/search choices have different available options and reach distinct endings | `player/tests/bundle_runtime.rs` |
| Native integration | Signed Rust load; Dart decode; asset read; tamper rejection; disposal; signed game branch and unknown signer | `storyscript_bundle/example/integration_test/storybundle_loading_test.dart` |
| Web/platform | Wasm generation, COOP/COEP integration, Android/iOS/macOS/Linux/Windows/Web builds | `.github/workflows/storybundle-ci.yml` |

## Required commands

```bash
cargo test --manifest-path parser/rust/Cargo.toml
cargo check --manifest-path player/Cargo.toml --no-default-features
cargo fmt --manifest-path bundle/rust/Cargo.toml -- --check
cargo clippy --manifest-path bundle/rust/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path bundle/rust/Cargo.toml
cargo run --manifest-path bundle/rust/Cargo.toml -- schema check
cargo check --manifest-path bundle/rust/fuzz/Cargo.toml
cargo check --manifest-path storyscript_player_core/rust/Cargo.toml
cargo test --manifest-path storyscript_bundle/rust/Cargo.toml
(cd storyscript_bundle && flutter analyze && flutter test)
(cd storyscript_bundle/example && flutter analyze && \
  flutter test test)
(cd storyscript_bundle/example && \
  flutter test integration_test/storybundle_loading_test.dart -d macos)
```

## Manual acceptance

- [ ] Initialize a new path with explicit `--id` and `--name`; compile the generated project.
- [ ] Confirm initialization rejects existing empty/non-empty directories, files,
  and symlinks without changing their contents.
- [ ] Confirm the generated project retains `assets/` after source-control checkout.
- [ ] Generate an external ephemeral key and export the canonical fixture twice.
- [ ] Confirm outputs are byte-identical and use `.storybundle`.
- [ ] List ZIP entries; confirm no source/source-map/comment/host path/private key.
- [ ] Strictly verify with the public key; inspect is visibly untrusted.
- [ ] Tamper separately with canonical manifest, compiled payload, and one asset.
- [ ] Confirm no model or asset API is returned on any failed load.
- [ ] Confirm lazy asset reads reject missing paths and caller limits before allocation.
- [ ] Exercise exact compiler/schema mismatch and unknown signer recovery guidance.
- [ ] Exercise dynamic glob closure and literal/dynamic missing-asset failures.
- [ ] Exercise archive, uncompressed, entry, IR, manifest, count, path, and nesting limits.
- [ ] Confirm one archive copy into Rust, one verified Protobuf copy to Dart, and
  only requested asset copies; monitor memory near 100/64/16 MiB boundaries.
- [ ] Confirm invalid signature, unknown signer, compiler mismatch, schema mismatch,
  malformed/oversized, missing asset, decode failure, and disposed-resource UI
  states retain no stale trusted content.
- [ ] Exercise compact/wide layouts, 200% text scale, keyboard focus order,
  screen-reader labels, 48x48 controls, and non-color-only status cues.
- [ ] Run native integration on macOS and Web integration in Chrome with COOP/COEP.
- [ ] Build Android debug APK, unsigned iOS simulator, macOS, Linux, Windows, and Web.
- [ ] Regenerate Protobuf/FRB/Web outputs and require a clean diff.
- [ ] Run cargo audit/deny and secret scanning; confirm only the public fixture key
  and allowlisted test `.storybundle` are committed.

## Test-to-contract mapping

- Signature/trust failures map to `B_UNKNOWN_SIGNER`, `B_BAD_SIGNATURE`,
  `B_DIGEST_MISMATCH`, and `B_UNSIGNED_DISALLOWED` before model exposure.
- Compatibility failures map to exact `B_COMPILER_MISMATCH`,
  `B_SCHEMA_MISMATCH`, or `B_UNSUPPORTED_FORMAT` after signature verification.
- Resource failures map to `B_RESOURCE_LIMIT` in Dart preflight and Rust authority.
- Asset/lifecycle failures map to `B_MISSING_ASSET` and `B_RESOURCE_DISPOSED`.
- The generated Dart model fingerprint remains
  `0c1bacf81cbe7b4b2cafb68c7d1305f383efda9b3548e7ebfd2b0a5f158d2810`.

Headless Rust and non-UI Dart player tests are agent-owned. Bundle Inspector
widget tests, native/Web integration, accessibility, and platform builds are
human-deferred for the player/save change.

## Station Nine game QA (2026-09-27)

| Unit / failure mode | Check | Automated coverage |
| --- | --- | --- |
| Strict load | Signed key permits playback; unknown signer does not expose a player | Flutter integration; widget retry fake |
| Scene/media | Initial scene carries background, actor dialogue requests portrait; missing art may show a warning without blocking text | Flutter integration and game widget |
| Branching | Patch reveals two final choices; search reveals the conditional override; distinct endings terminate | Rust `station_nine_signed_game_has_distinct_playable_branches`, Flutter integration |
| Lifecycle | Continue/choice are disabled during a pending action; replay releases old player; late open after removal releases its resource | Game widget tests |
| Action failure | A failed choice displays its structured code and retains the prior menu for retry; terminal faults offer a fresh start | Game widget (choice retry and fault restart) |
| Inspector regression | Separate signed fixture still loads, previews, and disposes | Inspector widget and Flutter integration |

These scenarios map to `docs/contracts/storybundle_v1.md` (strict signature,
asset closure, archive bounds) and `docs/contracts/storyplayer_save_v1.md`
(semantic events, choices, effects, stable progression and resource limits).
Run `cargo test --manifest-path player/Cargo.toml --no-default-features
--features storybundle-runtime --test bundle_runtime`, `flutter test test` in
the example, and `flutter test integration_test/storybundle_loading_test.dart
-d macos` for native FFI. Manually verify narrow layouts, 200% text scaling,
keyboard/screen-reader choice order, and Web playback with COOP/COEP. This
sample does not test save slots or audio playback. No HTTP contract was changed.

## Illustrated chapter QA (2026-09-27)

| Unit / failure mode | Expected behavior | Test |
| --- | --- | --- |
| Narrative arc | Patch and archive routes each contain at least 1,500 resolved words; all reachable choices finish within bounded steps | `player/tests/bundle_runtime.rs` |
| Choice consequences | Patch has seal/vent; archive additionally has override; patch/seal survives, archive/seal fails, override saves Station Nine but resets Dot, vent preserves the crew | Rust signed-bundle branch test; macOS integration covers archive/override |
| Character and artwork | Name-only Neri dialogue, emotion-specific Dot portraits, route-specific archive/reactor background, and ending dawn background are all referenced by verified assets | Strict CLI export/verify; Rust effects and Flutter asset integration |
| Reading on a narrow screen | Long pages scroll; advancing resets scroll to the next beat's top; choice buttons retain text and accessible ordinal labels | `storyscript_bundle/example/test/game_screen_test.dart` |
| Signature rotation | New source is compiled with a new external Ed25519 test key; pinned public key verifies new bundle; unchanged source/key yields byte-identical archives | CLI export/verify/re-export comparison; Rust and native FFI tests |
| Regression | Original inspector fixture, strict failure/retry, replay/disposal, and failure recovery remain intact | Existing inspector/game widget and integration suites |

No new runtime, HTTP, save, or archive contract is introduced. The original
`demo.storybundle` is unchanged. Manual acceptance remains for reading pace,
screen-reader order at 200% scale, visual polish, and live Web playback; the
Web compile alone is not proof of browser playback.

## 2026-09-19 initialization execution status

Passed locally: all 9 `cli_init` tests, all 51 bundle crate tests, bundle formatting,
bundle clippy with warnings denied, schema fingerprint validation, CLI help smoke
test, and `git diff --check`. Cargo emitted its pre-existing non-blocking global
registry cache cleanup permission warning during some commands.

## 2026-09-12 execution status

Passed locally before verification was stopped by user direction: Rust bridge
tests, Flutter package analysis and 13 tests, inspector analysis and 4 widget
tests, 2 Rust-backed macOS integration tests, Android debug build, iOS simulator
build, Web FRB/Wasm generation, Flutter Web build, and the schema fingerprint
check. Chrome integration was blocked by missing `chromedriver`; Linux/Windows,
hosted CI, audit/deny, gitleaks, complete repository regression, and final clean
regeneration checks were not run and are not represented as passing.
