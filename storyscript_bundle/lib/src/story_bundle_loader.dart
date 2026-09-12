import 'dart:typed_data';

import 'bindings.dart';
import 'exceptions.dart';
import 'limits.dart';
import 'loaded_story_bundle.dart';
import 'proto/storybundle/v1/compiled_story.pb.dart';
import 'trust_store.dart';
import 'verification.dart';

typedef StoryBundleProgressCallback =
    void Function(StoryBundleLoadProgress progress);

/// Strict-by-default asynchronous loader for signed StoryBundles.
final class StoryBundleLoader {
  StoryBundleLoader({
    required StoryBundleTrustStore trustStore,
    StoryBundleLimits limits = const StoryBundleLimits(),
    StoryBundleBindings bindings = const FfiStoryBundleBindings(),
  }) : _trustStore = trustStore,
       _limits = limits,
       _bindings = bindings,
       _policy = StoryBundleVerificationPolicy.strict {
    limits.validate();
  }

  /// Explicit local-development policy. It never mutates global loader state.
  StoryBundleLoader.unsignedDevelopment({
    StoryBundleTrustStore? trustStore,
    StoryBundleLimits limits = const StoryBundleLimits(),
    StoryBundleBindings bindings = const FfiStoryBundleBindings(),
  }) : _trustStore = trustStore ?? StoryBundleTrustStore.empty(),
       _limits = limits,
       _bindings = bindings,
       _policy = StoryBundleVerificationPolicy.unsignedDevelopment {
    limits.validate();
  }

  final StoryBundleTrustStore _trustStore;
  final StoryBundleLimits _limits;
  final StoryBundleBindings _bindings;
  final StoryBundleVerificationPolicy _policy;
  int _generation = 0;

  Future<LoadedStoryBundle> openBytes(
    Uint8List bytes, {
    StoryBundleProgressCallback? onProgress,
  }) async {
    final generation = ++_generation;
    onProgress?.call(StoryBundleLoadProgress.preflight);
    if (bytes.lengthInBytes > _limits.maxArchiveBytes) {
      throw StoryBundleLimitException(
        'B_RESOURCE_LIMIT',
        'bundle exceeds the requested archive-byte limit',
      );
    }
    onProgress?.call(StoryBundleLoadProgress.verifying);
    final payload = await _bindings.openBytes(bytes, _request);
    return _finish(payload, generation, onProgress);
  }

  /// Opens a native filesystem path. Web callers must use [openBytes].
  Future<LoadedStoryBundle> openPath(
    String path, {
    StoryBundleProgressCallback? onProgress,
  }) async {
    final generation = ++_generation;
    onProgress?.call(StoryBundleLoadProgress.verifying);
    final payload = await _bindings.openPath(path, _request);
    return _finish(payload, generation, onProgress);
  }

  /// Invalidates pending load results. Rust resources returned later are freed.
  void cancelPendingLoads() {
    _generation++;
  }

  StoryBundleBridgeRequest get _request => StoryBundleBridgeRequest(
    trustStore: _trustStore,
    policy: _policy,
    limits: _limits,
  );

  Future<LoadedStoryBundle> _finish(
    StoryBundleBridgePayload payload,
    int generation,
    StoryBundleProgressCallback? onProgress,
  ) async {
    if (generation != _generation) {
      await _bindings.dispose(payload.resource);
      throw const StoryBundleStaleLoadException();
    }
    onProgress?.call(StoryBundleLoadProgress.decoding);
    CompiledStory story;
    try {
      story = CompiledStory.fromBuffer(payload.compiledStoryBytes);
    } catch (error) {
      await _bindings.dispose(payload.resource);
      throw StoryBundleModelException(
        'B_PROTOBUF_DECODE',
        'verified Protobuf could not be decoded in Dart: $error',
      );
    }
    if (payload.compiledStoryBytes.lengthInBytes > _limits.maxCompiledIrBytes) {
      await _bindings.dispose(payload.resource);
      throw StoryBundleLimitException(
        'B_RESOURCE_LIMIT',
        'verified compiled IR exceeds the requested Dart limit',
      );
    }
    final loaded = LoadedStoryBundle(
      resource: payload.resource,
      bindings: _bindings,
      manifest: payload.manifest,
      verification: payload.verification,
      story: story,
      assets: payload.assets,
    );
    onProgress?.call(StoryBundleLoadProgress.complete);
    return loaded;
  }
}
