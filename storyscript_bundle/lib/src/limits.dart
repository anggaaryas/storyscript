import 'rust/api/bundle.dart' as rust;

/// Resource ceilings applied in Dart before FFI and authoritatively in Rust.
final class StoryBundleLimits {
  const StoryBundleLimits({
    this.maxArchiveBytes = hardMaxArchiveBytes,
    this.maxTotalUncompressedBytes = hardMaxTotalUncompressedBytes,
    this.maxEntryBytes = hardMaxEntryBytes,
    this.maxCompiledIrBytes = hardMaxCompiledIrBytes,
    this.maxManifestBytes = hardMaxManifestBytes,
    this.maxEntries = hardMaxEntries,
    this.maxPathBytes = hardMaxPathBytes,
    this.maxSemanticDepth = hardMaxSemanticDepth,
  }) : assert(maxArchiveBytes > 0 && maxArchiveBytes <= hardMaxArchiveBytes),
       assert(
         maxTotalUncompressedBytes > 0 &&
             maxTotalUncompressedBytes <= hardMaxTotalUncompressedBytes,
       ),
       assert(maxEntryBytes > 0 && maxEntryBytes <= hardMaxEntryBytes),
       assert(
         maxCompiledIrBytes > 0 && maxCompiledIrBytes <= hardMaxCompiledIrBytes,
       ),
       assert(maxManifestBytes > 0 && maxManifestBytes <= hardMaxManifestBytes),
       assert(maxEntries > 0 && maxEntries <= hardMaxEntries),
       assert(maxPathBytes > 0 && maxPathBytes <= hardMaxPathBytes),
       assert(maxSemanticDepth > 0 && maxSemanticDepth <= hardMaxSemanticDepth);

  static const int hardMaxArchiveBytes = 100 * 1024 * 1024;
  static const int hardMaxTotalUncompressedBytes = 100 * 1024 * 1024;
  static const int hardMaxEntryBytes = 64 * 1024 * 1024;
  static const int hardMaxCompiledIrBytes = 16 * 1024 * 1024;
  static const int hardMaxManifestBytes = 1024 * 1024;
  static const int hardMaxEntries = 4096;
  static const int hardMaxPathBytes = 1024;
  static const int hardMaxSemanticDepth = 128;

  final int maxArchiveBytes;
  final int maxTotalUncompressedBytes;
  final int maxEntryBytes;
  final int maxCompiledIrBytes;
  final int maxManifestBytes;
  final int maxEntries;
  final int maxPathBytes;
  final int maxSemanticDepth;

  void validate() {
    final values = <(String, int, int)>[
      ('maxArchiveBytes', maxArchiveBytes, hardMaxArchiveBytes),
      (
        'maxTotalUncompressedBytes',
        maxTotalUncompressedBytes,
        hardMaxTotalUncompressedBytes,
      ),
      ('maxEntryBytes', maxEntryBytes, hardMaxEntryBytes),
      ('maxCompiledIrBytes', maxCompiledIrBytes, hardMaxCompiledIrBytes),
      ('maxManifestBytes', maxManifestBytes, hardMaxManifestBytes),
      ('maxEntries', maxEntries, hardMaxEntries),
      ('maxPathBytes', maxPathBytes, hardMaxPathBytes),
      ('maxSemanticDepth', maxSemanticDepth, hardMaxSemanticDepth),
    ];
    for (final (name, value, hardMaximum) in values) {
      if (value <= 0 || value > hardMaximum) {
        throw ArgumentError.value(
          value,
          name,
          'must be between 1 and the hard maximum $hardMaximum',
        );
      }
    }
  }

  rust.BridgeLimits toBridge() {
    validate();
    return rust.BridgeLimits(
      maxArchiveBytes: BigInt.from(maxArchiveBytes),
      maxTotalUncompressedBytes: BigInt.from(maxTotalUncompressedBytes),
      maxEntryBytes: BigInt.from(maxEntryBytes),
      maxCompiledIrBytes: BigInt.from(maxCompiledIrBytes),
      maxManifestBytes: BigInt.from(maxManifestBytes),
      maxEntries: maxEntries,
      maxPathBytes: maxPathBytes,
      maxSemanticDepth: maxSemanticDepth,
    );
  }
}
