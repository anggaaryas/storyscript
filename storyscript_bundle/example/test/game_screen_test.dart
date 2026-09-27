import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';
import 'package:storyscript_bundle_example/app.dart';
import 'package:storyscript_bundle_example/features/bundle_inspector/bundle_inspector_controller.dart';
import 'package:storyscript_bundle_example/features/game/game_controller.dart';

import 'test_support.dart';

void main() {
  testWidgets(
    'plays choices, displays artwork, restarts, and opens inspector',
    (tester) async {
      final binding = _GameBindings();
      await tester.pumpWidget(_app(binding));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('game-background')), findsOneWidget);
      expect(find.text('CHAPTER ONE  /  A BORROWED DAWN'), findsOneWidget);
      expect(find.text('Entering arrival…'), findsOneWidget);

      await _continue(tester);
      expect(find.text('The station is overheating.'), findsOneWidget);
      await _continue(tester);
      expect(find.byKey(const Key('game-portrait')), findsOneWidget);
      expect(find.text('Dot'), findsOneWidget);
      await _continue(tester);
      expect(find.text('Patch the line'), findsOneWidget);
      expect(find.bySemanticsLabel('Option 2: Search'), findsOneWidget);
      await tester.tap(find.byKey(const Key('game-choice-1')));
      await tester.pumpAndSettle();
      expect(binding.choices, [1]);
      expect(find.text('You found the key.'), findsOneWidget);
      await _continue(tester);
      expect(find.text('Use the override'), findsOneWidget);
      await tester.tap(find.byKey(const Key('game-choice-1')));
      await tester.pumpAndSettle();
      expect(find.text('The station is saved.'), findsOneWidget);
      await _continue(tester);
      expect(find.text('The End'), findsOneWidget);
      await tester.tap(find.byKey(const Key('game-restart')));
      await tester.pumpAndSettle();
      expect(binding.openCalls, 2);
      expect(binding.disposeCalls, 1);

      await tester.tap(find.byTooltip('Open demo Bundle Inspector'));
      await tester.pumpAndSettle();
      expect(find.text('StoryBundle Inspector'), findsOneWidget);
      await tester.pageBack();
      await tester.pumpAndSettle();
      expect(find.text('Entering arrival…'), findsOneWidget);
      await tester.pumpWidget(const SizedBox());
      await tester.pumpAndSettle();
      expect(binding.disposeCalls, 2);
    },
  );

  testWidgets(
    'reports verification failure and retries without unsigned fallback',
    (tester) async {
      final binding = _GameBindings()..reject = true;
      await tester.pumpWidget(_app(binding));
      await tester.pumpAndSettle();
      expect(find.textContaining('B_BAD_SIGNATURE'), findsOneWidget);
      binding.reject = false;
      await tester.tap(find.text('Retry'));
      await tester.pumpAndSettle();
      expect(find.text('Entering arrival…'), findsOneWidget);
      expect(binding.openCalls, 2);
    },
  );

  testWidgets('disposes a player returned after the screen is removed', (
    tester,
  ) async {
    final binding = _GameBindings();
    final pending = Completer<StoryBundlePlayerBridgePayload>();
    binding.pendingOpen = pending;
    await tester.pumpWidget(_app(binding));
    await tester.pump();
    await tester.pumpWidget(const SizedBox());
    pending.complete(binding.payload);
    await tester.pumpAndSettle();
    expect(binding.disposeCalls, 1);
    expect(tester.takeException(), isNull);
  });

  testWidgets('prevents duplicate steps and keeps text playable without art', (
    tester,
  ) async {
    final binding = _GameBindings()..artFails = true;
    await tester.pumpWidget(_app(binding));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('art-warning')), findsOneWidget);
    expect(find.byKey(const Key('game-background')), findsNothing);
    final pending = Completer<StoryBundlePlayerDelta>();
    binding.pendingAdvance = pending;
    await tester.tap(find.byKey(const Key('game-next')));
    await tester.pump();
    expect(
      tester.widget<FilledButton>(find.byKey(const Key('game-next'))).onPressed,
      isNull,
    );
    expect(binding.advanceCalls, 1);
    binding.pendingAdvance = null;
    pending.complete(
      _delta(
        StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.narration,
          text: 'Still playable.',
        ),
        sequence: 1,
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Still playable.'), findsOneWidget);
    expect(binding.advanceCalls, 1);
  });

  testWidgets('failed choice keeps the previous checkpoint available', (
    tester,
  ) async {
    final binding = _GameBindings()..failChoice = true;
    await tester.pumpWidget(_app(binding));
    await tester.pumpAndSettle();
    await _continue(tester);
    await _continue(tester);
    await _continue(tester);
    await tester.tap(find.byKey(const Key('game-choice-0')));
    await tester.pumpAndSettle();
    expect(find.textContaining('R_GAME_FAILURE'), findsOneWidget);
    expect(find.text('Patch the line'), findsOneWidget);
    binding.failChoice = false;
    await tester.tap(find.byKey(const Key('game-choice-0')));
    await tester.pumpAndSettle();
    expect(find.text('You found the key.'), findsOneWidget);
    expect(find.byKey(const Key('game-error')), findsNothing);
  });

  testWidgets('faulted player offers a new session', (tester) async {
    final binding = _GameBindings()..faultAfterFirstAdvance = true;
    await tester.pumpWidget(_app(binding));
    await tester.pumpAndSettle();
    await _continue(tester);
    expect(find.textContaining('R_STORY_FAULT'), findsOneWidget);
    expect(find.byKey(const Key('game-restart')), findsOneWidget);
    await tester.tap(find.byKey(const Key('game-restart')));
    await tester.pumpAndSettle();
    expect(binding.disposeCalls, 1);
    expect(find.text('Entering arrival…'), findsOneWidget);
  });

  testWidgets('long pages scroll and the next beat starts at the top', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(320, 600));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final binding = _GameBindings()..longNarration = true;
    await tester.pumpWidget(_app(binding));
    await tester.pumpAndSettle();
    await _continue(tester);
    await tester.ensureVisible(find.byKey(const Key('game-next')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('game-next')));
    await tester.pumpAndSettle();
    expect(find.text('CHAPTER ONE  /  A BORROWED DAWN'), findsOneWidget);
    expect(
      tester.getTopLeft(find.text('CHAPTER ONE  /  A BORROWED DAWN')).dy,
      greaterThanOrEqualTo(0),
    );
  });
}

