// Non-UI controller tests: deliberately no testWidgets, rendering or native FFI.
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import '../../test/test_support.dart';
import 'package:storyscript_bundle_example/features/game/game_controller.dart';

void main() {
  test(
    'candidate locale restore rerenders exact sequence and rolls back failure',
    () async {
      final bindings = FakeStoryBundlePlayerBindings()..localized = true;
      final game = GameController(
        loader: StoryBundlePlayerLoader(
          trustStore: StoryBundleTrustStore.empty(),
          bindings: bindings,
        ),
        bundleBytes: () async => Uint8List.fromList([7, 8]),
      );
      game.setInitialLocales(['en']);
      await game.start();
      await game.advance();
      final before = game.delta!;
      expect(before.event.text, 'Hello');
      final switching = game.changeLocales(['id']);
      expect(game.busy, true);
      expect(await game.changeLocales(['en']), false);
      expect(await switching, true);
      expect(game.delta!.event.text, 'Halo');
      expect(game.delta!.sequence, before.sequence);
      expect(game.resolvedLocale, 'id');
      expect(bindings.receivedSave, [1, 2, 3]);
      expect(bindings.disposeCalls, 1);
      final stable = game.delta;
      bindings.failure = const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_LOCALIZATION_FORMAT',
          scene: '',
          message: 'resolver failure',
        ),
      );
      expect(await game.changeLocales(['en']), false);
      expect(identical(game.delta, stable), true);
      expect(game.resolvedLocale, 'id');
      expect(game.requestedLocales, ['id']);
      expect(game.phase, GamePhase.playing);
      expect(game.busy, false);
      expect(bindings.disposeCalls, 1);
      expect(game.error, contains('R_LOCALIZATION_FORMAT'));
      game.dispose();
    },
  );

  test(
    'verified bundle reused and artwork retained without PREP/media reads',
    () async {
      final bundles = FakeStoryBundleBindings();
      final players = FakeStoryBundlePlayerBindings()..localized = true;
      players.currentDelta = StoryBundlePlayerDelta(
        event: StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.dialogue,
          text: 'Hello',
          portraitPath: 'portraits/dot.svg',
        ),
        effects: const [
          StoryBundlePlayerEffect(
            kind: 'background',
            path: 'backgrounds/station.svg',
          ),
        ],
        scene: 'arrival',
        status: StoryBundlePlayerStatus.active,
        sequence: 4,
        firstRetainedSequence: 0,
        omittedHistoryCount: 0,
      );
      final game = GameController(
        loader: StoryBundlePlayerLoader(
          trustStore: StoryBundleTrustStore.empty(),
          bindings: players,
        ),
        bundleLoader: StoryBundleLoader(
          trustStore: StoryBundleTrustStore.empty(),
          bindings: bundles,
        ),
        bundleBytes: () async => Uint8List.fromList([7, 8]),
      );
      game.setInitialLocales(['en']);
      await game.start();
      final background = game.background;
      final portrait = game.portrait;
      final bundle = players.receivedBundle;
      expect(players.assetCalls, 2);
      expect(await game.changeLocales(['id']), true);
      expect(identical(players.receivedBundle, bundle), true);
      expect(players.assetCalls, 2);
      expect(identical(game.background, background), true);
      expect(identical(game.portrait, portrait), true);
      expect(game.delta!.sequence, 4);
      expect(bundles.openBytesCalls, 1);
      expect(players.openBytesCalls, 0);
      players.failure = const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_LOCALIZATION_FORMAT',
          scene: '',
          message: 'resolver failure',
        ),
      );
      expect(await game.changeLocales(['en']), false);
      expect(identical(game.background, background), true);
      expect(identical(game.portrait, portrait), true);
      expect(game.resolvedLocale, 'id');
      game.dispose();
      await Future<void>.delayed(Duration.zero);
      expect(bundles.disposeCalls, 1);
    },
  );

  test('disposal during save prevents candidate publication', () async {
    final players = FakeStoryBundlePlayerBindings()..localized = true;
    final game = GameController(
      loader: StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: players,
      ),
      bundleBytes: () async => Uint8List.fromList([7, 8]),
    );
    game.setInitialLocales(['en']);
    await game.start();
    final before = game.delta;
    final pending = game.changeLocales(['id']);
    game.dispose();
    expect(await pending, false);
    expect(identical(game.delta, before), true);
    expect(game.resolvedLocale, 'en');
    expect(players.disposeCalls, 1);
  });
}
