import 'dart:typed_data';

import 'package:storyscript_bundle/storyscript_bundle.dart';

final class InspectorFakeBindings implements StoryBundleBindings {
  InspectorFakeBindings({this.openError});

  StoryBundleException? openError;
  int disposeCalls = 0;
  int readCalls = 0;

  @override
  Future<StoryBundleBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
  ) async {
    final error = openError;
    if (error != null) throw error;
    return inspectorPayload();
  }

  @override
  Future<StoryBundleBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
  ) => openBytes(Uint8List(0), request);

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String logicalPath,
    int maximumBytes,
  ) async {
    readCalls++;
    return Uint8List.fromList(
      '<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>'
          .codeUnits,
    );
  }

  @override
  Future<void> dispose(Object resource) async {
    disposeCalls++;
  }
}

StoryBundleBridgePayload inspectorPayload() {
  final story = CompiledStory(
    formatVersion: 1,
    project: ProjectMetadata(
      id: 'storybundle.inspector',
      name: 'Inspector Fixture',
      version: '1.0.0',
    ),
    initialization: Initialization(
      startScene: 'start',
      actors: <Actor>[Actor(id: 'HERO')],
    ),
    logicBlocks: <LogicBlock>[LogicBlock(name: 'prepare')],
    scenes: <Scene>[Scene(label: 'start', story: StoryBlock())],
  );
  return StoryBundleBridgePayload(
    resource: Object(),
    manifest: StoryBundleManifest(
      formatVersion: 1,
      compilerVersion: '0.1.0',
      schemaSha256: storyBundleSchemaSha256,
      project: const StoryBundleProjectMetadata(
        id: 'storybundle.inspector',
        name: 'Inspector Fixture',
        version: '1.0.0',
      ),
      signerKeyId: 'a' * 64,
      resourceLimits: const StoryBundleLimits(),
      entries: const <StoryBundleManifestEntry>[],
    ),
    verification: StoryBundleVerification(
      signerKeyId: 'a' * 64,
      isUnsignedDevelopment: false,
    ),
    assets: const <StoryBundleAsset>[
      StoryBundleAsset(
        logicalPath: 'portraits/hero.svg',
        size: 67,
        sha256: 'b',
      ),
    ],
    compiledStoryBytes: story.writeToBuffer(),
  );
}