Future<void> _continue(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('game-next')));
  await tester.pumpAndSettle();
}

Widget _app(_GameBindings binding) => BundleInspectorApp(
  controller: BundleInspectorController(
    loader: StoryBundleLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: InspectorFakeBindings(),
    ),
    fixtureBytes: () async => Uint8List(1),
  ),
  gameController: GameController(
    loader: StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: binding,
    ),
    bundleBytes: () async => Uint8List(1),
  ),
);

StoryBundlePlayerDelta _delta(
  StoryBundlePlayerEvent event, {
  int sequence = 0,
  bool finished = false,
  List<StoryBundlePlayerEffect> effects = const [],
}) => StoryBundlePlayerDelta(
  event: event,
  effects: effects,
  scene: 'arrival',
  status: finished
      ? StoryBundlePlayerStatus.finished
      : StoryBundlePlayerStatus.active,
  sequence: sequence,
  firstRetainedSequence: 0,
  omittedHistoryCount: 0,
);

final class _GameBindings implements StoryBundlePlayerBindings {
  int openCalls = 0;
  int disposeCalls = 0;
  int step = 0;
  bool reject = false;
  bool failChoice = false;
  bool faultAfterFirstAdvance = false;
  bool longNarration = false;
  bool artFails = false;
  int advanceCalls = 0;
  Completer<StoryBundlePlayerBridgePayload>? pendingOpen;
  Completer<StoryBundlePlayerDelta>? pendingAdvance;
  final List<int> choices = [];

