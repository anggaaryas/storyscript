# Player Save Recovery Playbook

Audience: operators and support engineers diagnosing headless-player failures.

## Locale-neutral save recovery

Keyed current/pending/choice/history records store IDs and exact argument snapshots,
not rendered prose or a selected locale. Rust project/verified-bundle restore can
negotiate different requested preferences and rerender without PREP/STORY replay.
Both Dart bridges support the same project/bundle locale preferences at restore.
Station Nine commits shell locale only after candidate restoration succeeds.
Check resolved locale, snapshots and numeric/formatting bounds; keep the existing
player unchanged if candidate restore fails. Raw source/path restore displays IDs.
The rewritten-v1 runtime rejects prior progress without migration: re-export bundles
and restart incompatible saves, or roll back a complete old matching toolchain set.
Argument snapshots can contain PII; do not log them. See
[localization release/recovery](localization_release_recovery.md).

## First response

1. Preserve the active session; restore always targets a new candidate player.
2. Record error code, app/player version, story project/version, origin kind,
   bundle signer status/key ID, compiler/schema identities, and byte length.
3. Never log or attach raw save bytes unless an approved secure support channel
   is used. Variables may contain PII.

## Error actions

| Error | Action |
| --- | --- |
| `R_SAVE_STATE_CORRUPT` | Treat bytes as malformed/tampered/truncated; recover a backup. Do not weaken validation. |
| `R_SAVE_INCOMPATIBLE` | Reopen the exact source or authenticated bundle/compiler/schema/project/signer version. |
| `R_SAVE_LIMIT` | Raise only a host-lowered cap up to the hard maximum, or restore an earlier smaller save. |
| `R_EXECUTION_LIMIT` | Keep the prior stable boundary; inspect resource/actual/limit and story logic. |
| `R_PLAYER_BUSY` | Serialize mutations and retry after the in-flight operation completes. |
| `R_PLAYER_DISPOSED` / `B_RESOURCE_DISPOSED` | Recreate through the owning loader; do not reuse stale handles. |

## Correctness checks

- Compare current event, scene, media state, sequence, first-retained sequence,
  and omitted count before/after restore.
- Confirm the current scene's PREP and STORY were not replayed. A later scene
  transition should execute PREP normally.
- Advance through a known random operation and compare with the original session;
  a mismatch indicates wrong identity/state, not acceptable entropy drift.
- For bundles, verify strict versus unsigned-development status, signer key ID,
  compiled-entry digest, schema, compiler, project ID, and project version.

## Rollback and loss

Roll back application runtime, source/bundle artifact, schema, and trust store as
one exact set. v1 has no migration. If no matching artifact or backup exists,
progress is unrecoverable; do not reconstruct state by replaying PREP/STORY.
Host applications own storage durability, encryption, authentication, backups,
retention, deletion, and cloud authorization.
