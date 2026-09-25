import 'dart:typed_data';

import 'bindings.dart';
import 'models.dart';

final class SourceStoryPlayerLoader {
  const SourceStoryPlayerLoader({
    this.bindings = const FfiSourcePlayerBindings(),
    this.limits = const StoryPlayerLimits(),
  });

  final SourcePlayerBindings bindings;
  final StoryPlayerLimits limits;

  Future<SourceStoryPlayer> openSource(String source) =>
      _open(bindings.openSource(source, limits));
  Future<SourceStoryPlayer> openPath(String path) =>
      _open(bindings.openPath(path, limits));
  Future<SourceStoryPlayer> restoreSource(String source, Uint8List save) =>
      _open(bindings.restoreSource(source, Uint8List.fromList(save), limits));
  Future<SourceStoryPlayer> restorePath(String path, Uint8List save) =>
      _open(bindings.restorePath(path, Uint8List.fromList(save), limits));

  Future<SourceStoryPlayer> _open(
    Future<SourcePlayerBridgePayload> pending,
  ) async {
    final payload = await pending;
    return SourceStoryPlayer._(bindings, payload.resource, payload.current);
  }
}

final class SourceStoryPlayer {
  SourceStoryPlayer._(this._bindings, this._resource, this._current);

  final SourcePlayerBindings _bindings;
  final Object _resource;
  StoryPlayerDelta _current;
  bool _disposed = false;
  Future<void>? _disposePending;

  StoryPlayerDelta get current {
    _ensureActive();
    return _current;
  }

  Future<StoryPlayerDelta> advance() =>
      _mutate(() => _bindings.advance(_resource));
  Future<StoryPlayerDelta> choose(int index) {
    if (index < 0) {
      throw const StoryPlayerException(
        StoryPlayerError(
          code: 'R_INVALID_CHOICE',
          scene: '',
          message: 'choice index must be non-negative',
        ),
      );
    }
    return _mutate(() => _bindings.choose(_resource, index));
  }

  Future<StoryPlayerHistoryPage> history({
    required int startSequence,
    int maximum = 256,
  }) {
    _ensureActive();
    if (startSequence < 0 || maximum <= 0 || maximum > 256) {
      throw const StoryPlayerException(
        StoryPlayerError(
          code: 'R_INVALID_ARGUMENT',
          scene: '',
          message: 'history bounds are invalid',
        ),
      );
    }
    return _bindings.history(_resource, startSequence, maximum);
  }

  Future<Uint8List> exportSave() async {
    _ensureActive();
    return Uint8List.fromList(await _bindings.exportSave(_resource));
  }

  Future<void> dispose() => _disposePending ??= _dispose();

  Future<void> _dispose() async {
    if (_disposed) return;
    _disposed = true;
    await _bindings.dispose(_resource);
  }

  Future<StoryPlayerDelta> _mutate(
    Future<StoryPlayerDelta> Function() operation,
  ) async {
    _ensureActive();
    final next = await operation();
    if (_disposed) {
      throw const StoryPlayerException(
        StoryPlayerError(
          code: 'R_PLAYER_DISPOSED',
          scene: '',
          message: 'player was disposed while operation completed',
        ),
      );
    }
    return _current = next;
  }

  void _ensureActive() {
    if (_disposed) {
      throw const StoryPlayerException(
        StoryPlayerError(
          code: 'R_PLAYER_DISPOSED',
          scene: '',
          message: 'player has been disposed',
        ),
      );
    }
  }
}
