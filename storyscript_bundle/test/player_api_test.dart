import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import 'test_support.dart';

void main() {
  test(
    'fused loader returns compact player payload without compiled model',
    () async {
      final bindings = FakeStoryBundlePlayerBindings();
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      );
      final player = await loader.openBytes(Uint8List.fromList([1, 2, 3]));
      expect(bindings.openBytesCalls, 1);
      expect(player.current.event.kind, StoryBundlePlayerEventKind.scene);
      expect(bindings.payload, isA<StoryBundlePlayerBridgePayload>());
    },
  );

  test('invalid choices are rejected before the bridge', () async {
    final player = await StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: FakeStoryBundlePlayerBindings(),
    ).openBytes(Uint8List(0));
    expect(() => player.choose(-1), throwsA(isA<StoryBundlePlayerException>()));
  });
}
