import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_player_core/storyscript_player_core.dart';

void main() {
  test(
    'injected headless facade maps compact progression and opaque saves',
    () async {
      final bindings = _FakeBindings();
      final player = await SourceStoryPlayerLoader(
        bindings: bindings,
      ).openSource('story');
      expect(player.current.event.kind, StoryPlayerEventKind.scene);
      expect((await player.advance()).event.text, 'hello');
      expect(
        (await player.history(startSequence: 0, maximum: 1)).entries,
        hasLength(1),
      );
      final save = await player.exportSave();
      save[0] = 99;
      expect((await player.exportSave())[0], 1);
      await player.dispose();
      await player.dispose();
      expect(bindings.disposeCalls, 1);
      expect(() => player.current, throwsA(isA<StoryPlayerException>()));
    },
  );

  test('choice bounds and bridge errors remain structured', () async {
    final bindings = _FakeBindings();
    final player = await SourceStoryPlayerLoader(
      bindings: bindings,
    ).openSource('story');
    expect(() => player.choose(-1), throwsA(isA<StoryPlayerException>()));
    bindings.failure = const StoryPlayerException(
      StoryPlayerError(
        code: 'R_INVALID_ACTION',
        scene: 'first',
        message: 'choice required',
      ),
    );
    await expectLater(
      player.advance(),
      throwsA(
        predicate<StoryPlayerException>(
          (error) =>
              error.error.code == 'R_INVALID_ACTION' &&
              error.error.scene == 'first',
        ),
      ),
    );
  });

  test(
    'project preferences and resolution are immutable and saves are copied',
    () async {
      final tags = ['id-ID', 'en'];
      final locales = StoryPlayerLocalePreferences(tags);
      tags[0] = 'fr';
      expect(locales.locales, ['id-ID', 'en']);
      expect(() => locales.locales.add('fr'), throwsUnsupportedError);
      final bindings = _FakeBindings();
      final loader = SourceStoryPlayerLoader(bindings: bindings);
      final player = await loader.openProject('project', locales: locales);
      expect(bindings.requested, ['id-ID', 'en']);
      expect(player.resolvedLocale, 'id');
      expect(player.hasUnresolvedLocalization, isFalse);
      final save = Uint8List.fromList([1, 2, 3]);
      final restored = await loader.restoreProject(
        'project',
        save,
        locales: StoryPlayerLocalePreferences(['en']),
      );
      save[0] = 99;
      expect(bindings.receivedSave, [1, 2, 3]);
      expect(restored.resolvedLocale, 'en');
      expect(restored.current.event.text, 'Hello');
      expect(player.resolvedLocale, 'id');
      final fallback = await loader.openProject(
        'project',
        locales: StoryPlayerLocalePreferences(['fr']),
      );
      expect(fallback.resolvedLocale, 'en');
      final raw = await loader.openSource('@"greeting"');
      expect(raw.resolvedLocale, isNull);
      expect(raw.hasUnresolvedLocalization, isTrue);
    },
  );

  test(
    'failed project candidate keeps existing player and structured error',
    () async {
      final bindings = _FakeBindings();
      final loader = SourceStoryPlayerLoader(bindings: bindings);
      final old = await loader.openProject('project');
      bindings.failure = const StoryPlayerException(
        StoryPlayerError(
          code: 'R_LOCALIZATION_NUMBER',
          scene: 'first',
          message: 'unsafe number',
        ),
      );
      await expectLater(
        loader.restoreProject('project', Uint8List(0)),
        throwsA(
          predicate<StoryPlayerException>(
            (e) => e.error.code == 'R_LOCALIZATION_NUMBER',
          ),
        ),
      );
      expect(old.current.event.kind, StoryPlayerEventKind.scene);
      expect(bindings.disposeCalls, 0);
    },
  );
}

StoryPlayerDelta _delta(StoryPlayerEvent event, [int sequence = 0]) =>
    StoryPlayerDelta(
      event: event,
      effects: const [],
      scene: 'first',
      status: StoryPlayerStatus.active,
      sequence: sequence,
      firstRetainedSequence: 0,
      omittedHistoryCount: 0,
    );

final class _FakeBindings implements SourcePlayerBindings {
  final resource = Object();
  var currentDelta = _delta(
    StoryPlayerEvent(kind: StoryPlayerEventKind.scene, scene: 'first'),
  );
  StoryPlayerException? failure;
  int disposeCalls = 0;
  List<String> requested = [];
  Uint8List? receivedSave;

  @override
  Future<SourcePlayerBridgePayload> openProject(
    String root,
    StoryPlayerLocalePreferences locales,
    StoryPlayerLimits limits,
  ) async {
    if (failure case final error?) throw error;
    requested = locales.locales;
    return SourcePlayerBridgePayload(
      resource: resource,
      current: currentDelta,
      locale: StoryPlayerLocaleResolution(
        resolvedLocale: requested.firstOrNull?.startsWith('id') == true
            ? 'id'
            : 'en',
      ),
    );
  }

  @override
  Future<SourcePlayerBridgePayload> restoreProject(
    String root,
    Uint8List save,
    StoryPlayerLocalePreferences locales,
    StoryPlayerLimits limits,
  ) async {
    receivedSave = save;
    final payload = await openProject(root, locales, limits);
    return SourcePlayerBridgePayload(
      resource: resource,
      current: _delta(
        StoryPlayerEvent(kind: StoryPlayerEventKind.narration, text: 'Hello'),
        1,
      ),
      locale: payload.locale,
    );
  }

  @override
  Future<StoryPlayerDelta> advance(Object resource) async {
    if (failure case final error?) throw error;
    return currentDelta = _delta(
      StoryPlayerEvent(kind: StoryPlayerEventKind.narration, text: 'hello'),
      1,
    );
  }

  @override
  Future<StoryPlayerDelta> choose(Object resource, int index) async =>
      currentDelta;

  @override
  Future<StoryPlayerDelta> current(Object resource) async => currentDelta;

  @override
  Future<void> dispose(Object resource) async {
    disposeCalls++;
  }

  @override
  Future<Uint8List> exportSave(Object resource) async =>
      Uint8List.fromList([1, 2, 3]);

  @override
  Future<StoryPlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  ) async => StoryPlayerHistoryPage(
    entries: [
      StoryPlayerHistoryEntry(
        sequence: 0,
        event: currentDelta.event,
        effects: const [],
        scene: 'first',
      ),
    ],
    nextSequence: 1,
    firstRetainedSequence: 0,
    omittedHistoryCount: 0,
  );

  @override
  Future<SourcePlayerBridgePayload> openPath(
    String path,
    StoryPlayerLimits limits,
  ) => openSource(path, limits);

  @override
  Future<SourcePlayerBridgePayload> openSource(
    String source,
    StoryPlayerLimits limits,
  ) async => SourcePlayerBridgePayload(
    resource: resource,
    current: currentDelta,
    locale: StoryPlayerLocaleResolution(
      hasUnresolvedLocalization: source.contains('@"'),
    ),
  );

  @override
  Future<SourcePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryPlayerLimits limits,
  ) => restoreSource(path, save, limits);

  @override
  Future<SourcePlayerBridgePayload> restoreSource(
    String source,
    Uint8List save,
    StoryPlayerLimits limits,
  ) => openSource(source, limits);
}
