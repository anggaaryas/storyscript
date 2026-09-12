sealed class StoryBundleException implements Exception {
  const StoryBundleException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => 'StoryBundleException($code): $message';

  static StoryBundleException fromCode(String code, String message) {
    if (code == 'B_RESOURCE_LIMIT') {
      return StoryBundleLimitException(code, message);
    }
    if (code == 'B_UNKNOWN_SIGNER' || code == 'B_SIGNING_KEY_INVALID') {
      return StoryBundleTrustException(code, message);
    }
    if (code == 'B_BAD_SIGNATURE' ||
        code == 'B_DIGEST_MISMATCH' ||
        code == 'B_UNSIGNED_DISALLOWED') {
      return StoryBundleVerificationException(code, message);
    }
    if (code == 'B_COMPILER_MISMATCH' ||
        code == 'B_SCHEMA_MISMATCH' ||
        code == 'B_UNSUPPORTED_FORMAT') {
      return StoryBundleCompatibilityException(code, message);
    }
    if (code == 'B_MISSING_ASSET') {
      return StoryBundleAssetException(code, message);
    }
    if (code == 'B_RESOURCE_DISPOSED') {
      return StoryBundleDisposedException(code, message);
    }
    if (code == 'B_PROTOBUF_DECODE' || code == 'B_SEMANTIC_VIOLATION') {
      return StoryBundleModelException(code, message);
    }
    return StoryBundleArchiveException(code, message);
  }
}

final class StoryBundleLimitException extends StoryBundleException {
  const StoryBundleLimitException(super.code, super.message);
}

final class StoryBundleTrustException extends StoryBundleException {
  const StoryBundleTrustException(super.code, super.message);
}

final class StoryBundleVerificationException extends StoryBundleException {
  const StoryBundleVerificationException(super.code, super.message);
}

final class StoryBundleCompatibilityException extends StoryBundleException {
  const StoryBundleCompatibilityException(super.code, super.message);
}

final class StoryBundleAssetException extends StoryBundleException {
  const StoryBundleAssetException(super.code, super.message);
}

final class StoryBundleDisposedException extends StoryBundleException {
  const StoryBundleDisposedException(super.code, super.message);
}

final class StoryBundleModelException extends StoryBundleException {
  const StoryBundleModelException(super.code, super.message);
}

final class StoryBundleArchiveException extends StoryBundleException {
  const StoryBundleArchiveException(super.code, super.message);
}

final class StoryBundleStaleLoadException extends StoryBundleException {
  const StoryBundleStaleLoadException()
    : super('B_STALE_LOAD', 'a newer load superseded this result');
}
