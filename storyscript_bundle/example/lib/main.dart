import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

import 'app.dart';
import 'features/bundle_inspector/bundle_inspector_controller.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final publicKeyHex = (await rootBundle.loadString(
    'assets/demo_public_key.txt',
  )).trim();
  final loader = StoryBundleLoader(
    trustStore: StoryBundleTrustStore([
      StoryBundleTrustKey(_decodeHex(publicKeyHex)),
    ]),
  );
  runApp(
    BundleInspectorApp(
      controller: BundleInspectorController(
        loader: loader,
        fixtureBytes: () async {
          final data = await rootBundle.load('assets/demo.storybundle');
          return data.buffer.asUint8List(
            data.offsetInBytes,
            data.lengthInBytes,
          );
        },
      ),
    ),
  );
}

Uint8List _decodeHex(String value) {
  if (value.length.isOdd) {
    throw const FormatException('public key hex must have an even length');
  }
  return Uint8List.fromList([
    for (var index = 0; index < value.length; index += 2)
      int.parse(value.substring(index, index + 2), radix: 16),
  ]);
}
