# StoryBundle Project Checklist

Audience: developers preparing their first export.

## Localization (rewritten v1)

- [ ] Pin compiler `0.1.0-localization.1`, not the package's `0.1.0` version.
- [ ] For keyed content follow [localized project checklist](localized_story_project_checklist.md):
  complete canonical default/supported catalogs and `localize check` before export.
- [ ] Verify signed catalog digests/bounds and no eager generic Dart catalog transfer.
- [ ] Re-export/re-sign old-v1 bundles; old saves require a restart, with no migration.
- [ ] Test source-project / signed-bundle en/id/fallback parity and locale-neutral
  cross-locale restores. Both Rust and Dart locale APIs are ready; use the immutable
  ordered preference models and display requested/resolved fallback. Editor advice
  never replaces strict CLI check. Station Nine is the localized reference app.

## Authoring

- [ ] Initialize a new project path with an explicit stable ID and display name:

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- init ./my-story \
  --id com.example.story \
  --name "Example Story"
```

- [ ] Confirm the target did not exist before initialization; `init` never merges
  into or overwrites an existing file, directory, or symlink.
- [ ] If authoring manually, add one `StoryScript.toml`, one valid entry script,
  and the configured asset-root directory at the project root.
- [ ] Set non-empty project ID/name and a SemVer project version.
- [ ] Pin `compiler-version` exactly to `storyscript_parser::COMPILER_VERSION`.
- [ ] Keep the entry and all includes relative; do not use `..` or symlink escapes.
- [ ] Keep exactly one root `* INIT`; each included child uses one `* REQUIRE`.
- [ ] Put assets under the configured asset root with normalized POSIX-style paths.
- [ ] Confirm every literal portrait/background/BGM/SFX path exists.
- [ ] Add explicit `dynamic-files` or `dynamic-globs` for each `${variable}` asset path.
- [ ] Confirm each dynamic template has at least one compatible match.

## Signing and export

- [ ] Generate an Ed25519 PKCS#8 private key outside the repository/project.
- [ ] Restrict private-key filesystem permissions and back it up securely.
- [ ] Derive and distribute only the public key to loader trust stores.
- [ ] Export to a filename ending in `.storybundle`.
- [ ] Run `schema check` before export when schema files or generators changed.

```bash
openssl genpkey -algorithm ED25519 -out /secure/storybundle-private.pem
openssl pkey -in /secure/storybundle-private.pem -pubout \
  -out /secure/storybundle-public.pem

cargo run --manifest-path bundle/rust/Cargo.toml -- schema check
cargo run --manifest-path bundle/rust/Cargo.toml -- export \
  --project ./my-story \
  --output /tmp/my-story.storybundle \
  --signing-key /secure/storybundle-private.pem
```

## Verification and readiness

- [ ] Run bounded `inspect` and remember its metadata is untrusted.
- [ ] Run strict `verify` using only the expected public key.
- [ ] Re-export with unchanged inputs/key and compare bytes for equality.
- [ ] Confirm ZIP entries contain no `.StoryScript`, source map, host path, or key.
- [ ] Confirm archive/uncompressed size is at most 100 MiB, each asset at most
  64 MiB, compiled IR at most 16 MiB, manifest at most 1 MiB, and entries at most
  4,096.
- [ ] Test unknown key, tampered asset, compiler mismatch, and schema mismatch.

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- \
  inspect /tmp/my-story.storybundle --json
cargo run --manifest-path bundle/rust/Cargo.toml -- \
  verify /tmp/my-story.storybundle \
  --public-key /secure/storybundle-public.pem --json
```

## Flutter integration

- [ ] Add the local/package-ready `storyscript_bundle` package and call
  `RustLib.init()` before the first load.
- [ ] Provision authenticated raw 32-byte Ed25519 public keys in a
  `StoryBundleTrustStore`; do not trust a key embedded in a bundle.
- [ ] Use ordinary `StoryBundleLoader` in production. Reserve
  `StoryBundleLoader.unsignedDevelopment()` for conspicuous local fixtures.
- [ ] Use `openBytes` on every platform; use `openPath` only on native hosts.
- [ ] Lower `StoryBundleLimits` when the host needs a smaller memory profile.
- [ ] Read assets lazily and call `LoadedStoryBundle.dispose()` on replacement,
  navigation, and owner disposal.
- [ ] For Web, build FRB Wasm and serve with COOP `same-origin` and COEP
  `require-corp` headers.
- [ ] Run the Bundle Inspector and verify trust text, model counts, asset preview,
  reload, failure clearing, and disposal.
- [ ] For playback, import `storyscript_bundle_player.dart` and prefer fused
  verified player creation when Dart does not need the inspector model.
- [ ] Treat save identity as exact bundle/compiler/schema/project/digest/signer
  identity; source and bundle saves are not interchangeable.
- [ ] Lower player limits, handle history truncation, and dispose every player
  lease even if its parent `LoadedStoryBundle` was disposed first.
- [ ] Complete `docs/onboarding/headless_player_integration_checklist.md`.
- [ ] Run `storyscript_bundle/example` to read the signed branching chapter;
  choose both the patch and search paths, then try the conditional override.
  Open the toolbar inspector to view its separate signed fixture.
- [ ] In the expanded chapter, check Neri's name-only dialogue, Dot's
  alert/calm/dim portraits, distinct reactor/archive/dawn backgrounds, and
  whether the override's memory cost has been established before choosing it.
  Scroll a long page on a narrow screen and confirm the next beat starts at top.
- [ ] When changing `example/game/story/main.StoryScript` or its SVG assets,
  create an external ephemeral signing key, rebuild the game bundle, update its
  public test key, and rerun Rust and Flutter integration tests. Never change
  the original inspector fixture just to update the game.
