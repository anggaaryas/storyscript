# storyscript_bundle

Flutter FFI package for opening verified StoryScript `.storybundle` archives on
Android, iOS, macOS, Linux, Windows, and Web. Rust authenticates and validates
the archive before Dart receives its generated, typed Protobuf model.

The format removes raw `.StoryScript` source, comments, and source locations. It
is not encryption or DRM: semantic text and assets remain recoverable.

## Initialize and load

```dart
await RustLib.init();

final trustStore = StoryBundleTrustStore([
  StoryBundleTrustKey(
    publisherPublicKeyBytes, // exactly 32 raw Ed25519 bytes
    expectedKeyId: publisherKeyId,
  ),
]);
final loader = StoryBundleLoader(trustStore: trustStore);
final loaded = await loader.openBytes(bundleBytes);

print(loaded.manifest.project.name);
print(loaded.story.scenes.length);
final portrait = await loaded.readAsset('portraits/hero.png');
await loaded.dispose();
```

Strict trusted-signature verification is the default. Local unsigned fixtures
require `StoryBundleLoader.unsignedDevelopment()` and the returned
`verification.isUnsignedDevelopment` remains true. There is no global bypass.

Native hosts may call `openPath`; Web is bytes-only. Calls are asynchronous and
asset bytes are copied lazily. Opening transfers bundle bytes once into Rust and
returns one verified Protobuf payload; it does not copy every asset into Dart.

## Headless player entrypoint

Import `storyscript_bundle_player.dart` when the host needs execution rather
than inspection. `StoryBundlePlayerLoader` verifies and opens bytes/path directly
into a Rust player without returning or decoding `CompiledStory` in Dart. It can
also create multiple isolated players from an existing `LoadedStoryBundle`.

Players expose compact current/advance/choice deltas, explicit bounded history
pages, opaque save export/restore, bounded asset reads, and idempotent disposal.
Save bytes require the exact authenticated bundle origin and are not encrypted
or authenticated. The host owns UI and persistence.

All six player loader routes accept immutable ordered locale preferences:

```dart
final loader = StoryBundlePlayerLoader(trustStore: trustStore);
final player = await loader.openBytes(bundleBytes,
  locales: StoryBundlePlayerLocalePreferences(['id-ID', 'en']));
print(player.resolvedLocale);
final save = await player.exportSave();
final candidate = await loader.restoreBytes(bundleBytes, save,
  locales: StoryBundlePlayerLocalePreferences(['en']));
await player.dispose(); // publish candidate first; retain old player on failure
```

`openPath`, `restorePath`, `fromBundle` and `restoreFromBundle` take the same named
`locales` parameter. Empty preferences select the project default; non-localized
bundles report null `resolvedLocale`. `player.locale` is immutable. Catalogs are
signed, fully verified and read through Rust's retained bundle lease; fused paths
transfer neither catalog bodies nor the compiled model to Dart. Save bytes contain
locale-neutral message snapshots and restore rerenders current/pending/history
without PREP/STORY replay. Dart receives rendered plain text only, while the host
localizes its shell. Custom adapters opt into `LocaleAwareStoryBundlePlayerBindings`;
legacy injected bindings still work with defaults but explicit preferences fail
with `R_LOCALIZATION_BINDINGS`. See the [localization guide](../docs/feature/storyscript_localization.md).

## Limits and lifecycle

Hard ceilings are 100 MiB archive/total uncompressed, 64 MiB per entry, 16 MiB
compiled IR, 1 MiB manifest, 4,096 entries, 1,024-byte paths, and semantic depth
128. Localization additionally caps 64 locales, 100,000 message/term IDs, 16 MiB
per catalog and 256-byte IDs. `StoryBundleLimits` may lower but not raise them. Dart preflights observable
limits and Rust enforces all limits authoritatively.

`LoadedStoryBundle.dispose()` is idempotent and releases its caller handle.
Reads after disposal throw `StoryBundleDisposedException`; existing child
players retain a shared verified lease until the final player is disposed.
Starting or cancelling a newer load prevents stale results from being exposed.

## Generated contracts

- FRB runtime/codegen: `2.13.0`
- Web `wasm-bindgen` crate/CLI: `0.2.118`
- Dart Protobuf runtime: `6.1.0`
- Dart Protobuf generator: `protoc_plugin 25.1.0`
- Canonical schema: `../bundle/proto/storybundle/v1/compiled_story.proto`
- Descriptor SHA-256: `0c1bacf81cbe7b4b2cafb68c7d1305f383efda9b3548e7ebfd2b0a5f158d2810`

Regenerate FRB with `flutter_rust_bridge_codegen generate`. From this package
directory, build Web bindings into the Flutter example with
`flutter_rust_bridge_codegen build-web --output ../example/web` (the output is
resolved from the `rust/` crate directory); FRB supplies the complete
threaded-Wasm flags. Rebuild Wasm after regenerating the bridge;
otherwise the example may load an older Rust content hash. Regenerate
Protobuf from the repository root:

```bash
dart pub global activate protoc_plugin 25.1.0
protoc --proto_path=bundle/proto \
  --dart_out=storyscript_bundle/lib/src/proto \
  bundle/proto/storybundle/v1/compiled_story.proto
```

Web Wasm builds require COOP/COEP headers; see the repository playbook and the
playable example with its separate inspector. This package is version `0.1.0`,
licensed LGPL-2.1-only, and is package-ready but not published to pub.dev.
