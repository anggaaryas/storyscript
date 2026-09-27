import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';
import 'package:storyscript_bundle/src/rust/api/bundle.dart' as rust;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(RustLib.init);

  testWidgets('WebAssembly bridge initializes its worker pool', (tester) async {
    final limits = await rust.bridgeHardLimits();

    expect(limits.maxArchiveBytes, BigInt.from(100 * 1024 * 1024));
    expect(limits.maxEntries, 4096);
  });

  testWidgets(
    'Rust verifies fixture, Dart decodes model, reads asset, disposes',
    (tester) async {
      final loader = await _loader();
      final bytes = await _fixtureBytes();

      final loaded = await loader.openBytes(bytes);

      expect(loaded.story.project.id, 'storybundle.demo');
      expect(loaded.story.scenes, isNotEmpty);
      expect(loaded.manifest.schemaSha256, storyBundleSchemaSha256);
      expect(loaded.verification.isStrictlyVerified, isTrue);
      expect(await loaded.readAsset('portraits/hero.svg'), isNotEmpty);
      await loaded.dispose();
      await expectLater(
        loaded.readAsset('portraits/hero.svg'),
        throwsA(isA<StoryBundleDisposedException>()),
      );
    },
  );

  testWidgets('tampered fixture is rejected before model exposure', (
    tester,
  ) async {
    final loader = await _loader();
    final bytes = await _fixtureBytes();
    final marker = 'StoryBundle Demo'.codeUnits;
    final markerIndex = _indexOf(bytes, marker);
    expect(markerIndex, isNonNegative);
    bytes[markerIndex] ^= 1;

    await expectLater(
      loader.openBytes(bytes),
      throwsA(isA<StoryBundleException>()),
    );
  });

  testWidgets('signed Station Nine game reaches a choice-dependent ending', (
    tester,
  ) async {
    final keyHex = (await rootBundle.loadString(
      'assets/station_nine_public_key.txt',
    )).trim();
    final bytes = await rootBundle.load('assets/station_nine.storybundle');
    final archive = Uint8List.fromList(
      bytes.buffer.asUint8List(bytes.offsetInBytes, bytes.lengthInBytes),
    );
    final loader = StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore([
        StoryBundleTrustKey(_decodeHex(keyHex)),
      ]),
    );
    final player = await loader.openBytes(archive);
    try {
      expect(player.current.event.kind, StoryBundlePlayerEventKind.scene);
      expect(player.current.effects.single.kind, 'background');
      expect(await player.readAsset('backgrounds/station.svg'), isNotEmpty);
      expect(await player.readAsset('backgrounds/archive.svg'), isNotEmpty);
      expect(await player.readAsset('portraits/dot_dim.svg'), isNotEmpty);
      var choices = 0;
      var saved = false;
      var pages = 0;
      for (var step = 0; step < 160; step++) {
        final delta = player.current;
        if (delta.event.kind == StoryBundlePlayerEventKind.end) break;
        pages++;
        if (delta.event.kind == StoryBundlePlayerEventKind.narration &&
            (delta.event.text ?? '').contains('Station Nine is saved')) {
          saved = true;
        }
        if (delta.event.kind == StoryBundlePlayerEventKind.choices) {
          expect(delta.event.choices.length, choices == 0 ? 2 : 3);
          choices++;
          await player.choose(1); // Search for the key, then use it.
        } else {
          await player.advance();
        }
      }
      expect(choices, 2);
      expect(pages, greaterThan(20));
      expect(saved, isTrue);
      expect(player.current.event.kind, StoryBundlePlayerEventKind.end);
    } finally {
      await player.dispose();
    }
    await expectLater(
      StoryBundlePlayerLoader(
        trustStore: StoryBundleTrustStore.empty(),
      ).openBytes(archive),
      throwsA(isA<StoryBundlePlayerException>()),
    );
  });
}

Future<StoryBundleLoader> _loader() async {
  final keyHex = (await rootBundle.loadString(
    'assets/demo_public_key.txt',
  )).trim();
  return StoryBundleLoader(
    trustStore: StoryBundleTrustStore([
      StoryBundleTrustKey(_decodeHex(keyHex)),
    ]),
  );
}

Future<Uint8List> _fixtureBytes() async {
  final data = await rootBundle.load('assets/demo.storybundle');
  return Uint8List.fromList(
    data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
  );
}

Uint8List _decodeHex(String value) => Uint8List.fromList([
  for (var index = 0; index < value.length; index += 2)
    int.parse(value.substring(index, index + 2), radix: 16),
]);

int _indexOf(List<int> bytes, List<int> marker) {
  for (var start = 0; start <= bytes.length - marker.length; start++) {
    var matches = true;
    for (var offset = 0; offset < marker.length; offset++) {
      if (bytes[start + offset] != marker[offset]) {
        matches = false;
        break;
      }
    }
    if (matches) return start;
  }
  return -1;
}
