import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

import 'test_support.dart';

void main() {
  test(
    'strict bridge contract is the default and receives bounded input',
    () async {
      final bridge = FakeStoryBundleBindings();
      final loader = StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        limits: const StoryBundleLimits(maxArchiveBytes: 64),
        bindings: bridge,
      );

      final loaded = await loader.openBytes(Uint8List(8));

      expect(bridge.lastRequest?.policy, StoryBundleVerificationPolicy.strict);
      expect(bridge.lastRequest?.limits.maxArchiveBytes, 64);
      expect(loaded.story.project.id, 'storybundle.test');
    },
  );

  test('Dart preflight rejects oversized bytes without invoking FFI', () async {
    final bridge = FakeStoryBundleBindings();
    final loader = StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      limits: const StoryBundleLimits(maxArchiveBytes: 3),
      bindings: bridge,
    );

    await expectLater(
      loader.openBytes(Uint8List(4)),
      throwsA(isA<StoryBundleLimitException>()),
    );
    expect(bridge.openBytesCalls, 0);
  });

  test('trust keys reject malformed raw public keys before FFI', () {
    expect(() => StoryBundleTrustKey(Uint8List(31)), throwsArgumentError);
  });
}
