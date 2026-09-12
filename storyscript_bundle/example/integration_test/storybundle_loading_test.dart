import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(RustLib.init);

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
