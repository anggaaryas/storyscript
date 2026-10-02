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
      playerBindings.localized = true;
      final player = await StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: playerBindings,
      ).fromBundle(loaded, locales: StoryBundlePlayerLocalePreferences(['id']));
      expect(playerBindings.requested, ['id']);
      expect(player.resolvedLocale, 'id');
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
    bindings.localized = true;
    final pending = loader.openBytes(
      Uint8List(0),
      locales: StoryBundlePlayerLocalePreferences(['id-ID']),
    );
    loader.cancelPendingLoads();
    bindings.pendingOpen!.complete(bindings.payload);
    await expectLater(pending, throwsA(isA<StoryBundlePlayerException>()));
    expect(bindings.disposeCalls, 1);
  });

  test('newer locale load wins and older candidate is disposed', () async {
    final bindings = FakeStoryBundlePlayerBindings()..localized = true;
    final oldCompletion = Completer<StoryBundlePlayerBridgePayload>();
    bindings.pendingOpen = oldCompletion;
    final loader = StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: bindings,
    );
    final old = loader.openBytes(
      Uint8List(0),
      locales: StoryBundlePlayerLocalePreferences(['id']),
    );
    final oldPayload = bindings.payload;
    bindings.pendingOpen = null;
    final current = await loader.openBytes(
      Uint8List(0),
      locales: StoryBundlePlayerLocalePreferences(['en']),
    );
    oldCompletion.complete(oldPayload);
    await expectLater(
      old,
      throwsA(
        predicate<StoryBundlePlayerException>(
          (e) => e.error.code == 'R_STALE_LOAD',
        ),
      ),
    );
    expect(current.resolvedLocale, 'en');
    expect(bindings.disposeCalls, 1);
  });

  test(
    'existing verified-bundle restore forwards locales and copies save',
    () async {
      final bundle = await StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: FakeStoryBundleBindings(),
      ).openBytes(Uint8List(0));
      final bindings = FakeStoryBundlePlayerBindings()..localized = true;
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      );
      final save = Uint8List.fromList([1, 2]);
      final candidate = await loader.restoreFromBundle(
        bundle,
        save,
        locales: StoryBundlePlayerLocalePreferences(['en', 'id']),
      );
      save[0] = 99;
      expect(bindings.receivedSave, [1, 2]);
      expect(bindings.requested, ['en', 'id']);
      expect(bindings.receivedBundle, isNotNull);
      expect(candidate.resolvedLocale, 'en');
      await candidate.dispose();
      final next = await loader.fromBundle(bundle);
      expect(next.resolvedLocale, 'en');
      await bundle.dispose();
      expect((await next.advance()).event.text, 'Hello');
      await next.dispose();
    },
  );
}