  StoryBundlePlayerBridgePayload get payload => StoryBundlePlayerBridgePayload(
    resource: Object(),
    current: _delta(
      StoryBundlePlayerEvent(
        kind: StoryBundlePlayerEventKind.scene,
        scene: 'arrival',
      ),
      effects: const [
        StoryBundlePlayerEffect(
          kind: 'background',
          path: 'backgrounds/station.svg',
        ),
      ],
    ),
  );

  @override
  Future<StoryBundlePlayerBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) {
    openCalls++;
    step = 0;
    if (reject) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'B_BAD_SIGNATURE',
          scene: '',
          message: 'Untrusted game',
        ),
      );
    }
    return pendingOpen?.future ?? Future.value(payload);
  }

  @override
  Future<StoryBundlePlayerDelta> advance(Object resource) async {
    advanceCalls++;
    if (pendingAdvance != null) return pendingAdvance!.future;
    step++;
    if (faultAfterFirstAdvance && step == 1) {
      return StoryBundlePlayerDelta(
        event: StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.error,
          error: const StoryBundleRuntimeError(
            code: 'R_STORY_FAULT',
            scene: 'arrival',
            message: 'Player stopped',
          ),
        ),
        effects: const [],
        scene: 'arrival',
        status: StoryBundlePlayerStatus.faulted,
        sequence: 1,
        firstRetainedSequence: 0,
        omittedHistoryCount: 0,
      );
    }
    return switch (step) {
      1 => _delta(
        StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.narration,
          text: longNarration
              ? List.filled(100, 'The station is overheating.').join(' ')
              : 'The station is overheating.',
        ),
        sequence: step,
      ),
      2 => _delta(
        StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.dialogue,
          actorName: 'Dot',
          portraitPath: 'portraits/dot.svg',
          text: 'Choose quickly.',
        ),
        sequence: step,
      ),
      3 || 5 => _delta(
        StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.choices,
          choices: [
            StoryBundlePlayerChoice(
              text: step == 3 ? 'Patch the line' : 'Seal',
              targetScene: 'seal',
            ),
            StoryBundlePlayerChoice(
              text: step == 3 ? 'Search' : 'Use the override',
              targetScene: 'override',
            ),
          ],
        ),
        sequence: step,
      ),
      _ => _delta(
        StoryBundlePlayerEvent(kind: StoryBundlePlayerEventKind.end),
        sequence: step,
        finished: true,
      ),
    };
  }

  @override
  Future<StoryBundlePlayerDelta> choose(Object resource, int index) async {
    if (failChoice) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_GAME_FAILURE',
          scene: 'arrival',
          message: 'Choice could not advance',
        ),
      );
    }
    choices.add(index);
    step++;
    return _delta(
      StoryBundlePlayerEvent(
        kind: StoryBundlePlayerEventKind.narration,
        text: step == 4 ? 'You found the key.' : 'The station is saved.',
      ),
      sequence: step,
    );
  }

  @override
  Future<void> dispose(Object resource) async {
    disposeCalls++;
  }

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String path,
    int maximumBytes,
  ) async {
    if (artFails) throw StateError('Artwork missing');
    return Uint8List.fromList(
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>'
          .codeUnits,
    );
  }

  @override
  Future<StoryBundlePlayerDelta> current(Object resource) =>
      throw UnimplementedError();
  @override
  Future<Uint8List> exportSave(Object resource) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  ) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerBridgePayload> openFromBundle(
    Object bundleResource,
    StoryBundlePlayerLimits limits,
  ) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerBridgePayload> restoreBytes(
    Uint8List bytes,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => throw UnimplementedError();
  @override
  Future<StoryBundlePlayerBridgePayload> restoreFromBundle(
    Object bundleResource,
    Uint8List save,
    StoryBundlePlayerLimits limits,
  ) => throw UnimplementedError();
}
