import 'dart:convert';
import 'dart:typed_data';

import 'asset.dart';
import 'bindings.dart';
import 'exceptions.dart';
import 'manifest.dart';
import 'proto/storybundle/v1/compiled_story.pb.dart';
import 'verification.dart';

/// A verified semantic model tied to its Rust-owned archive resource.
final class LoadedStoryBundle {
  LoadedStoryBundle({
    required Object resource,
    required StoryBundleBindings bindings,
    required this.manifest,
    required this.verification,
    required this.story,
    required Iterable<StoryBundleAsset> assets,
  }) : _resource = resource,
       _bindings = bindings,
       assets = List.unmodifiable(assets);

  final Object _resource;
  final StoryBundleBindings _bindings;
  final StoryBundleManifest manifest;
  final StoryBundleVerification verification;
  final CompiledStory story;
  final List<StoryBundleAsset> assets;

  Future<void>? _disposeFuture;

  bool get isDisposed => _disposeFuture != null;

  Future<Uint8List> readAsset(String logicalPath, {int? maximumBytes}) async {
    _ensureOpen();
    _validateLogicalPath(logicalPath);
    final descriptor = assets.cast<StoryBundleAsset?>().firstWhere(
      (asset) => asset?.logicalPath == logicalPath,
      orElse: () => null,
    );
    if (descriptor == null) {
      throw StoryBundleAssetException(
        'B_MISSING_ASSET',
        'required asset is missing: $logicalPath',
      );
    }
    final limit = maximumBytes ?? manifest.resourceLimits.maxEntryBytes;
    if (limit <= 0 || limit > manifest.resourceLimits.maxEntryBytes) {
      throw StoryBundleLimitException(
        'B_RESOURCE_LIMIT',
        'asset read limit must be between 1 and '
            '${manifest.resourceLimits.maxEntryBytes} bytes',
      );
    }
    if (descriptor.size > limit) {
      throw StoryBundleLimitException(
        'B_RESOURCE_LIMIT',
        "asset '$logicalPath' exceeds the requested read limit",
      );
    }
    return _bindings.readAsset(_resource, logicalPath, limit);
  }

  Future<void> dispose() => _disposeFuture ??= _dispose();

  Future<void> _dispose() async {
    await _bindings.dispose(_resource);
  }

  void _ensureOpen() {
    if (isDisposed) {
      throw const StoryBundleDisposedException(
        'B_RESOURCE_DISPOSED',
        'bundle resource has been disposed',
      );
    }
  }

  void _validateLogicalPath(String path) {
    final components = path.split('/');
    if (path.isEmpty ||
        path.startsWith('/') ||
        path.contains('\\') ||
        components.any((part) => part.isEmpty || part == '.' || part == '..') ||
        utf8.encode(path).length > manifest.resourceLimits.maxPathBytes) {
      throw StoryBundleAssetException(
        'B_ASSET_INVALID',
        'asset path is not a normalized bundle-relative path: $path',
      );
    }
  }
}
