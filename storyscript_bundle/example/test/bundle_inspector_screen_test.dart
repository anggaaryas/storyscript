import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle_example/app.dart';
import 'package:storyscript_bundle_example/features/bundle_inspector/bundle_inspector_controller.dart';

import 'test_support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('empty, loading, verified, preview, and disposed states', (
    tester,
  ) async {
    final bindings = InspectorFakeBindings();
    final fixture = Completer<Uint8List>();
    final controller = BundleInspectorController(
      loader: StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      ),
      fixtureBytes: () => fixture.future,
    );
    addTearDown(controller.dispose);
    await tester.pumpWidget(BundleInspectorApp(controller: controller));

    expect(find.text('No bundle loaded'), findsOneWidget);
    expect(tester.getSize(find.byKey(const Key('load-bundle'))).height, 48);

    await tester.tap(find.byKey(const Key('load-bundle')));
    await tester.pump();
    expect(find.textContaining('Loading'), findsOneWidget);
    fixture.complete(Uint8List(1));
    await tester.pumpAndSettle();

    expect(find.text('Signature trusted and verified'), findsOneWidget);
    expect(find.text('Inspector Fixture'), findsOneWidget);
    expect(find.text('1 scenes • 1 logic blocks • 1 actors'), findsOneWidget);

    await tester.tap(find.byKey(const Key('preview-portraits/hero.svg')));
    await tester.pumpAndSettle();
    expect(bindings.readCalls, 1);
    expect(
      find.bySemanticsLabel('Preview of portraits/hero.svg'),
      findsOneWidget,
    );

    await tester.tap(find.byKey(const Key('release-bundle')));
    await tester.pumpAndSettle();
    expect(find.text('Bundle disposed'), findsOneWidget);
    expect(bindings.disposeCalls, 1);
  });

  testWidgets('uses compact and wide responsive layouts', (tester) async {
    final controller = _controller(InspectorFakeBindings());
    addTearDown(controller.dispose);
    await controller.loadFixture();

    await tester.binding.setSurfaceSize(const Size(500, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(BundleInspectorApp(controller: controller));
    expect(find.byKey(const Key('compact-layout')), findsOneWidget);

    await tester.binding.setSurfaceSize(const Size(1200, 800));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('wide-layout')), findsOneWidget);
  });

  testWidgets('failure clears trusted content and exposes code plus text', (
    tester,
  ) async {
    final bindings = InspectorFakeBindings();
    final controller = _controller(bindings);
    addTearDown(controller.dispose);
    await controller.loadFixture();
    await tester.pumpWidget(BundleInspectorApp(controller: controller));
    expect(find.text('Inspector Fixture'), findsOneWidget);

    bindings.openError = const StoryBundleVerificationException(
      'B_BAD_SIGNATURE',
      'Ed25519 verification failed',
    );
    await tester.tap(find.byKey(const Key('load-bundle')));
    await tester.pumpAndSettle();

    expect(find.text('Inspector Fixture'), findsNothing);
    expect(find.text('B_BAD_SIGNATURE'), findsOneWidget);
    expect(find.text('Ed25519 verification failed'), findsOneWidget);
    expect(find.text('Load failed'), findsOneWidget);
  });

  testWidgets(
    'toolbar supports keyboard focus and non-color status semantics',
    (tester) async {
      final controller = _controller(InspectorFakeBindings());
      addTearDown(controller.dispose);
      final semantics = tester.ensureSemantics();
      await tester.pumpWidget(BundleInspectorApp(controller: controller));

      expect(find.bySemanticsLabel('No bundle loaded'), findsWidgets);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      expect(FocusManager.instance.primaryFocus, isNotNull);
      semantics.dispose();
    },
  );
}

BundleInspectorController _controller(InspectorFakeBindings bindings) =>
    BundleInspectorController(
      loader: StoryBundleLoader(
        trustStore: StoryBundleTrustStore.empty(),
        bindings: bindings,
      ),
      fixtureBytes: () async => Uint8List(1),
    );
