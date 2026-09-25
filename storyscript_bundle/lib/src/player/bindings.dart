import 'dart:typed_data';

import '../bindings.dart';
import '../rust/api/bundle.dart' as bundle_rust;
import '../rust/api/player.dart' as rust;
import '../trust_store.dart';
import 'models.dart';

final class StoryBundlePlayerBridgePayload {
  const StoryBundlePlayerBridgePayload({
    required this.resource,
    required this.current,
  });
  final Object resource;
  final StoryBundlePlayerDelta current;
}

abstract interface class StoryBundlePlayerBindings {
  Future<StoryBundlePlayerBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerBridgePayload> restoreBytes(
    Uint8List bytes,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerBridgePayload> openFromBundle(
    Object bundleResource,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerBridgePayload> restoreFromBundle(
    Object bundleResource,
    Uint8List save,
    StoryBundlePlayerLimits limits,
  );
  Future<StoryBundlePlayerDelta> current(Object resource);
  Future<StoryBundlePlayerDelta> advance(Object resource);
  Future<StoryBundlePlayerDelta> choose(Object resource, int index);
  Future<StoryBundlePlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  );
  Future<Uint8List> exportSave(Object resource);
  Future<Uint8List> readAsset(Object resource, String path, int maximumBytes);
  Future<void> dispose(Object resource);
}

final class FfiStoryBundlePlayerBindings implements StoryBundlePlayerBindings {
  const FfiStoryBundlePlayerBindings();

