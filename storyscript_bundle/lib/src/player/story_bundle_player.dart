import 'dart:convert';
import 'dart:typed_data';

import '../bindings.dart';
import '../limits.dart';
import '../loaded_story_bundle.dart';
import '../trust_store.dart';
import 'bindings.dart';
import 'models.dart';

final class StoryBundlePlayerLoader {
  StoryBundlePlayerLoader({
    required StoryBundleTrustStore trustStore,
    StoryBundleLimits bundleLimits = const StoryBundleLimits(),
    StoryBundlePlayerLimits playerLimits = const StoryBundlePlayerLimits(),
    StoryBundlePlayerBindings bindings = const FfiStoryBundlePlayerBindings(),
  }) : _trustStore = trustStore,
       _bundleLimits = bundleLimits,
       _playerLimits = playerLimits,
       _bindings = bindings,
       _policy = StoryBundleVerificationPolicy.strict {
    bundleLimits.validate();
  }
  StoryBundlePlayerLoader.unsignedDevelopment({
    StoryBundleTrustStore? trustStore,
    StoryBundleLimits bundleLimits = const StoryBundleLimits(),
    StoryBundlePlayerLimits playerLimits = const StoryBundlePlayerLimits(),
    StoryBundlePlayerBindings bindings = const FfiStoryBundlePlayerBindings(),
  }) : _trustStore = trustStore ?? StoryBundleTrustStore.empty(),
       _bundleLimits = bundleLimits,
       _playerLimits = playerLimits,
       _bindings = bindings,
       _policy = StoryBundleVerificationPolicy.unsignedDevelopment {
    bundleLimits.validate();
  }

  final StoryBundleTrustStore _trustStore;
  final StoryBundleLimits _bundleLimits;
  final StoryBundlePlayerLimits _playerLimits;
  final StoryBundlePlayerBindings _bindings;
  final StoryBundleVerificationPolicy _policy;
  int _generation = 0;

  Future<StoryBundlePlayer> openBytes(Uint8List bytes) {
    if (bytes.lengthInBytes > _bundleLimits.maxArchiveBytes) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'B_RESOURCE_LIMIT',
          scene: '',
          message: 'bundle exceeds archive limit',
        ),
      );
    }
    return _finish(
      _bindings.openBytes(Uint8List.fromList(bytes), _request, _playerLimits),
      ++_generation,
    );
  }

  Future<StoryBundlePlayer> restoreBytes(Uint8List bytes, Uint8List save) =>
      _finish(
        _bindings.restoreBytes(
          Uint8List.fromList(bytes),
          Uint8List.fromList(save),
          _request,
          _playerLimits,
        ),
        ++_generation,
      );
  Future<StoryBundlePlayer> openPath(String path) =>
      _finish(_bindings.openPath(path, _request, _playerLimits), ++_generation);
  Future<StoryBundlePlayer> restorePath(String path, Uint8List save) => _finish(
    _bindings.restorePath(
      path,
      Uint8List.fromList(save),
      _request,
      _playerLimits,
    ),
    ++_generation,
  );
  Future<StoryBundlePlayer> fromBundle(LoadedStoryBundle bundle) => _finish(
    _bindings.openFromBundle(bundle.playerResourceHandoff(), _playerLimits),
    ++_generation,
  );
  Future<StoryBundlePlayer> restoreFromBundle(
    LoadedStoryBundle bundle,
    Uint8List save,
  ) => _finish(
    _bindings.restoreFromBundle(
      bundle.playerResourceHandoff(),
      Uint8List.fromList(save),
      _playerLimits,
    ),
    ++_generation,
  );
  void cancelPendingLoads() {
    _generation++;
  }

  StoryBundleBridgeRequest get _request => StoryBundleBridgeRequest(
    trustStore: _trustStore,
    policy: _policy,
    limits: _bundleLimits,
  );
  Future<StoryBundlePlayer> _finish(
    Future<StoryBundlePlayerBridgePayload> pending,
    int generation,
  ) async {
    final payload = await pending;
    if (generation != _generation) {
      await _bindings.dispose(payload.resource);
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_STALE_LOAD',
          scene: '',
          message: 'player load was superseded',
        ),
      );
    }
    return StoryBundlePlayer._(
      _bindings,
      payload.resource,
      payload.current,
      _bundleLimits.maxEntryBytes,
    );
  }
}

final class StoryBundlePlayer {
  StoryBundlePlayer._(
    this._bindings,
    this._resource,
    this._current,
    this._maximumAssetBytes,
  );
  final StoryBundlePlayerBindings _bindings;
  final Object _resource;
  final int _maximumAssetBytes;
  StoryBundlePlayerDelta _current;
  bool _disposed = false;
  Future<void>? _disposeFuture;
  StoryBundlePlayerDelta get current {
    _ensureOpen();
    return _current;
  }

  Future<StoryBundlePlayerDelta> advance() =>
      _mutate(() => _bindings.advance(_resource));
  Future<StoryBundlePlayerDelta> choose(int index) {
    if (index < 0) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_INVALID_CHOICE',
          scene: '',
          message: 'choice index must be non-negative',
        ),
      );
    }
    return _mutate(() => _bindings.choose(_resource, index));
  }

  Future<StoryBundlePlayerHistoryPage> history({
    required int startSequence,
    int maximum = 256,
  }) {
    _ensureOpen();
    if (startSequence < 0 || maximum <= 0 || maximum > 256) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_INVALID_ARGUMENT',
          scene: '',
          message: 'history bounds are invalid',
        ),
      );
    }
    return _bindings.history(_resource, startSequence, maximum);
  }

  Future<Uint8List> exportSave() async {
    _ensureOpen();
    return Uint8List.fromList(await _bindings.exportSave(_resource));
  }

  Future<Uint8List> readAsset(String path, {int? maximumBytes}) async {
    _ensureOpen();
    final parts = path.split('/');
    if (path.isEmpty ||
        path.startsWith('/') ||
        path.contains('\\') ||
        parts.any((part) => part.isEmpty || part == '.' || part == '..') ||
        utf8.encode(path).length > 1024) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'B_ASSET_INVALID',
          scene: '',
          message: 'asset path is not normalized',
        ),
      );
    }
    final limit = maximumBytes ?? _maximumAssetBytes;
    if (limit <= 0 || limit > _maximumAssetBytes) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'B_RESOURCE_LIMIT',
          scene: '',
          message: 'asset read limit is invalid',
        ),
      );
    }
    return Uint8List.fromList(
      await _bindings.readAsset(_resource, path, limit),
    );
  }

  Future<void> dispose() => _disposeFuture ??= _dispose();
  Future<void> _dispose() async {
    if (_disposed) {
      return;
    }
    _disposed = true;
    await _bindings.dispose(_resource);
  }

  Future<StoryBundlePlayerDelta> _mutate(
    Future<StoryBundlePlayerDelta> Function() operation,
  ) async {
    _ensureOpen();
    final result = await operation();
    if (_disposed) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_PLAYER_DISPOSED',
          scene: '',
          message: 'player was disposed while operation completed',
        ),
      );
    }
    return _current = result;
  }

  void _ensureOpen() {
    if (_disposed) {
      throw const StoryBundlePlayerException(
        StoryBundleRuntimeError(
          code: 'R_PLAYER_DISPOSED',
          scene: '',
          message: 'player has been disposed',
        ),
      );
    }
  }
}
