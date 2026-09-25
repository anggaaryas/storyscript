import 'dart:typed_data';

import '../rust/api/player_v2.dart' as rust;
import 'models.dart';

final class SourcePlayerBridgePayload {
  const SourcePlayerBridgePayload({
    required this.resource,
    required this.current,
  });
  final Object resource;
  final StoryPlayerDelta current;
}

abstract interface class SourcePlayerBindings {
  Future<SourcePlayerBridgePayload> openSource(
    String source,
    StoryPlayerLimits limits,
  );
  Future<SourcePlayerBridgePayload> openPath(
    String path,
    StoryPlayerLimits limits,
  );
  Future<SourcePlayerBridgePayload> restoreSource(
    String source,
    Uint8List save,
    StoryPlayerLimits limits,
  );
  Future<SourcePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryPlayerLimits limits,
  );
  Future<StoryPlayerDelta> current(Object resource);
  Future<StoryPlayerDelta> advance(Object resource);
  Future<StoryPlayerDelta> choose(Object resource, int index);
  Future<StoryPlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  );
  Future<Uint8List> exportSave(Object resource);
  Future<void> dispose(Object resource);
}

final class FfiSourcePlayerBindings implements SourcePlayerBindings {
  const FfiSourcePlayerBindings();

  @override
  Future<SourcePlayerBridgePayload> openSource(
    String source,
    StoryPlayerLimits limits,
  ) => _open(rust.sourcePlayerOpenRaw(source: source, limits: _limits(limits)));

  @override
  Future<SourcePlayerBridgePayload> openPath(
    String path,
    StoryPlayerLimits limits,
  ) => _open(rust.sourcePlayerOpenPath(path: path, limits: _limits(limits)));

  @override
  Future<SourcePlayerBridgePayload> restoreSource(
    String source,
    Uint8List save,
    StoryPlayerLimits limits,
  ) => _open(
    rust.sourcePlayerRestoreRaw(
      source: source,
      save: save,
      limits: _limits(limits),
    ),
  );

  @override
  Future<SourcePlayerBridgePayload> restorePath(
    String path,
    Uint8List save,
    StoryPlayerLimits limits,
  ) => _open(
    rust.sourcePlayerRestorePath(
      path: path,
      save: save,
      limits: _limits(limits),
    ),
  );

  @override
  Future<StoryPlayerDelta> current(Object resource) async =>
      _action(await rust.sourcePlayerCurrent(resource: _resource(resource)));

  @override
  Future<StoryPlayerDelta> advance(Object resource) async =>
      _action(await rust.sourcePlayerAdvance(resource: _resource(resource)));

  @override
  Future<StoryPlayerDelta> choose(Object resource, int index) async => _action(
    await rust.sourcePlayerChoose(resource: _resource(resource), index: index),
  );

