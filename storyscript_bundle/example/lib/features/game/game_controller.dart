import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

enum GamePhase { idle, loading, playing, failure }

final class GameController extends ChangeNotifier {
  GameController({
    required StoryBundlePlayerLoader loader,
    required Future<Uint8List> Function() bundleBytes,
  }) : _loader = loader,
       _bundleBytes = bundleBytes;

  final StoryBundlePlayerLoader _loader;
  final Future<Uint8List> Function() _bundleBytes;
  StoryBundlePlayer? _player;
  StoryBundlePlayerDelta? _delta;
  Uint8List? _background;
  Uint8List? _portrait;
  String? _artWarning;
  String? _error;
  GamePhase _phase = GamePhase.idle;
  bool _busy = false;
  bool _closed = false;

  GamePhase get phase => _phase;
  bool get busy => _busy;
  StoryBundlePlayerDelta? get delta => _delta;
  Uint8List? get background => _background;
  Uint8List? get portrait => _portrait;
  String? get artWarning => _artWarning;
  String? get error => _error;

  Future<void> start() async {
    if (_closed || _busy) return;
    _busy = true;
    _phase = GamePhase.loading;
    _delta = null;
    _background = null;
    _portrait = null;
    _error = null;
    _artWarning = null;
    notifyListeners();
    final previous = _player;
    _player = null;
    try {
      if (previous != null) await previous.dispose();
      if (_closed) return;
      final bytes = await _bundleBytes();
      if (_closed) return;
      final player = await _loader.openBytes(bytes);
      if (_closed) {
        await player.dispose();
        return;
      }
      _player = player;
      await _show(player.current);
      if (!_closed) _phase = GamePhase.playing;
    } catch (cause) {
      if (!_closed) {
        _phase = GamePhase.failure;
        _error = _describe(cause);
      }
    } finally {
      if (!_closed) {
        _busy = false;
        notifyListeners();
      }
    }
  }

  Future<void> advance() => _act((player) => player.advance());

  Future<void> choose(int index) => _act((player) => player.choose(index));

  Future<void> _act(
    Future<StoryBundlePlayerDelta> Function(StoryBundlePlayer) action,
  ) async {
    final player = _player;
    if (_closed ||
        _busy ||
        player == null ||
        _phase != GamePhase.playing ||
        _delta?.status != StoryBundlePlayerStatus.active) {
      return;
    }
    _busy = true;
    notifyListeners();
    try {
      await _show(await action(player));
    } catch (cause) {
      if (!_closed) _error = _describe(cause);
    } finally {
      if (!_closed) {
        _busy = false;
        notifyListeners();
      }
    }
  }

  Future<void> _show(StoryBundlePlayerDelta delta) async {
    _delta = delta;
    _error = null;
    _artWarning = null;
    _portrait = null;
    for (final effect in delta.effects) {
      if (effect.kind == 'background' && effect.path != null) {
        _background = await _readArtwork(effect.path!);
      }
    }
    final portraitPath = delta.event.portraitPath;
    if (portraitPath != null) _portrait = await _readArtwork(portraitPath);
  }

  Future<Uint8List?> _readArtwork(String path) async {
    try {
      return await _player!.readAsset(path);
    } catch (cause) {
      if (!_closed) _artWarning = 'Artwork unavailable: ${_describe(cause)}';
      return null;
    }
  }

  String _describe(Object cause) => switch (cause) {
    StoryBundlePlayerException(:final error) =>
      '${error.code}: ${error.message}',
    _ => cause.toString(),
  };

  @override
  void dispose() {
    _closed = true;
    _loader.cancelPendingLoads();
    final player = _player;
    _player = null;
    if (player != null) unawaited(player.dispose());
    super.dispose();
  }
}
