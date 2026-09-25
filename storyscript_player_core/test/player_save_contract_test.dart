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
  ) async =>
      SourcePlayerBridgePayload(resource: resource, current: currentDelta);

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
