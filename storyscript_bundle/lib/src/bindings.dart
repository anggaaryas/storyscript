import 'dart:typed_data';

import 'asset.dart';
import 'exceptions.dart';
import 'limits.dart';
import 'manifest.dart';
import 'rust/api/bundle.dart' as rust;
import 'trust_store.dart';
import 'verification.dart';

enum StoryBundleVerificationPolicy { strict, unsignedDevelopment }

final class StoryBundleBridgeRequest {
  const StoryBundleBridgeRequest({
    required this.trustStore,
    required this.policy,
    required this.limits,
  });

  final StoryBundleTrustStore trustStore;
  final StoryBundleVerificationPolicy policy;
  final StoryBundleLimits limits;
}

/// Verified data returned by the bridge. [resource] remains Rust-owned.
final class StoryBundleBridgePayload {
  StoryBundleBridgePayload({
    required this.resource,
    required this.manifest,
    required this.verification,
    required Iterable<StoryBundleAsset> assets,
    required Uint8List compiledStoryBytes,
  }) : assets = List.unmodifiable(assets),
       compiledStoryBytes = Uint8List.fromList(compiledStoryBytes);

  final Object resource;
  final StoryBundleManifest manifest;
  final StoryBundleVerification verification;
  final List<StoryBundleAsset> assets;
  final Uint8List compiledStoryBytes;
}

/// Injectable bridge boundary used by the public loader and deterministic tests.
abstract interface class StoryBundleBindings {
  Future<StoryBundleBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
  );

  Future<StoryBundleBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
  );

  Future<Uint8List> readAsset(
    Object resource,
    String logicalPath,
    int maximumBytes,
  );

  Future<void> dispose(Object resource);
}

final class FfiStoryBundleBindings implements StoryBundleBindings {
  const FfiStoryBundleBindings();

  @override
  Future<StoryBundleBridgePayload> openBytes(
    Uint8List bytes,
    StoryBundleBridgeRequest request,
  ) async => _open(
    rust.bundleOpenBytes(
      bytes: bytes,
      trustKeys: _trustKeys(request.trustStore),
      policy: _policy(request.policy),
      limits: request.limits.toBridge(),
    ),
  );

  @override
  Future<StoryBundleBridgePayload> openPath(
    String path,
    StoryBundleBridgeRequest request,
  ) async => _open(
    rust.bundleOpenPath(
      path: path,
      trustKeys: _trustKeys(request.trustStore),
      policy: _policy(request.policy),
      limits: request.limits.toBridge(),
    ),
  );

  @override
  Future<Uint8List> readAsset(
    Object resource,
    String logicalPath,
    int maximumBytes,
  ) async {
    final result = await rust.bundleReadAsset(
      resource: _resource(resource),
      logicalPath: logicalPath,
      maximumBytes: BigInt.from(maximumBytes),
    );
    _throwBridgeError(result.error);
    final bytes = result.bytes;
    if (bytes == null) {
      throw const StoryBundleArchiveException(
        'B_BRIDGE_CONTRACT',
        'asset bridge returned neither bytes nor an error',
      );
    }
    return bytes;
  }

  @override
  Future<void> dispose(Object resource) async {
    final result = await rust.bundleDispose(resource: _resource(resource));
    _throwBridgeError(result.error);
  }

  Future<StoryBundleBridgePayload> _open(
    Future<rust.BridgeOpenResult> pending,
  ) async {
    final result = await pending;
    _throwBridgeError(result.error);
    final opened = result.opened;
    if (opened == null) {
      throw const StoryBundleArchiveException(
        'B_BRIDGE_CONTRACT',
        'open bridge returned neither a bundle nor an error',
      );
    }
    try {
      return StoryBundleBridgePayload(
        resource: opened.resource,
        manifest: _manifest(opened.manifest),
        verification: StoryBundleVerification(
          signerKeyId: opened.verification.signerKeyId,
          isUnsignedDevelopment: opened.verification.isUnsignedDevelopment,
        ),
        assets: opened.assets.map(
          (asset) => StoryBundleAsset(
            logicalPath: asset.logicalPath,
            size: _toInt(asset.size, 'asset size'),
            sha256: asset.sha256,
          ),
        ),
        compiledStoryBytes: opened.compiledStory,
      );
    } finally {
      opened.dispose();
    }
  }

  static rust.BundleResource _resource(Object resource) {
    if (resource is! rust.BundleResource) {
      throw const StoryBundleArchiveException(
        'B_BRIDGE_CONTRACT',
        'invalid Rust bundle resource',
      );
    }
    return resource;
  }

  static List<rust.BridgeTrustKey> _trustKeys(StoryBundleTrustStore store) =>
      store.keys
          .map(
            (key) => rust.BridgeTrustKey(
              publicKey: key.publicKey,
              expectedKeyId: key.expectedKeyId,
            ),
          )
          .toList(growable: false);

  static rust.BridgeVerificationPolicy _policy(
    StoryBundleVerificationPolicy policy,
  ) => switch (policy) {
    StoryBundleVerificationPolicy.strict =>
      rust.BridgeVerificationPolicy.strict,
    StoryBundleVerificationPolicy.unsignedDevelopment =>
      rust.BridgeVerificationPolicy.unsignedDevelopment,
  };

  static StoryBundleManifest _manifest(rust.BridgeManifest value) {
    final limits = value.resourceLimits;
    return StoryBundleManifest(
      formatVersion: value.formatVersion,
      compilerVersion: value.compilerVersion,
      schemaSha256: value.schemaSha256,
      project: StoryBundleProjectMetadata(
        id: value.project.id,
        name: value.project.name,
        version: value.project.version,
      ),
      signerKeyId: value.signerKeyId,
      resourceLimits: StoryBundleLimits(
        maxArchiveBytes: _toInt(limits.maxArchiveBytes, 'archive limit'),
        maxTotalUncompressedBytes: _toInt(
          limits.maxTotalUncompressedBytes,
          'uncompressed limit',
        ),
        maxEntryBytes: _toInt(limits.maxEntryBytes, 'entry limit'),
        maxCompiledIrBytes: _toInt(
          limits.maxCompiledIrBytes,
          'compiled IR limit',
        ),
        maxManifestBytes: _toInt(limits.maxManifestBytes, 'manifest limit'),
        maxEntries: limits.maxEntries,
        maxPathBytes: limits.maxPathBytes,
        maxSemanticDepth: limits.maxSemanticDepth,
      ),
      entries: value.entries.map(
        (entry) => StoryBundleManifestEntry(
          path: entry.path,
          entryType: entry.entryType,
          compression: entry.compression,
          compressedSize: _toInt(entry.compressedSize, 'compressed size'),
          uncompressedSize: _toInt(entry.uncompressedSize, 'uncompressed size'),
          sha256: entry.sha256,
        ),
      ),
    );
  }

  static int _toInt(BigInt value, String field) {
    if (value.isNegative || value > BigInt.parse('9223372036854775807')) {
      throw StoryBundleArchiveException(
        'B_BRIDGE_CONTRACT',
        '$field is outside the supported Dart integer range',
      );
    }
    return value.toInt();
  }

  static void _throwBridgeError(rust.BridgeError? error) {
    if (error != null) {
      throw StoryBundleException.fromCode(error.code, error.message);
    }
  }
}
