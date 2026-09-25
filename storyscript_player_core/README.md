# storyscript_player_core

Flutter FFI package for source-based StoryScript playback on native and Web.
It contains no widgets, routing, state-management integration, or persistence.

## Headless API

```dart
await RustLib.init();
final loader = SourceStoryPlayerLoader();
final player = await loader.openSource(sourceText);

final delta = await player.advance();
final page = await player.history(startSequence: 0);
final save = await player.exportSave(); // opaque Uint8List
await player.dispose();

final restored = await loader.restoreSource(sourceText, save);
```

`openPath`/`restorePath` are native filesystem conveniences. Progression returns
compact semantic events/effects and truncation metadata; transcript pages and
save bytes are explicit calls. Restore creates a new session and requires the
exact semantic source identity. The current scene's PREP/STORY is not replayed.

The generated legacy numeric-session functions remain exported for existing
consumers. New integrations should use `SourceStoryPlayerLoader` and its
injectable `SourcePlayerBindings` seam.

## Ownership and limits

`StoryPlayerLimits` can lower the Rust hard profile. Mutations are serialized;
busy/disposed/errors are structured, and disposal is idempotent. Host apps own
UI, storage, encryption/authentication, backup, deletion, and cloud/AuthZ.
Progress saves are not encrypted or authenticated by the SDK.

FRB runtime and generated bindings are pinned to 2.12.0. Regenerate with the
matching `flutter_rust_bridge_codegen` version, then run:

```sh
cargo test --manifest-path rust/Cargo.toml
flutter analyze
flutter test test/player_save_contract_test.dart
```
