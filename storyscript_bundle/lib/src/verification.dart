/// Verification result bound to a loaded semantic model.
final class StoryBundleVerification {
  const StoryBundleVerification({
    required this.signerKeyId,
    required this.isUnsignedDevelopment,
  });

  final String? signerKeyId;
  final bool isUnsignedDevelopment;

  bool get isStrictlyVerified => signerKeyId != null && !isUnsignedDevelopment;
}

enum StoryBundleLoadProgress { preflight, verifying, decoding, complete }
