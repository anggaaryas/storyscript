import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import 'test_support.dart';

void main() {
  test('save bytes are opaque copies and restore remains Rust-bound', () async {
    final bindings = FakeStoryBundlePlayerBindings();
    final loader = StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bindings,
    );
    final player = await loader.openBytes(Uint8List(0));
    final first = await player.exportSave();
    first[0] = 99;
    expect((await player.exportSave())[0], 1);
    final restored = await loader.restoreBytes(
      Uint8List(0),
      Uint8List.fromList([1, 2, 3]),
    );
    expect(restored.current.event.kind, StoryBundlePlayerEventKind.scene);
  });
}
