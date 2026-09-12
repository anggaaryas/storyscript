import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

import 'test_support.dart';

void main() {
  test('assets are copied lazily with caller bounds', () async {
    final bridge = FakeStoryBundleBindings();
    final loaded = await StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bridge,
    ).openBytes(Uint8List(1));

    expect(bridge.readCalls, 0);
    expect(await loaded.readAsset('images/preview.png', maximumBytes: 4), <int>[
      1,
      2,
      3,
      4,
    ]);
    expect(bridge.readCalls, 1);
  });

  test('missing, unnormalized, and over-limit reads fail before FFI', () async {
    final bridge = FakeStoryBundleBindings();
    final loaded = await StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bridge,
    ).openBytes(Uint8List(1));

    await expectLater(
      loaded.readAsset('../preview.png'),
      throwsA(isA<StoryBundleAssetException>()),
    );
    await expectLater(
      loaded.readAsset('images/missing.png'),
      throwsA(isA<StoryBundleAssetException>()),
    );
    await expectLater(
      loaded.readAsset('images/preview.png', maximumBytes: 3),
      throwsA(isA<StoryBundleLimitException>()),
    );
    expect(bridge.readCalls, 0);
  });

  test(
    'dispose is idempotent and reads-after-dispose are structured',
    () async {
      final bridge = FakeStoryBundleBindings();
      final loaded = await StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bridge,
      ).openBytes(Uint8List(1));

      await Future.wait(<Future<void>>[loaded.dispose(), loaded.dispose()]);

      expect(bridge.disposeCalls, 1);
      await expectLater(
        loaded.readAsset('images/preview.png'),
        throwsA(isA<StoryBundleDisposedException>()),
      );
    },
  );
}
