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
  test(
    'cross-locale bytes/path candidates copy saves and keep old resolution',
    () async {
      final bindings = FakeStoryBundlePlayerBindings()..localized = true;
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      );
      final old = await loader.openBytes(
        Uint8List(0),
        locales: StoryBundlePlayerLocalePreferences(['id']),
      );
      await old.advance();
      final save = await old.exportSave();
      final candidate = await loader.restoreBytes(
        Uint8List(0),
        save,
        locales: StoryBundlePlayerLocalePreferences(['en']),
      );
      save[0] = 99;
      expect(bindings.receivedSave, [1, 2, 3]);
      expect(candidate.resolvedLocale, 'en');
      expect(candidate.current.sequence, old.current.sequence);
      expect(candidate.current.event.text, 'Hello');
      expect(old.current.event.text, 'Halo');
      expect(old.resolvedLocale, 'id');
      final path = await loader.restorePath(
        'bundle',
        Uint8List.fromList([1]),
        locales: StoryBundlePlayerLocalePreferences(['id-ID', 'en']),
      );
      expect(path.resolvedLocale, 'id');
      expect(bindings.requested, ['id-ID', 'en']);
    },
  );

  test(
    'failed localized candidate preserves old checkpoint and error fields',
    () async {
      final bindings = FakeStoryBundlePlayerBindings()..localized = true;
      final loader = StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      );
      final old = await loader.openBytes(
        Uint8List(0),
        locales: StoryBundlePlayerLocalePreferences(['id']),
      );
      final checkpoint = old.current;
      bindings.failure = const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_LOCALIZATION_NUMBER',
          scene: 'start',
          message: 'unsafe number',
        ),
      );
      await expectLater(
        loader.restoreBytes(Uint8List(0), Uint8List(0)),
        throwsA(
          predicate<StoryBundlePlayerException>(
            (e) =>
                e.error.code == 'R_LOCALIZATION_NUMBER' &&
                e.error.scene == 'start',
          ),
        ),
      );
      expect(old.current, same(checkpoint));
      expect(old.resolvedLocale, 'id');
      expect(bindings.disposeCalls, 0);
    },
  );
}
