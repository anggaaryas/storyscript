import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

import 'test_support.dart';

void main() {
  test('ordinary constructor always requests strict verification', () async {
    final bridge = FakeStoryBundleBindings();
    final loader = StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bridge,
    );

    await loader.openBytes(Uint8List(1));

    expect(bridge.lastRequest?.policy, StoryBundleVerificationPolicy.strict);
  });

  test('unsigned development is explicit and remains visible', () async {
    final bridge = FakeStoryBundleBindings(
      payload: makeBridgePayload(unsignedDevelopment: true),
    );
    final loader = StoryBundleLoader.unsignedDevelopment(bindings: bridge);

    final loaded = await loader.openBytes(Uint8List(1));

    expect(
      bridge.lastRequest?.policy,
      StoryBundleVerificationPolicy.unsignedDevelopment,
    );
    expect(loaded.verification.isUnsignedDevelopment, isTrue);
    expect(loaded.verification.isStrictlyVerified, isFalse);
  });

  test('bridge error codes map to structured exception families', () {
    expect(
      StoryBundleException.fromCode('B_UNKNOWN_SIGNER', 'unknown'),
      isA<StoryBundleTrustException>(),
    );
    expect(
      StoryBundleException.fromCode('B_BAD_SIGNATURE', 'bad'),
      isA<StoryBundleVerificationException>(),
    );
    expect(
      StoryBundleException.fromCode('B_SCHEMA_MISMATCH', 'schema'),
      isA<StoryBundleCompatibilityException>(),
    );
  });
}
