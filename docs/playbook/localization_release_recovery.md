# Localization Release and Recovery

Audience: release operators, runtime maintainers and support engineers.

## Release gate

1. Match compiler `0.1.0-localization.1`, both pinned v1 descriptors and runtime
   `storyscript-player/0.1.0:model-v1-localization`.
2. Run `localize check --project "$PROJECT" --json`; review TODO translations.
3. Run parser/catalog/contract/archive/runtime/save and source/bundle bridge
   regression checkpoints, Flutter analysis and focused non-UI Dart tests.
4. Export/sign using an external protected key and verify strictly. Repeat export
   to prove determinism; confirm all canonical locale entries are signed and no
   author comments or host paths were packaged.
5. Provision authenticated public trust keys and deploy matching toolchain/artifacts.
   Keep game/Inspector fixture keys separate; discard ephemeral private test keys.
6. Canary English/Indonesian, unsupported fallback, source/bundle parity and
   cross-locale restore. Dart facade tests are injected and Rust bridge tests exercise
   real projects/archives; editor tests run separately, while native/Web FFI and UI
   acceptance remain human-owned.

CI runs editor tests/package/audit, both descriptors/generator drift, strict Station
Nine catalogs, Rust runtime/security suites and focused non-UI Dart/controller
tests. `tests/localization_gates.py` checks doc/dependency/schema/ARB invariants.
Existing widget/platform jobs require explicit human `run_ui_platform` dispatch;
Web integration uses `run_web_wasm`. A green agent gate is not platform acceptance.
The release workflow verifies strict catalogs and localization lockstep before
building CLI archives; only humans dispatch/accept/publish releases.

## First response

Preserve the active player. Record structured code, scene, compiler/schema/runtime
identity, project/version, verification status/key ID, requested/resolved locale,
resource bounds and catalog logical path. Do not log save bytes or argument values:
saves are not encrypted or authenticated, and snapshots may contain PII.

| Symptom | Diagnosis and action |
|---|---|
| Missing/unused/duplicate messages or argument drift | Run `extract` and `check`; compare source-relative sites and every declared catalog. Use `sync` only to append stubs; reconcile obsolete/conflicting entries manually. |
| Malformed FTL, cycle, attribute/custom function | Correct the catalog using the restricted Fluent profile. Do not weaken validation or use partial output. |
| Case/Unicode path or locale alias, symlink escape | Use exact canonical locale filenames beneath the sandboxed root; remove aliases/escapes and rebuild. |
| `B_DIGEST_MISMATCH`, bad signature, missing/extra catalog | Quarantine the archive. Re-export/re-sign from validated source with trusted keys; never retry unsigned in production. |
| `B_RESOURCE_LIMIT` | Reduce locale/catalog/graph/output size or lower a host profile; never raise hard limits. Catalogs share archive-wide bounds. |
| `R_LOCALIZATION_LOCALE` | Supply valid hyphenated BCP-47 language identifiers. Unsupported valid tags fall back; malformed tags fail rather than silently succeeding. |
| Unresolved ID output, no resolved locale | The host used raw source/path APIs. Open the project descriptor/catalogs explicitly; IDs are not translated output. |
| `R_LOCALIZATION_BINDINGS` | An injected legacy bundle adapter cannot honor explicit preferences. Implement `LocaleAwareStoryBundlePlayerBindings` or use the standard FFI adapter; do not discard preferences silently. |
| `R_STALE_LOAD` | A newer bundle load or cancellation superseded this result. The candidate was disposed; keep the published session and do not publish the stale locale. |
| Bridge content-hash mismatch after SDK upgrade | Regenerate with source FRB 2.12.0 / bundle FRB 2.13.0 and rebuild the matching native/Web binary. Do not mix generated Dart and old Rust/Wasm artifacts. |
| `R_LOCALIZATION_NUMBER` | The exact value cannot survive Fluent's `f64`. Use exact-safe values or precompute a locale-neutral string deliberately; do not silently round. Failed interaction retains the checkpoint. |
| `R_LOCALIZATION_FORMAT` | Resolver failed; inspect terms/bindings/placeable complexity. No partial pattern is publishable. Reduce complexity and run check/runtime tests. |
| `R_LOCALIZATION_ARGUMENT` / `R_LOCALIZATION_MESSAGE` | Check loaded story/catalog contracts and runtime variable availability. Do not inject resources or recompute pending snapshots. |
| `R_EXECUTION_LIMIT` during restore/switch | Restore formatting is bounded too. Keep the old player and checkpoint, reduce complexity/history or restore an earlier save under valid caps. |
| `R_SAVE_STATE_CORRUPT` | Reject unknown IDs, wrong names/types/values, arrays or impossible progress; recover a protected backup. |
| Old-v1 `B_SCHEMA_MISMATCH` / `R_SAVE_INCOMPATIBLE` | Re-export bundles with matching rewritten descriptors and restart old progress. No migration exists. |
| Stale editor localization index | Save/reopen buffers or reload the workspace after external changes; watchers rebuild source/config/FTL indexes. Older buffer versions are ignored and rename refuses changed closed-file snapshots. Resolve paths/includes and run CLI check as authority. |

## Locale changes and rollback

A session's locale cannot be mutated. Export its locale-neutral save, open/restore
a new candidate using different ordered preferences, and swap only after success.
Dispose the old resource after publication; on failure retain player, locale,
event/history, media and progress. Restore must not replay current PREP/STORY.
Rust constructors and Dart `restoreProject` / bundle bytes/path/existing-bundle
restore APIs support this now. Use immutable ordered preference models and expose
requested versus `player.resolvedLocale` when fallback matters. Catalog parsing and
message arguments remain in Rust; do not rewrite rendered text in Dart. Station
Nine retains its verified bundle, restores a candidate and commits the shell
locale only on success. Busy/error feedback is app-owned ARB text. On failed switch
retry after resolving the structured cause; do not restart or discard the old player.

Roll back runtime/compiler, source or bundle, both schemas, package binaries and
trust mapping as one matching set. The in-place v1 replacement deliberately rejects
old bundles and saves; preserve the old toolchain only to run its own matching
artifacts. If no compatible artifact/backup exists, progress is lost. Never
reconstruct it by replaying PREP/STORY or promise a migration.

See [normative contract](../contracts/storyscript_localization_v1.md) and
[QA matrix](../qa-docs/storyscript_localization_v1_qa.md). Platform builds and all
Flutter widget/golden/integration executions remain human-owned.
