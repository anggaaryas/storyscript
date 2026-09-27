import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Web build commands write Wasm into the Flutter example', () {
    final packageDirectory = Directory.current.absolute;
    // FRB invokes wasm-pack from rust/, so --output is relative to that crate.
    final rustDirectory = packageDirectory.uri.resolve('rust/');
    final exampleWeb = packageDirectory.uri
        .resolve('example/web')
        .normalizePath();
    final instructions = [
      File.fromUri(packageDirectory.uri.resolve('README.md')),
      File.fromUri(packageDirectory.uri.resolve('example/README.md')),
      File.fromUri(
        packageDirectory.uri.resolve('../.github/workflows/storybundle-ci.yml'),
      ),
    ];

    for (final file in instructions) {
      final matches = RegExp(
        r'flutter_rust_bridge_codegen build-web\s+--output\s+([^\s`]+)',
      ).allMatches(file.readAsStringSync()).toList();
      expect(matches, hasLength(1), reason: file.path);
      final output = matches.single.group(1)!;
      expect(
        rustDirectory.resolve(output).normalizePath(),
        exampleWeb,
        reason: file.path,
      );
    }
  });
}
