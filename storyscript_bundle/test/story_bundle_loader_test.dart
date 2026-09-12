import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

import 'test_support.dart';

void main() {
  test(
    'loader binds typed model, manifest, verification, and progress',
    () async {
      final bridge = FakeStoryBundleBindings();
      final progress = <StoryBundleLoadProgress>[];
      final loader = StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bridge,
      );

      final loaded = await loader.openBytes(
        Uint8List(4),
        onProgress: progress.add,
      );

      expect(loaded.story.scenes.single.label, 'start');
      expect(loaded.manifest.schemaSha256, storyBundleSchemaSha256);
      expect(loaded.verification.isStrictlyVerified, isTrue);
      expect(progress, StoryBundleLoadProgress.values);
    },
  );

  test(
    'stale completion is rejected and its Rust resource is disposed',
    () async {
      final bridge = FakeStoryBundleBindings();
      bridge.pendingOpen = Completer<StoryBundleBridgePayload>();
      final loader = StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bridge,
      );
      final pending = loader.openBytes(Uint8List(4));

      loader.cancelPendingLoads();
      bridge.pendingOpen!.complete(bridge.payload);

      await expectLater(pending, throwsA(isA<StoryBundleStaleLoadException>()));
      expect(bridge.disposeCalls, 1);
    },
  );

  test('Dart decode failure disposes verified Rust resource', () async {
    final bridge = FakeStoryBundleBindings(
      payload: makeBridgePayload(
        protobufBytes: Uint8List.fromList(<int>[0xff]),
      ),
    );
    final loader = StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bridge,
    );

    await expectLater(
      loader.openBytes(Uint8List(1)),
      throwsA(isA<StoryBundleModelException>()),
    );
    expect(bridge.disposeCalls, 1);
  });
}
