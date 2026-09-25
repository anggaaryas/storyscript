import 'dart:typed_data';

enum StoryBundlePlayerStatus { active, finished, faulted }

enum StoryBundlePlayerEventKind {
  scene,
  narration,
  dialogue,
  choices,
  media,
  end,
  error,
}

final class StoryBundlePlayerLimits {
  const StoryBundlePlayerLimits({
    this.operationsPerInteraction = 100000,
    this.logicDepth = 128,
    this.pendingEventsPerScene = 16384,
    this.arrayElements = 16384,
    this.renderedBytes = 1024 * 1024,
    this.historyEntries = 10000,
    this.historyBytes = 8 * 1024 * 1024,
    this.saveBytes = 16 * 1024 * 1024,
  });
  final int operationsPerInteraction;
  final int logicDepth;
  final int pendingEventsPerScene;
  final int arrayElements;
  final int renderedBytes;
  final int historyEntries;
  final int historyBytes;
  final int saveBytes;
}

final class StoryBundlePlayerChoice {
  const StoryBundlePlayerChoice({
    required this.text,
    required this.targetScene,
  });
  final String text;
  final String targetScene;
}

final class StoryBundleRuntimeError {
  const StoryBundleRuntimeError({
    required this.code,
    required this.scene,
    required this.message,
    this.resource,
    this.actual,
    this.limit,
  });
  final String code;
  final String scene;
  final String message;
  final String? resource;
  final int? actual;
  final int? limit;
}

final class StoryBundlePlayerException implements Exception {
  const StoryBundlePlayerException(this.error);
  final StoryBundleRuntimeError error;
  @override
  String toString() => '${error.code}: ${error.message}';
}

final class StoryBundlePlayerEvent {
  StoryBundlePlayerEvent({
    required this.kind,
    this.text,
    this.scene,
    this.actorId,
    this.actorName,
    this.emotion,
    this.position,
    this.portraitPath,
    Iterable<StoryBundlePlayerChoice> choices = const [],
    this.error,
  }) : choices = List.unmodifiable(choices);
  final StoryBundlePlayerEventKind kind;
  final String? text;
  final String? scene;
  final String? actorId;
  final String? actorName;
  final String? emotion;
  final String? position;
  final String? portraitPath;
  final List<StoryBundlePlayerChoice> choices;
  final StoryBundleRuntimeError? error;
}

final class StoryBundlePlayerEffect {
  const StoryBundlePlayerEffect({required this.kind, this.path});
  final String kind;
  final String? path;
}

final class StoryBundlePlayerDelta {
  StoryBundlePlayerDelta({
    required this.event,
    required Iterable<StoryBundlePlayerEffect> effects,
    required this.scene,
    required this.status,
    required this.sequence,
    required this.firstRetainedSequence,
    required this.omittedHistoryCount,
  }) : effects = List.unmodifiable(effects);
  final StoryBundlePlayerEvent event;
  final List<StoryBundlePlayerEffect> effects;
  final String scene;
  final StoryBundlePlayerStatus status;
  final int sequence;
  final int firstRetainedSequence;
  final int omittedHistoryCount;
}

final class StoryBundlePlayerHistoryEntry {
  StoryBundlePlayerHistoryEntry({
    required this.sequence,
    required this.event,
    required Iterable<StoryBundlePlayerEffect> effects,
    required this.scene,
  }) : effects = List.unmodifiable(effects);
  final int sequence;
  final StoryBundlePlayerEvent event;
  final List<StoryBundlePlayerEffect> effects;
  final String scene;
}

final class StoryBundlePlayerHistoryPage {
  StoryBundlePlayerHistoryPage({
    required Iterable<StoryBundlePlayerHistoryEntry> entries,
    required this.nextSequence,
    required this.firstRetainedSequence,
    required this.omittedHistoryCount,
  }) : entries = List.unmodifiable(entries);
  final List<StoryBundlePlayerHistoryEntry> entries;
  final int nextSequence;
  final int firstRetainedSequence;
  final int omittedHistoryCount;
}

Uint8List copyBundlePlayerBytes(List<int> bytes) => Uint8List.fromList(bytes);
