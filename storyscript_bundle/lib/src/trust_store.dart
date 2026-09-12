import 'dart:typed_data';

/// One host-authenticated raw Ed25519 public key.
final class StoryBundleTrustKey {
  StoryBundleTrustKey(Uint8List publicKey, {this.expectedKeyId})
    : publicKey = Uint8List.fromList(publicKey) {
    if (publicKey.length != 32) {
      throw ArgumentError.value(
        publicKey.length,
        'publicKey.length',
        'Ed25519 public keys must contain exactly 32 bytes',
      );
    }
    if (expectedKeyId != null &&
        !RegExp(r'^[0-9a-f]{64}$').hasMatch(expectedKeyId!)) {
      throw ArgumentError.value(
        expectedKeyId,
        'expectedKeyId',
        'must be a lowercase SHA-256 hex digest',
      );
    }
  }

  final Uint8List publicKey;
  final String? expectedKeyId;
}

/// Immutable trust store supplied by the embedding application.
final class StoryBundleTrustStore {
  StoryBundleTrustStore(Iterable<StoryBundleTrustKey> keys)
    : keys = List.unmodifiable(keys);

  StoryBundleTrustStore.empty() : keys = const [];

  final List<StoryBundleTrustKey> keys;
}
