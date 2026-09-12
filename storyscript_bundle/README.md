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

## Limits and lifecycle

Hard ceilings are 100 MiB archive/total uncompressed, 64 MiB per entry, 16 MiB
compiled IR, 1 MiB manifest, 4,096 entries, 1,024-byte paths, and semantic depth
128. `StoryBundleLimits` may lower but not raise them. Dart preflights observable
limits and Rust enforces all limits authoritatively.

`LoadedStoryBundle.dispose()` is idempotent and immediately releases Rust-held
archive bytes. Reads after disposal throw `StoryBundleDisposedException`.
Starting or cancelling a newer load prevents stale results from being exposed.

## Generated contracts

- FRB runtime/codegen: `2.12.0`
- Dart Protobuf runtime: `6.1.0`
- Dart Protobuf generator: `protoc_plugin 25.1.0`
- Canonical schema: `../bundle/proto/storybundle/v1/compiled_story.proto`
- Descriptor SHA-256: `aee45aee882cacdfeb6d320808ed0ac72805da27c87b5486289e6a9b6be9d70a`

Regenerate FRB with `flutter_rust_bridge_codegen generate`. Build Web bindings
into the example with `flutter_rust_bridge_codegen build-web --output
../example/web` plus the shared-memory flags used by CI. Regenerate Protobuf
from the repository root:

```bash
dart pub global activate protoc_plugin 25.1.0
protoc --proto_path=bundle/proto \
  --dart_out=storyscript_bundle/lib/src/proto \
  bundle/proto/storybundle/v1/compiled_story.proto
```

Web Wasm builds require COOP/COEP headers; see the repository playbook and the
example inspector. This package is version `0.1.0`, licensed LGPL-2.1-only, and
is package-ready but not published to pub.dev.
