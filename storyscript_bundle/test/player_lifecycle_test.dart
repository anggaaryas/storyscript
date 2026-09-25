import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import 'test_support.dart';

void main() {
  test(
    'verified bundle handoff and parent-first disposal keep child usable',
    () async {
      final bundleBindings = FakeStoryBundleBindings();
      final loaded = await StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bundleBindings,
      ).openBytes(Uint8List(0));
      final playerBindings = FakeStoryBundlePlayerBindings();
      final player = await StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: playerBindings,
      ).fromBundle(loaded);
      await loaded.dispose();
      expect((await player.readAsset('images/preview.png'))[0], 4);
      await player.dispose();
      await player.dispose();
      expect(playerBindings.disposeCalls, 1);
    },
  );

  test('stale player completion is disposed', () async {
    final bindings = FakeStoryBundlePlayerBindings();
    bindings.pendingOpen = Completer<StoryBundlePlayerBridgePayload>();
    final loader = StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bindings,
    );
    final pending = loader.openBytes(Uint8List(0));
    loader.cancelPendingLoads();
    bindings.pendingOpen!.complete(bindings.payload);
    await expectLater(pending, throwsA(isA<StoryBundlePlayerException>()));
    expect(bindings.disposeCalls, 1);
  });
}
