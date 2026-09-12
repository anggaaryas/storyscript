import 'dart:async';
import 'dart:typed_data';

import 'package:storyscript_bundle/storyscript_bundle.dart';

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
