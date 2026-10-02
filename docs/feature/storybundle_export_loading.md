# StoryBundle Export and Flutter Loading

Audience: StoryScript authors and Rust/Flutter integrators.

## What is available

The `storyscript-bundle` Rust library and CLI compile a `StoryScript.toml` project
into a deterministic, signed `.storybundle`, inspect its untrusted envelope, and
strictly verify/load it. The `storyscript_bundle` Flutter package provides the
same hardened loader on Android, iOS, macOS, Linux, Windows, and Web, then decodes
the verified payload into the generated full Dart Protobuf model. The bundle
contains semantic Protobuf IR, required assets and canonical signed locale catalogs,
not raw StoryScript source or author FTL comments/host paths.

This is packaging, not encryption or DRM. Anyone with the bundle can recover its
semantic text and assets. A separate optional headless player entrypoint executes
verified bundles in Rust. Localization deliberately rewrites both existing v1
descriptors; prior bundles/saves are rejected without migration.

The Flutter example now includes a playable signed branching light-novel
chapter as well as the original Bundle Inspector. These use **separate**
checked-in bundles and trusted public test keys; see
`storyscript_bundle/example/README.md`. The
inspector's model view is not a StoryScript executor.

## Project model

Create a minimal project with explicit identity and display name:

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- init ./my-story \
  --id com.example.story \
  --name "Example"
```

The target must not already exist. `init` creates `StoryScript.toml`, a valid
`story/main.StoryScript`, and `assets/.gitkeep` so the required asset root remains
present after source-control checkout. It does not generate keys or demo media.
The generated project version starts at `0.1.0` and dynamic asset declarations
are omitted. The descriptor can be extended using the complete format below:

```toml
[project]
id = "com.example.story"
name = "Example"
version = "0.1.0"
entry = "story/main.StoryScript"
compiler-version = "0.1.0-localization.1"

[assets]
root = "assets"
literal-policy = "required"
dynamic-files = ["portraits/hero-happy.svg"]
dynamic-globs = ["backgrounds/*.svg"]

[localization]
default-locale = "en"
supported-locales = ["en", "id"]
root = "localization"
```

- The compiler pin must exactly match `storyscript_parser::COMPILER_VERSION`.
- Entry/include/asset paths are relative, normalized, and sandboxed.
- Literal portrait/background/BGM/SFX paths are included automatically.
- A path containing `${variable}` is dynamic. It must match at least one explicit
  `dynamic-files` or `dynamic-globs` candidate; all compatible matches are included.
- Private-key fields and unknown fields are rejected.
- Localization is optional for plain projects and required for keyed text. Use
  `init --localized` for an English starter. Add complete `en.ftl`/`id.ftl` catalogs
  and use `localize extract`, `sync`, and non-mutating `check` before export.
- Default/supported locale metadata and scalar message contracts are semantic IR;
  canonical catalogs remain separate signed entries. See [localization](storyscript_localization.md).

## Export flow

1. Canonicalize the project, entry, includes, and asset root.
2. Compile and validate root/child modules in source order.
3. Convert every semantic AST variant to source-free Protobuf v1.
4. Validate complete Fluent catalogs, transitive variables/types/profile/bounds,
   canonicalize comment-free records, and resolve a closed sorted asset graph.
5. Index every catalog and asset digest in the manifest and sign its domain-separated
   digest with Ed25519.
6. Write fixed-metadata, stored ZIP entries and enforce all hard limits.

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- export \
  --project ./my-story \
  --output /tmp/my-story.storybundle \
  --signing-key /secure/storybundle-private.pem
```

The signing key must be PKCS#8 PEM and outside the project/repository. Key bytes are
zeroized after parsing and never written to `StoryScript.toml` or logs.

## Inspect and verify

`inspect` validates only bounded ZIP/manifest envelope shape and labels output as
untrusted. It never decodes the semantic model.

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- inspect /tmp/my-story.storybundle --json
```

`verify` accepts only a trusted Ed25519 SubjectPublicKeyInfo PEM and runs the full
fail-closed pipeline:

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- verify \
  /tmp/my-story.storybundle \
  --public-key /secure/storybundle-public.pem \
  --json
```

The Rust loader defaults to `VerificationPolicy::Strict`. The conspicuously named
`UnsignedDevelopment` policy is library-only, is never global, and marks returned
verification status. Assets remain archive-backed and are copied only by an
explicit bounded `LoadedBundle::read_asset` call.
Catalog validation follows digest/semantic verification before exposing any model
or player. `LoadedBundle::read_catalog` returns one bounded verified locale resource.

## Flutter load flow

Initialize FRB once, provision authenticated raw 32-byte Ed25519 public keys, and
construct the strict loader:

```dart
await RustLib.init();
final loader = StoryBundleLoader(
  trustStore: StoryBundleTrustStore([
    StoryBundleTrustKey(publicKeyBytes, expectedKeyId: keyId),
  ]),
);
final bundle = await loader.openBytes(archiveBytes);
final scenes = bundle.story.scenes;
final portrait = await bundle.readAsset('portraits/hero.png');
await bundle.dispose();
```

Native applications may use `openPath`; Web is bytes-only. The copy profile is
one archive transfer into Rust, one verified Protobuf transfer to Dart, and one
copy per requested asset. Opening never copies or caches all assets/catalogs in Dart;
the generated model includes default/supported locale metadata, not FTL bodies.
`StoryBundleLoader.unsignedDevelopment()` is the only unsigned entry point and
its returned status remains visibly unsigned. Progress hooks and load generations
prevent slow/stale results from replacing a newer selection.

## Compatibility and limits

Format version, compiler SemVer, and descriptor SHA-256 must match exactly. There is
no v1 migration range. Limits are 100 MiB archive/total uncompressed, 64 MiB per
asset, 16 MiB compiled IR, 1 MiB manifest, 4,096 entries, 1,024-byte paths, and
semantic depth 128. Localization additionally caps declared locales at 64, IDs at
100,000, catalogs at 16 MiB and message IDs at 256 bytes; formatted events remain
at most 1 MiB. Archive-wide limits still win. Hosts may lower operational limits.
Re-export/re-sign all old-v1 bundles and restart incompatible saves; no migration.

The example at `storyscript_bundle/example/` starts a playable branching game.
Its toolbar opens the separate responsive Bundle Inspector, which shows trust
and compatibility status, project/model summaries, normalized assets, and
bounded image previews. The inspector itself does not execute stories or play
media.

Applications that need playback import `storyscript_bundle_player.dart`. Its
preferred fused path verifies and executes entirely in Rust without retransferring
the compiled model to Dart; an inspector-plus-player flow can hand an existing
`LoadedStoryBundle` capability to multiple isolated players. Player progress is
stored separately as opaque exact-origin save bytes. Use
the Rust `_with_locales` constructors for ordered requested locale negotiation.
Keyed saves are locale-neutral and restore from exact arguments into another locale;
locale-aware Dart parameters remain phase 7. See
`docs/feature/headless_story_player.md`.

See the normative contract at `docs/contracts/storybundle_v1.md`.
