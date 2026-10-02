# Workspace Localization, Station Nine and Release Lockstep

Date: 2026-10-02

## Delivery

Completed Phases 8–10 of
`docs/plan/20261002001_storyscript_localization_plan.md` in checkpoint order.
Reconciled the earlier core/bridge journal with actual files; did not restart or
alter the frozen contract phases. Recorded progress at each completed checkpoint.

- Added a bounded TypeScript project index over config, root `@include [...]`
  manifests, keyed source tokens and Fluent ASTs. Open buffers override disk;
  watchers/config/workspace changes rebuild closed files. Providers include keyed
  and scalar-variable completion, definitions/references, workspace symbols,
  unique-ID atomic versioned rename and append-only synchronization/missing catalog
  creation. Path/canonical-locale/profile/coverage/dependency/scope diagnostics are
  advisory; Rust remains authoritative. No Rust binary is shipped/invoked by LSP.
- Added locked Fluent syntax/TOML parsing, a compiler-valid en/id include fixture,
  deterministic tests and Fluent grammar. Compatible npm lock refresh fixed the
  previously reported npm vulnerabilities. Explicitly unignored the editor lock.
- Converted Station Nine's 86 story sites to stable IDs with complete English and
  Indonesian catalogs, a resident-count selector and exact stability arguments.
  English prose/branches/assets remain; identifiers and actor names remain neutral.
- Added independent Flutter shell ARBs/generated localizations and app-owned locale
  state across Game/Inspector. Platform preference order initializes the story;
  selector changes export/restore a candidate using the retained verified bundle,
  then commit shell/story locale and dispose the old player. Failure retains
  locale/event/progress/artwork. Restore preserves artwork without PREP/media reads.
- Re-exported/re-signed Station Nine twice with one external ephemeral key; bytes
  matched, strict verification passed and the private key was deleted. SHA-256:
  `94f40965c83071dd5e13eb5718fd0aa40e08eb9f5f865a4edc316da3000e60e6`.
  Separately verified the Inspector's existing Phase 4 rewritten-v1 fixture/key;
  it required no new rebuild and was not merged with the game trust identity.
- Updated CI/release catalog/editor/generated/non-UI/dependency gates, static
  localization assertions and all relevant feature/checklist/playbook/QA/README
  materials. Preserved existing human UI/platform commands behind explicit
  dispatch inputs. No release was dispatched or committed.
- Normalized seven parser Phase 2 files for the existing fmt gate, with no semantic
  change. Synced the fuzz lock's missing ICU dependency entries. Expanded auditing
  found inactive optional `rkyv 0.7.46` via `rust_decimal 1.41.0` in player/source
  locks; targeted compatible upgrades to 1.43.0 removed the vulnerable entries,
  matching the bundle toolchain. No advisory suppression or RNG identity change.

## Verification

- Required full Rust matrix: **192 passing tests** (64 parser, 65 bundle, 48 player,
  5 source bridge, 10 bundle bridge). Player CLI-help regression also passed.
- Editor: npm clean install, **10 tests**, compile/package and zero-vulnerability
  npm audit. Editor fixture's authoritative Rust catalog check passed (2 messages).
- Non-UI Dart: source **4**, bundle focused **20** plus other loader/trust/asset/Web
  destination **10**, reference controller **3** tests. All analyses and gen-l10n
  passed. Controller tests cover successful same-sequence rerender, rollback,
  artwork retention, verified lease reuse, busy rejection and disposal during save.
- Station Nine all five branches run in en/id/fallback source/bundle lockstep and
  cross-locale restore from byte-identical saves; original English/asset tests pass.
- Protobuf and both pinned FRB generators reproduce owned outputs with no diff;
  ARB generation repeats byte-identically. Both frozen descriptors remain unchanged.
- Schema, installer smoke tests, fuzz compile, parser/bundle fmt, bundle Clippy,
  all five Rust audits, bundle cargo-deny, static docs/dependency/ARB checks,
  actionlint 1.7.12 (without optional shellcheck) and git whitespace checks pass.
- Existing allowed dependency audit warnings (unmaintained, unsoundness and yanked
  advisories), cargo-deny duplicate/unused-allowance warnings, VSIX file-count
  warning and prior bridge-only supplemental Clippy `result_large_err` debt remain
  documented in QA. This is not a warning-free or hosted-platform acceptance claim.
- No prohibited widget/golden/integration test was created, changed or run;
  checked the existing game/Inspector/integration trees have no diff. Native/Web
  artifact builds, accessibility/platform acceptance and actual release dispatch
  remain human-owned. No live service, Docker or Supabase was used.

## Touched files

- Editor: `tool/vscode-storyscript/{package.json,package-lock.json,.vscodeignore,README.md}`,
  `client/src/extension.ts`, `server/src/{server,localization,localizationSource}.ts`,
  `server/test/{tsconfig.json,localization.test.ts,fixtures/localized/**}`,
  `syntaxes/fluent.tmLanguage.json`.
- Reference story/artifact: `storyscript_bundle/example/game/{StoryScript.toml,story/main.StoryScript,localization/en.ftl,localization/id.ftl}`,
  `assets/{station_nine.storybundle,station_nine_public_key.txt}`.
- Reference app: `example/{pubspec.yaml,pubspec.lock,l10n.yaml,README.md}`,
  `lib/{app.dart,main.dart,l10n/*,localization/locale_scope.dart}`,
  `lib/features/game/{game_controller,game_screen}.dart`,
  `lib/features/bundle_inspector/bundle_inspector_screen.dart`,
  `lib/features/bundle_inspector/widgets/{asset_panel,metadata_panel,model_panel}.dart`,
  `test/game_controller_locale_test.dart`.
- Core/locks: `player/tests/bundle_runtime.rs`, `player/Cargo.lock`,
  `storyscript_player_core/rust/Cargo.lock`, `bundle/rust/fuzz/Cargo.lock`,
  formatting-only `parser/rust/src/{ast,compiler,lexer,parser,token,validator}.rs`
  and `parser/rust/tests/compiler_project.rs`.
- Delivery: `.gitignore`, `.github/workflows/{storybundle-ci,release}.yml`,
  `tests/localization_gates.py`, root/bundle/player/both SDK READMEs,
  all three localization/bundle/save contracts, localization/headless/bundle feature
  docs, corresponding onboarding/playbook/QA docs, governing plan and this journal.

## Human handoff

Update/run the existing Game and Inspector widget tests and native/Web integration
tests. Accept English/Indonesian shell/story lockstep, same-checkpoint language
switch and rollback, unsupported locale, count output, long translated choices,
loading/error/empty/fault states, 200% text, narrow/wide layouts, keyboard/screen
reader traversal, 48x48 targets, signed catalog tamper rejection and Inspector
metadata. Dispatch `run_ui_platform` / `run_web_wasm` only after those tests are
maintained, then accept platform builds and release artifacts. Old bundles/saves
have **no migration**; re-export/re-sign/redeploy and restart incompatible progress.
