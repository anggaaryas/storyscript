import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

enum GamePhase { idle, loading, playing, failure }

final class GameController extends ChangeNotifier {
  GameController({
    required StoryBundlePlayerLoader loader,
    required Future<Uint8List> Function() bundleBytes,
    StoryBundleLoader? bundleLoader,
  }) : _loader = loader,
       _bundleLoader = bundleLoader,
       _bundleBytes = bundleBytes;

  final StoryBundlePlayerLoader _loader;
  final StoryBundleLoader? _bundleLoader;
  LoadedStoryBundle? _bundle;
  Uint8List? _archive;
  StoryBundlePlayerLocalePreferences _requested =
      const StoryBundlePlayerLocalePreferences.defaults();
  String? _resolved;
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
  List<String> get requestedLocales => _requested.locales;
  String? get resolvedLocale => _resolved;
  void setInitialLocales(List<String> locales) {
    if (_phase == GamePhase.idle && !_busy) {
      _requested = StoryBundlePlayerLocalePreferences(locales);
    }
  }

  /// A live session is never mutated. Failed restore leaves player/art/progress intact.
  Future<bool> changeLocales(List<String> locales) async {
    if (_closed || _busy) return false;
    final requested = StoryBundlePlayerLocalePreferences(locales);
    final previous = _player;
    if (previous == null) {
      _requested = requested;
      notifyListeners();
      return true;
    }
    _busy = true;
    _error = null;
    notifyListeners();
    StoryBundlePlayer? candidate;
    try {
      final save = await previous.exportSave();
      if (_closed) return false;
      candidate = _bundle != null
          ? await _loader.restoreFromBundle(_bundle!, save, locales: requested)
          : await _loader.restoreBytes(_archive!, save, locales: requested);
      if (_closed) return false;
      final delta = candidate.current;
      _player = candidate;
      _delta = delta;
      _requested = requested;
      _resolved = candidate.resolvedLocale;
      candidate = null;
      // Restore does not replay media. Keep artwork from this exact checkpoint.
      unawaited(previous.dispose().catchError((Object _) {}));
      return true;
    } catch (cause) {
      if (!_closed) _error = _describe(cause);
      return false;
    } finally {
      // Cleanup failure must not hide restore failure or strand the busy state.
      if (candidate != null) {
        await candidate.dispose().catchError((Object _) {});
      }
      if (!_closed) {
        _busy = false;
        notifyListeners();
      }
    }
  }

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
      final bytes = _archive ?? await _bundleBytes();
      if (_closed) return;
      _archive = Uint8List.fromList(bytes);
      final bundleLoader = _bundleLoader;
      if (_bundle == null && bundleLoader != null) {
        final bundle = await bundleLoader.openBytes(bytes);
        if (_closed) {
          await bundle.dispose();
          return;
        }
        _bundle = bundle;
      }
      final player = _bundle == null
          ? await _loader.openBytes(bytes, locales: _requested)
          : await _loader.fromBundle(_bundle!, locales: _requested);
      if (_closed) {
        await player.dispose();
        return;
      }
      _player = player;
      _resolved = player.resolvedLocale;
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
      if (!_closed) _artWarning = _describe(cause);
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
    _bundleLoader?.cancelPendingLoads();
    final player = _player;
    _player = null;
    if (player != null) unawaited(player.dispose());
    if (_bundle != null) unawaited(_bundle!.dispose());
    super.dispose();
  }
}
