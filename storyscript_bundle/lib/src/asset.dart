/// Signed metadata for one normalized archive-backed asset.
final class StoryBundleAsset {
  const StoryBundleAsset({
    required this.logicalPath,
    required this.size,
    required this.sha256,
  });

  final String logicalPath;
  final int size;
  final String sha256;
}
