import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart'
    show StoryBundleLimits, StoryBundleBridgeRequest;

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
      expect(player.resolvedLocale, isNull);
      expect(player.hasUnresolvedLocalization, isFalse);
    },
  );

  test('invalid choices are rejected before the bridge', () async {
    final player = await StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: FakeStoryBundlePlayerBindings(),
    ).openBytes(Uint8List(0));
    expect(() => player.choose(-1), throwsA(isA<StoryBundlePlayerException>()));
  });

  test('fused bytes and paths pass ordered immutable preferences', () async {
    final bindings = FakeStoryBundlePlayerBindings()..localized = true;
    final loader = StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bindings,
    );
    final tags = ['id-ID', 'en'];
    final locales = StoryBundlePlayerLocalePreferences(tags);
    tags[0] = 'fr';
    expect(() => locales.locales.add('fr'), throwsUnsupportedError);
    final bytes = Uint8List.fromList([1, 2]);
    final player = await loader.openBytes(bytes, locales: locales);
    bytes[0] = 99;
    expect(bindings.receivedBytes, [1, 2]);
    expect(bindings.requested, ['id-ID', 'en']);
    expect(player.resolvedLocale, 'id');
    final path = await loader.openPath(
      'bundle',
      locales: StoryBundlePlayerLocalePreferences(['fr']),
    );
    expect(bindings.requested, ['fr']);
    expect(path.resolvedLocale, 'en');
    expect(player.resolvedLocale, 'id');
  });

  test(
    'restore preflights archive before allocating copies or calling bridge',
    () {
      final bindings = FakeStoryBundlePlayerBindings();
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
        bundleLimits: const StoryBundleLimits(maxArchiveBytes: 1),
      );
      expect(
        () => loader.restoreBytes(Uint8List(2), Uint8List(0)),
        throwsA(
          predicate<StoryBundlePlayerException>(
            (e) => e.error.code == 'B_RESOURCE_LIMIT',
          ),
        ),
      );
      expect(bindings.receivedBytes, isNull);
    },
  );

  test(
    'legacy injected bindings work by default and reject explicit preferences',
    () async {
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: _LegacyBindings(),
      );
      final player = await loader.openBytes(Uint8List(0));
      expect(player.resolvedLocale, isNull);
      expect(
        () => loader.openBytes(
          Uint8List(0),
          locales: StoryBundlePlayerLocalePreferences(['id']),
        ),
        throwsA(
          predicate<StoryBundlePlayerException>(
            (e) => e.error.code == 'R_LOCALIZATION_BINDINGS',
          ),
        ),
      );
    },
  );
}

// Proves the pre-localization interface remains implementable without locale args.
final class _LegacyBindings implements StoryBundlePlayerBindings {
  @override
  Future<StoryBundlePlayerBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) async => StoryBundlePlayerBridgePayload(
    resource: Object(),
    current: makePlayerDelta(),
  );
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
