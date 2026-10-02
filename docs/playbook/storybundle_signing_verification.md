# StoryBundle Signing and Verification Playbook

Audience: release operators and application trust-store maintainers.

## Localization rewrite

The compiler identity is `0.1.0-localization.1`; both v1 descriptors were replaced.
Re-export/re-sign every prior bundle and deploy matching loaders/runtime/trust
keys; old saves have no migration and may require restarting progress.
Run `localize check --project "$PROJECT" --json` before export. Canonical locale
catalogs are signed payloads, not host-side translator files. A catalog digest,
coverage/profile, alias, missing/extra or size failure exposes no model/player;
never bypass it with development policy. Use
[localization recovery](localization_release_recovery.md) for locale, numeric,
resolver and cross-locale restore incidents. Generic Dart loading gets metadata,
not eager FTL body copies. Both separately trusted example fixtures were rebuilt
for this replacement; Station Nine now ships complete signed English/Indonesian
catalogs and its own rotated public key. Inspector retains its separate trust identity.

## Project preparation

For a new project, initialize a minimal source tree before provisioning signing
material:

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- init "$PROJECT" \
  --id com.example.story --name "Example Story"
```

The target must not already exist. Initialization never generates signing keys,
stores secret material, or overwrites an existing path. Review the generated
identity and source before entering the release procedure.

## Key handling

1. Generate an Ed25519 PKCS#8 key outside the repository and project.
2. Restrict private-key access to the export job. Never commit, log, bundle, or put
   its path in `StoryScript.toml`.
3. Derive the public SubjectPublicKeyInfo PEM. The key ID is lowercase
   `SHA-256(raw 32-byte Ed25519 public key)`.
4. Provision the public key in each host trust store before distributing bundles.

Flutter consumers receive raw 32-byte public keys, not PEM text. Convert and
authenticate keys during application release engineering, pin the expected key ID,
and avoid loading raw keys from mutable remote content without an authenticated
configuration channel. A key carried only inside a bundle never establishes trust.

The CLI refuses private keys located under the selected project or its containing
Git repository and zeroizes the PEM read buffer after key parsing.

## Release procedure

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- schema check
cargo run --manifest-path bundle/rust/Cargo.toml -- export \
  --project "$PROJECT" --output "$OUTPUT.storybundle" \
  --signing-key "$EXTERNAL_PRIVATE_KEY"
cargo run --manifest-path bundle/rust/Cargo.toml -- verify \
  "$OUTPUT.storybundle" --public-key "$PUBLIC_KEY" --json
```

Re-run export with unchanged project bytes, compiler/schema, options, and key. The
two archives must be byte-identical. Inspect ZIP names and confirm no raw source,
source map, absolute host path, or private key appears.

## Trust rotation

1. Add the new public key/key ID to hosts first.
2. Export and verify a canary bundle with the new private key.
3. Distribute newly signed bundles.
4. Retain the old public key until all old bundles are retired.
5. Remove the old key and verify that old bundles now fail as `B_UNKNOWN_SIGNER`.

Do not reuse a compromised key. Revoke it from trust stores, re-export unaffected
project inputs under a new key, and invalidate all artifacts signed by the old key.
`B_BAD_SIGNATURE` indicates tampering or key/signature mismatch; do not retry as
unsigned development in production.

## Failure response

| Code | Operator action |
| --- | --- |
| `B_UNKNOWN_SIGNER` | Confirm expected key ID; provision only an authenticated public key. |
| `B_BAD_SIGNATURE` | Quarantine the artifact and investigate tampering/wrong signing key. |
| `B_DIGEST_MISMATCH` | Quarantine; payload changed after signing. |
| `B_COMPILER_MISMATCH` | Rebuild with the loader's exact parser/exporter version. |
| `B_SCHEMA_MISMATCH` | Restore the matching schema/exporter/loader set and rebuild. |
| `B_RESOURCE_LIMIT` | Reduce project/assets or lower operational input; never raise hard limits. |
| `B_ARCHIVE_INVALID` / `B_MANIFEST_MALFORMED` | Treat as hostile or corrupt input. |
| `B_PROTOBUF_DECODE` / `B_SEMANTIC_VIOLATION` | Reject and rebuild from validated source. |
| `B_RESOURCE_DISPOSED` | Stop stale reads and reopen the bundle through its owning lifecycle. |
| `R_SAVE_INCOMPATIBLE` | Restore the exact authenticated bundle origin; never downgrade signature policy. |
| `R_SAVE_STATE_CORRUPT` | Reject the save and recover a protected backup; do not log its contents. |

## Unsigned development policy

The Rust library exposes `VerificationPolicy::UnsignedDevelopment`; it is not the
default and returned status sets `is_unsigned_development = true`. It is for local
fixtures only. The verification CLI intentionally uses strict policy. Never add a
global fallback from signature failure to unsigned loading.

## Rollback

Roll back the exporter, parser compiler version, schema fingerprint, loader, and
trust-store configuration as one set. v1 has exact matching and no decoder migration
or compatibility range. Keep the last verified toolchain and public trust mapping
available until the new set has completed canary verification.

## Flutter and Web operations

Call `LoadedStoryBundle.dispose()` before replacing a bundle and when the owning
screen/service closes. Child players hold independent verified leases and remain
usable after parent disposal; dispose each child to release the final archive.
A failed or stale load must leave no previously trusted
model or preview visible. Monitor `StoryBundleLoadProgress` for UX only; timeout
and cancellation policy belongs to the host, while stale completion is rejected by
the loader generation guard.

Build StoryBundle Web bindings with FRB 2.13.0 and serve the inspector/application with:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

When compiler/schema/package identities change, regenerate Rust/Dart Protobuf and
FRB bindings, require a clean generated diff, re-export all bundles, canary strict
loading, then deploy exporter/loader/trust-store changes as one release. Roll back
to the last matching set if exact compatibility fails.

The player handoff never weakens signature ordering. Strict/development status,
signer key ID, and compiled-entry digest become part of save origin identity.
Use `docs/playbook/player_save_recovery.md` for runtime/save incidents.

## Playable example fixture rotation

The playable `storyscript_bundle/example/game/` authoring tree exports to
`storyscript_bundle/example/assets/station_nine.storybundle`. Its raw public
test key is pinned separately in `station_nine_public_key.txt`; the private key
was discarded. A source or asset change requires a new external Ed25519 test
key, export and strict verification, replacement of both bundle and pinned
public key, and re-running `player/tests/bundle_runtime.rs` plus the Flutter
example integration test. Follow the exact commands in the example README.
The inspector's `demo.storybundle` and key remain independent. Do not silently
enable unsigned-development policy on a signature failure.
The chapter's longer prose and new background/portrait references are part of
the compiled identity and signed asset closure; publishing a revised chapter
requires a fresh bundle and matching test key. Existing saves for an earlier
game revision would not restore against the new origin (the sample stores none).