  @override
  Future<StoryPlayerHistoryPage> history(
    Object resource,
    int startSequence,
    int maximum,
  ) async {
    final result = await rust.sourcePlayerHistory(
      resource: _resource(resource),
      startSequence: BigInt.from(startSequence),
      maximum: maximum,
    );
    _throw(result.error);
    final page = result.page;
    if (page == null) throw _contract('history bridge returned no page');
    return StoryPlayerHistoryPage(
      entries: page.entries.map(
        (entry) => StoryPlayerHistoryEntry(
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
    final result = await rust.sourcePlayerExportSave(
      resource: _resource(resource),
    );
    _throw(result.error);
    final bytes = result.bytes;
    if (bytes == null) throw _contract('save bridge returned no bytes');
    return Uint8List.fromList(bytes);
  }

  @override
  Future<void> dispose(Object resource) async {
    final result = await rust.sourcePlayerDispose(
      resource: _resource(resource),
    );
    _throw(result.error);
  }

  Future<SourcePlayerBridgePayload> _open(
    Future<rust.BridgeSourcePlayerOpenResult> pending,
  ) async {
    final result = await pending;
    _throw(result.error);
    final opened = result.opened;
    if (opened == null) throw _contract('open bridge returned no player');
    try {
      return SourcePlayerBridgePayload(
        resource: opened.resource,
        current: _delta(opened.current),
      );
    } finally {
      opened.dispose();
    }
  }

  StoryPlayerDelta _action(rust.BridgePlayerActionResult result) {
    _throw(result.error);
    final delta = result.delta;
    if (delta == null) {
      throw _contract('action bridge returned no delta');
    }
    return _delta(delta);
  }

  static rust.SourcePlayerResource _resource(Object value) {
    if (value is! rust.SourcePlayerResource) {
      throw _contract('invalid Rust player resource');
    }
    return value;
  }

  static rust.BridgePlayerLimits _limits(StoryPlayerLimits value) =>
      rust.BridgePlayerLimits(
        operationsPerInteraction: BigInt.from(value.operationsPerInteraction),
        logicDepth: BigInt.from(value.logicDepth),
        pendingEventsPerScene: BigInt.from(value.pendingEventsPerScene),
        arrayElements: BigInt.from(value.arrayElements),
        renderedBytes: BigInt.from(value.renderedBytes),
        historyEntries: BigInt.from(value.historyEntries),
        historyBytes: BigInt.from(value.historyBytes),
        saveBytes: BigInt.from(value.saveBytes),
      );

  static StoryPlayerDelta _delta(rust.BridgePlayerDelta value) =>
      StoryPlayerDelta(
        event: _event(value.event),
        effects: value.effects.map(_effect),
        scene: value.scene,
        status: switch (value.status) {
          'active' => StoryPlayerStatus.active,
          'finished' => StoryPlayerStatus.finished,
          'faulted' => StoryPlayerStatus.faulted,
          _ => throw _contract('unknown player status'),
        },
        sequence: _integer(value.sequence),
        firstRetainedSequence: _integer(value.firstRetainedSequence),
        omittedHistoryCount: _integer(value.omittedHistoryCount),
      );

  static StoryPlayerEvent _event(rust.BridgeSemanticEvent value) =>
      StoryPlayerEvent(
        kind: switch (value.kind) {
          'scene' => StoryPlayerEventKind.scene,
          'narration' => StoryPlayerEventKind.narration,
          'dialogue' => StoryPlayerEventKind.dialogue,
          'choices' => StoryPlayerEventKind.choices,
          'media' => StoryPlayerEventKind.media,
          'end' => StoryPlayerEventKind.end,
          'error' => StoryPlayerEventKind.error,
          _ => throw _contract('unknown event kind'),
        },
        text: value.text,
        scene: value.scene,
        actorId: value.actorId,
        actorName: value.actorName,
        emotion: value.emotion,
        position: value.position,
        portraitPath: value.portraitPath,
        choices: value.choices.map(
          (choice) => StoryPlayerChoice(
            text: choice.text,
            targetScene: choice.targetScene,
          ),
        ),
        error: value.error == null ? null : _error(value.error!),
      );

  static StoryPlayerMediaEffect _effect(rust.BridgeMediaEffect value) =>
      StoryPlayerMediaEffect(kind: value.kind, path: value.path);

  static StoryPlayerError _error(rust.BridgePlayerError value) =>
      StoryPlayerError(
        code: value.code,
        scene: value.scene,
        message: value.message,
        resource: value.resource,
        actual: value.actual == null ? null : _integer(value.actual!),
        limit: value.limit == null ? null : _integer(value.limit!),
      );

  static void _throw(rust.BridgePlayerError? error) {
    if (error != null) {
      throw StoryPlayerException(_error(error));
    }
  }

  static int _integer(BigInt value) {
    final converted = value.toInt();
    if (BigInt.from(converted) != value) {
      throw _contract('bridge integer exceeds Dart range');
    }
    return converted;
  }

  static StoryPlayerException _contract(String message) => StoryPlayerException(
    StoryPlayerError(code: 'R_BRIDGE_CONTRACT', scene: '', message: message),
  );
}
