import 'dart:async';
import 'dart:typed_data';

import 'package:storyscript_bundle/storyscript_bundle.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

final class FakeStoryBundleBindings implements StoryBundleBindings {
  FakeStoryBundleBindings({StoryBundleBridgePayload? payload})
    : payload = payload ?? makeBridgePayload();

  StoryBundleBridgePayload payload;
  StoryBundleBridgeRequest? lastRequest;
  int openBytesCalls = 0;
  int openPathCalls = 0;
  int readCalls = 0;
  int disposeCalls = 0;
  Completer<StoryBundleBridgePayload>? pendingOpen;
  Object? lastDisposedResource;

  @override
  Future<StoryBundleBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
  ) {
    openBytesCalls++;
    lastRequest = request;
    return pendingOpen?.future ?? Future.value(payload);
  }

  @override
  Future<StoryBundleBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
  ) {
    openPathCalls++;
    lastRequest = request;
    return Future.value(payload);
  }

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String logicalPath,
    int maximumBytes,
  ) async {
    readCalls++;
    return Uint8List.fromList(<int>[1, 2, 3, 4]);
  }

  @override
  Future<void> dispose(Object resource) async {
    disposeCalls++;
    lastDisposedResource = resource;
  }
}

StoryBundleBridgePayload makeBridgePayload({
  bool unsignedDevelopment = false,
  Uint8List? protobufBytes,
}) {
  final story = CompiledStory(
    formatVersion: 1,
    project: ProjectMetadata(
      id: 'storybundle.test',
      name: 'Test Bundle',
      version: '1.0.0',
    ),
    initialization: Initialization(startScene: 'start'),
    scenes: <Scene>[Scene(label: 'start', story: StoryBlock())],
  );
  return StoryBundleBridgePayload(
    resource: Object(),
    manifest: makeManifest(),
    verification: StoryBundleVerification(
      signerKeyId: unsignedDevelopment ? null : 'a' * 64,
      isUnsignedDevelopment: unsignedDevelopment,
    ),
    assets: const <StoryBundleAsset>[
      StoryBundleAsset(logicalPath: 'images/preview.png', size: 4, sha256: 'b'),
    ],
    compiledStoryBytes: protobufBytes ?? story.writeToBuffer(),
  );
}

StoryBundleManifest makeManifest() => StoryBundleManifest(
  formatVersion: 1,
  compilerVersion: '0.1.0',
  schemaSha256: storyBundleSchemaSha256,
  project: const StoryBundleProjectMetadata(
    id: 'storybundle.test',
    name: 'Test Bundle',
    version: '1.0.0',
  ),
  signerKeyId: 'a' * 64,
  resourceLimits: const StoryBundleLimits(),
  entries: const <StoryBundleManifestEntry>[],
);

final class FakeStoryBundlePlayerBindings
    implements LocaleAwareStoryBundlePlayerBindings {
  final resource = Object();
  StoryBundlePlayerDelta currentDelta = makePlayerDelta();
  int openBytesCalls = 0;
  int fromBundleCalls = 0;
  int disposeCalls = 0;
  int assetCalls = 0;
  Completer<StoryBundlePlayerBridgePayload>? pendingOpen;
  bool localized = false;
  List<String> requested = [];
  String? selectedLocale;
  Uint8List? receivedSave;
  Uint8List? receivedBytes;
  Object? receivedBundle;
  StoryBundlePlayerException? failure;

  void negotiate(StoryBundlePlayerLocalePreferences locales) {
    if (failure case final error?) throw error;
    requested = locales.locales;
    selectedLocale = localized
        ? (requested.firstOrNull?.startsWith('id') == true ? 'id' : 'en')
        : null;
  }

  StoryBundlePlayerBridgePayload get payload => StoryBundlePlayerBridgePayload(
    resource: resource,
    current: currentDelta,
    locale: StoryBundlePlayerLocaleResolution(resolvedLocale: selectedLocale),
  );

  @override
  Future<StoryBundlePlayerDelta> advance(Object resource) async {
    return currentDelta = makePlayerDelta(
      event: StoryBundlePlayerEvent(
        kind: StoryBundlePlayerEventKind.narration,
        text: selectedLocale == 'id'
            ? 'Halo'
            : localized
            ? 'Hello'
            : 'hello',
      ),
      sequence: 1,
    );
  }

  @override
  Future<StoryBundlePlayerDelta> choose(Object resource, int index) async =>
      currentDelta;
  @override
  Future<StoryBundlePlayerDelta> current(Object resource) async => currentDelta;
  @override
  Future<void> dispose(Object resource) async {
    disposeCalls++;
  }

  @override
  Future<Uint8List> exportSave(Object resource) async =>
      Uint8List.fromList([1, 2, 3]);
  @override
  Future<StoryBundlePlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  ) async => StoryBundlePlayerHistoryPage(
    entries: [
      StoryBundlePlayerHistoryEntry(
        sequence: 0,
        event: currentDelta.event,
        effects: const [],
        scene: 'start',
      ),
    ],
    nextSequence: 1,
    firstRetainedSequence: 0,
    omittedHistoryCount: 0,
  );
  @override
  Future<StoryBundlePlayerBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) {
    openBytesCalls++;
    receivedBytes = bytes;
    negotiate(locales);
    return pendingOpen?.future ?? Future.value(payload);
  }

  @override
  Future<StoryBundlePlayerBridgePayload> openFromBundle(
    Object bundleResource,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) async {
    fromBundleCalls++;
    receivedBundle = bundleResource;
    negotiate(locales);
    return payload;
  }

  @override
  Future<StoryBundlePlayerBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) async {
    negotiate(locales);
    return payload;
  }

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String path,
    int maximumBytes,
  ) async {
    assetCalls++;
    return Uint8List.fromList([4, 5, 6]);
  }

  @override
  Future<StoryBundlePlayerBridgePayload> restoreBytes(
    Uint8List bytes,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) async {
    receivedBytes = bytes;
    return restorePath('', save, request, limits, locales: locales);
  }

  @override
  Future<StoryBundlePlayerBridgePayload> restoreFromBundle(
    Object bundleResource,
    Uint8List save,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) async {
    receivedBundle = bundleResource;
    receivedSave = save;
    negotiate(locales);
    return payload;
  }

  @override
  Future<StoryBundlePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits, {
    StoryBundlePlayerLocalePreferences locales =
        const StoryBundlePlayerLocalePreferences.defaults(),
  }) async {
    receivedSave = save;
    negotiate(locales);
    if (localized) {
      currentDelta = makePlayerDelta(
        event: StoryBundlePlayerEvent(
          kind: StoryBundlePlayerEventKind.narration,
          text: selectedLocale == 'id' ? 'Halo' : 'Hello',
        ),
        sequence: currentDelta.sequence,
      );
    }
    return payload;
  }
}

StoryBundlePlayerDelta makePlayerDelta({
  StoryBundlePlayerEvent? event,
  int sequence = 0,
}) => StoryBundlePlayerDelta(
  event:
      event ??
      StoryBundlePlayerEvent(
        kind: StoryBundlePlayerEventKind.scene,
        scene: 'start',
      ),
  effects: const [],
  scene: 'start',
  status: StoryBundlePlayerStatus.active,
  sequence: sequence,
  firstRetainedSequence: 0,
  omittedHistoryCount: 0,
);
