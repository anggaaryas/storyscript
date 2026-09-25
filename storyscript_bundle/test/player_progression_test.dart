import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import 'test_support.dart';

void main() {
  test('progression stays compact and history is explicitly paged', () async {
    final player = await StoryBundlePlayerLoader(
      trustStore: StoryBundleTrustStore.empty(),
      bindings: FakeStoryBundlePlayerBindings(),
    ).openBytes(Uint8List(0));
    final delta = await player.advance();
    expect(delta.event.text, 'hello');
    expect(delta.sequence, 1);
    final page = await player.history(startSequence: 0, maximum: 1);
    expect(page.entries, hasLength(1));
    expect(page.nextSequence, 1);
  });
}