  @override
  Future<StoryBundlePlayerBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerOpenBytes(
      bytes: bytes,
      trustKeys: _trust(request.trustStore),
      policy: _policy(request.policy),
      bundleLimits: request.limits.toBridge(),
      playerLimits: _limits(limits),
    ),
  );
  @override
  Future<StoryBundlePlayerBridgePayload> restoreBytes(
    Uint8List bytes,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerRestoreBytes(
      bytes: bytes,
      save: save,
      trustKeys: _trust(request.trustStore),
      policy: _policy(request.policy),
      bundleLimits: request.limits.toBridge(),
      playerLimits: _limits(limits),
    ),
  );
  @override
  Future<StoryBundlePlayerBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerOpenPath(
      path: path,
      trustKeys: _trust(request.trustStore),
      policy: _policy(request.policy),
      bundleLimits: request.limits.toBridge(),
      playerLimits: _limits(limits),
    ),
  );
  @override
  Future<StoryBundlePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryBundleBridgeRequest request,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerRestorePath(
      path: path,
      save: save,
      trustKeys: _trust(request.trustStore),
      policy: _policy(request.policy),
      bundleLimits: request.limits.toBridge(),
      playerLimits: _limits(limits),
    ),
  );
  @override
  Future<StoryBundlePlayerBridgePayload> openFromBundle(
    Object resource,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerOpenFromBundle(
      bundle: _bundleResource(resource),
      limits: _limits(limits),
    ),
  );
  @override
  Future<StoryBundlePlayerBridgePayload> restoreFromBundle(
    Object resource,
    Uint8List save,
    StoryBundlePlayerLimits limits,
  ) => _open(
    rust.bundlePlayerRestoreFromBundle(
      bundle: _bundleResource(resource),
      save: save,
      limits: _limits(limits),
    ),
  );

  @override
  Future<StoryBundlePlayerDelta> current(Object resource) async =>
      _action(await rust.bundlePlayerCurrent(resource: _resource(resource)));
  @override
  Future<StoryBundlePlayerDelta> advance(Object resource) async =>
      _action(await rust.bundlePlayerAdvance(resource: _resource(resource)));
  @override
  Future<StoryBundlePlayerDelta> choose(Object resource, int index) async =>
      _action(
        await rust.bundlePlayerChoose(
          resource: _resource(resource),
          index: index,
        ),
      );
  @override
  Future<StoryBundlePlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  ) async {
    final result = await rust.bundlePlayerHistory(
      resource: _resource(resource),
      startSequence: BigInt.from(startSequence),
      maximum: maximum,
    );
    _throw(result.error);
    final page = result.page;
    if (page == null) throw _contract('history bridge returned no page');
    return StoryBundlePlayerHistoryPage(
      entries: page.entries.map(
        (entry) => StoryBundlePlayerHistoryEntry(
          sequence: _integer(entry.sequence),
          event: _event(entry.event),
          effects: entry.effects.map(_effect),
          scene: entry.scene,
        ),
      ),
      nextSequence: _integer(page.nextSequence),
      firstRetainedSequence: _integer(page.firstRetainedSequence),
      omittedHistoryCount: _integer(page.omittedHistoryCount),
    );
  }

  @override
  Future<Uint8List> exportSave(Object resource) async {
    final result = await rust.bundlePlayerExportSave(
      resource: _resource(resource),
    );
    _throw(result.error);
    if (result.bytes == null) throw _contract('save bridge returned no bytes');
    return Uint8List.fromList(result.bytes!);
  }

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String path,
    int maximumBytes,
  ) async {
    final result = await rust.bundlePlayerReadAsset(
      resource: _resource(resource),
      logicalPath: path,
      maximumBytes: BigInt.from(maximumBytes),
    );
    _throw(result.error);
    if (result.bytes == null) throw _contract('asset bridge returned no bytes');
    return Uint8List.fromList(result.bytes!);
  }

  @override
  Future<void> dispose(Object resource) async {
    final result = await rust.bundlePlayerDispose(
      resource: _resource(resource),
    );
    _throw(result.error);
  }

  Future<StoryBundlePlayerBridgePayload> _open(
    Future<rust.BridgeBundlePlayerOpenResult> pending,
  ) async {
    final result = await pending;
    _throw(result.error);
    final opened = result.opened;
    if (opened == null) {
      throw _contract('open bridge returned no player');
    }
    try {
      return StoryBundlePlayerBridgePayload(
        resource: opened.resource,
        current: _delta(opened.current),
      );
    } finally {
      opened.dispose();
    }
  }

  StoryBundlePlayerDelta _action(rust.BridgeBundlePlayerActionResult result) {
    _throw(result.error);
    if (result.delta == null) {
      throw _contract('action bridge returned no delta');
    }
    return _delta(result.delta!);
  }

  static rust.BundlePlayerResource _resource(Object value) {
    if (value is! rust.BundlePlayerResource) {
      throw _contract('invalid Rust player resource');
    }
    return value;
  }

  static bundle_rust.BundleResource _bundleResource(Object value) {
    if (value is! bundle_rust.BundleResource) {
      throw _contract('invalid verified bundle resource');
    }
    return value;
  }

  static List<bundle_rust.BridgeTrustKey> _trust(StoryBundleTrustStore store) =>
      store.keys
          .map(
            (key) => bundle_rust.BridgeTrustKey(
              publicKey: key.publicKey,
              expectedKeyId: key.expectedKeyId,
            ),
          )
          .toList(growable: false);
  static bundle_rust.BridgeVerificationPolicy _policy(
    StoryBundleVerificationPolicy value,
  ) => switch (value) {
    StoryBundleVerificationPolicy.strict =>
      bundle_rust.BridgeVerificationPolicy.strict,
    StoryBundleVerificationPolicy.unsignedDevelopment =>
      bundle_rust.BridgeVerificationPolicy.unsignedDevelopment,
  };
  static rust.BridgeRuntimeLimits _limits(StoryBundlePlayerLimits value) =>
      rust.BridgeRuntimeLimits(
        operationsPerInteraction: BigInt.from(value.operationsPerInteraction),
        logicDepth: BigInt.from(value.logicDepth),
        pendingEventsPerScene: BigInt.from(value.pendingEventsPerScene),
        arrayElements: BigInt.from(value.arrayElements),
        renderedBytes: BigInt.from(value.renderedBytes),
        historyEntries: BigInt.from(value.historyEntries),
        historyBytes: BigInt.from(value.historyBytes),
        saveBytes: BigInt.from(value.saveBytes),
      );
  static StoryBundlePlayerDelta _delta(rust.BridgePlayerDelta value) =>
      StoryBundlePlayerDelta(
        event: _event(value.event),
        effects: value.effects.map(_effect),
        scene: value.scene,
        status: switch (value.status) {
          'active' => StoryBundlePlayerStatus.active,
          'finished' => StoryBundlePlayerStatus.finished,
          'faulted' => StoryBundlePlayerStatus.faulted,
          _ => throw _contract('unknown status'),
        },
        sequence: _integer(value.sequence),
        firstRetainedSequence: _integer(value.firstRetainedSequence),
        omittedHistoryCount: _integer(value.omittedHistoryCount),
      );
  static StoryBundlePlayerEvent _event(rust.BridgePlayerEvent value) =>
      StoryBundlePlayerEvent(
        kind: switch (value.kind) {
          'scene' => StoryBundlePlayerEventKind.scene,
          'narration' => StoryBundlePlayerEventKind.narration,
          'dialogue' => StoryBundlePlayerEventKind.dialogue,
          'choices' => StoryBundlePlayerEventKind.choices,
          'media' => StoryBundlePlayerEventKind.media,
          'end' => StoryBundlePlayerEventKind.end,
          'error' => StoryBundlePlayerEventKind.error,
          _ => throw _contract('unknown event'),
        },
        text: value.text,
        scene: value.scene,
        actorId: value.actorId,
        actorName: value.actorName,
        emotion: value.emotion,
        position: value.position,
        portraitPath: value.portraitPath,
        choices: value.choices.map(
          (choice) => StoryBundlePlayerChoice(
            text: choice.text,
            targetScene: choice.targetScene,
          ),
        ),
        error: value.error == null ? null : _error(value.error!),
      );
  static StoryBundlePlayerEffect _effect(rust.BridgePlayerEffect value) =>
      StoryBundlePlayerEffect(kind: value.kind, path: value.path);
  static StoryBundleRuntimeError _error(rust.BridgeRuntimeError value) =>
      StoryBundleRuntimeError(
        code: value.code,
        scene: value.scene,
        message: value.message,
        resource: value.resource,
        actual: value.actual == null ? null : _integer(value.actual!),
        limit: value.limit == null ? null : _integer(value.limit!),
      );
  static void _throw(rust.BridgeRuntimeError? value) {
    if (value != null) {
      throw StoryBundlePlayerException(_error(value));
    }
  }

  static int _integer(BigInt value) {
    final result = value.toInt();
    if (BigInt.from(result) != value) {
      throw _contract('integer overflow');
    }
    return result;
  }

  static StoryBundlePlayerException _contract(String message) =>
      StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_BRIDGE_CONTRACT',
          scene: '',
          message: message,
        ),
      );
}
