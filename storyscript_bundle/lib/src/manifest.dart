import 'limits.dart';

final class StoryBundleProjectMetadata {
  const StoryBundleProjectMetadata({
    required this.id,
    required this.name,
    required this.version,
  });

  final String id;
  final String name;
  final String version;
}

final class StoryBundleManifestEntry {
  const StoryBundleManifestEntry({
    required this.path,
    required this.entryType,
    required this.compression,
    required this.compressedSize,
    required this.uncompressedSize,
    required this.sha256,
  });

  final String path;
  final String entryType;
  final String compression;
  final int compressedSize;
  final int uncompressedSize;
  final String sha256;
}

/// Immutable view of the manifest that Rust authenticated and validated.
final class StoryBundleManifest {
  StoryBundleManifest({
    required this.formatVersion,
    required this.compilerVersion,
    required this.schemaSha256,
    required this.project,
    required this.signerKeyId,
    required this.resourceLimits,
    required Iterable<StoryBundleManifestEntry> entries,
  }) : entries = List.unmodifiable(entries);

  final int formatVersion;
  final String compilerVersion;
  final String schemaSha256;
  final StoryBundleProjectMetadata project;
  final String signerKeyId;
  final StoryBundleLimits resourceLimits;
  final List<StoryBundleManifestEntry> entries;
}
