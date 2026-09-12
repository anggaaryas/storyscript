import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

enum BundleInspectorPhase { empty, loading, verified, failure, disposed }

@immutable
final class BundleInspectorState {
  const BundleInspectorState({
    this.phase = BundleInspectorPhase.empty,
    this.progress,
    this.bundle,
    this.errorCode,
    this.errorMessage,
    this.selectedAsset,
    this.previewBytes,
    this.previewError,
  });

  final BundleInspectorPhase phase;
  final StoryBundleLoadProgress? progress;
  final LoadedStoryBundle? bundle;
  final String? errorCode;
  final String? errorMessage;
  final String? selectedAsset;
  final Uint8List? previewBytes;
  final String? previewError;
}

final class BundleInspectorController extends ChangeNotifier {
  BundleInspectorController({
    required StoryBundleLoader loader,
    required Future<Uint8List> Function() fixtureBytes,
  }) : _loader = loader,
       _fixtureBytes = fixtureBytes;

  final StoryBundleLoader _loader;
  final Future<Uint8List> Function() _fixtureBytes;
  BundleInspectorState _state = const BundleInspectorState();
  int _action = 0;
  bool _closed = false;

  BundleInspectorState get state => _state;

  Future<void> loadFixture() async {
    final action = ++_action;
    final previous = _state.bundle;
    _state = const BundleInspectorState(phase: BundleInspectorPhase.loading);
    notifyListeners();
    if (previous != null) {
      await previous.dispose();
    }
    try {
      final bytes = await _fixtureBytes();
      final loaded = await _loader.openBytes(
        bytes,
        onProgress: (progress) {
          if (!_closed && action == _action) {
            _state = BundleInspectorState(
              phase: BundleInspectorPhase.loading,
              progress: progress,
            );
            notifyListeners();
          }
        },
      );
      if (_closed || action != _action) {
        await loaded.dispose();
        return;
      }
      _state = BundleInspectorState(
        phase: BundleInspectorPhase.verified,
        bundle: loaded,
      );
    } on StoryBundleException catch (error) {
      if (!_closed && action == _action) {
        _state = BundleInspectorState(
          phase: BundleInspectorPhase.failure,
          errorCode: error.code,
          errorMessage: error.message,
        );
      }
    } catch (error) {
      if (!_closed && action == _action) {
        _state = BundleInspectorState(
          phase: BundleInspectorPhase.failure,
          errorCode: 'B_EXAMPLE_FAILURE',
          errorMessage: error.toString(),
        );
      }
    }
    if (!_closed && action == _action) {
      notifyListeners();
    }
  }

  Future<void> previewAsset(String logicalPath) async {
    final bundle = _state.bundle;
    if (bundle == null || _state.phase != BundleInspectorPhase.verified) {
      return;
    }
    final action = _action;
    _state = BundleInspectorState(
      phase: BundleInspectorPhase.verified,
      bundle: bundle,
      selectedAsset: logicalPath,
    );
    notifyListeners();
    try {
      final bytes = await bundle.readAsset(logicalPath);
      _validatePreview(logicalPath, bytes);
      if (!_closed && action == _action) {
        _state = BundleInspectorState(
          phase: BundleInspectorPhase.verified,
          bundle: bundle,
          selectedAsset: logicalPath,
          previewBytes: bytes,
        );
        notifyListeners();
      }
    } catch (error) {
      if (!_closed && action == _action) {
        _state = BundleInspectorState(
          phase: BundleInspectorPhase.verified,
          bundle: bundle,
          selectedAsset: logicalPath,
          previewError: error.toString(),
        );
        notifyListeners();
      }
    }
  }

  Future<void> releaseBundle() async {
    final bundle = _state.bundle;
    _loader.cancelPendingLoads();
    _action++;
    if (bundle != null) {
      await bundle.dispose();
    }
    if (!_closed) {
      _state = const BundleInspectorState(phase: BundleInspectorPhase.disposed);
      notifyListeners();
    }
  }

  void clear() {
    unawaited(releaseBundle());
    if (!_closed) {
      _state = const BundleInspectorState();
      notifyListeners();
    }
  }

  void _validatePreview(String path, Uint8List bytes) {
    final lower = path.toLowerCase();
    if (lower.endsWith('.svg')) {
      final prefix = utf8.decode(bytes, allowMalformed: true).trimLeft();
      if (!prefix.startsWith('<svg')) {
        throw const FormatException('asset is not valid SVG data');
      }
      return;
    }
    if (lower.endsWith('.png') &&
        bytes.length >= 8 &&
        listEquals(bytes.sublist(0, 8), const <int>[
          0x89,
          0x50,
          0x4e,
          0x47,
          0x0d,
          0x0a,
          0x1a,
          0x0a,
        ])) {
      return;
    }
    throw const FormatException('asset is not a supported image preview');
  }

  @override
  void dispose() {
    _closed = true;
    _loader.cancelPendingLoads();
    unawaited(_state.bundle?.dispose());
    super.dispose();
  }
}
