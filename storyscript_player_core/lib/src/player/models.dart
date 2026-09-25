import 'dart:typed_data';

enum StoryPlayerStatus { active, finished, faulted }

enum StoryPlayerEventKind {
  scene,
  narration,
  dialogue,
  choices,
  media,
  end,
  error,
}

final class StoryPlayerLimits {
  const StoryPlayerLimits({
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

final class StoryPlayerChoice {
  const StoryPlayerChoice({required this.text, required this.targetScene});
  final String text;
  final String targetScene;
}

final class StoryPlayerError {
  const StoryPlayerError({
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

final class StoryPlayerException implements Exception {
  const StoryPlayerException(this.error);
  final StoryPlayerError error;

  @override
  String toString() => '${error.code}: ${error.message}';
}

final class StoryPlayerEvent {
  StoryPlayerEvent({
    required this.kind,
    this.text,
    this.scene,
    this.actorId,
    this.actorName,
    this.emotion,
    this.position,
    this.portraitPath,
    Iterable<StoryPlayerChoice> choices = const [],
    this.error,
  }) : choices = List.unmodifiable(choices);

  final StoryPlayerEventKind kind;
  final String? text;
  final String? scene;
  final String? actorId;
  final String? actorName;
  final String? emotion;
  final String? position;
  final String? portraitPath;
  final List<StoryPlayerChoice> choices;
  final StoryPlayerError? error;
}

final class StoryPlayerMediaEffect {
  const StoryPlayerMediaEffect({required this.kind, this.path});
  final String kind;
  final String? path;
}

final class StoryPlayerDelta {
  StoryPlayerDelta({
    required this.event,
    required Iterable<StoryPlayerMediaEffect> effects,
    required this.scene,
    required this.status,
    required this.sequence,
    required this.firstRetainedSequence,
    required this.omittedHistoryCount,
  }) : effects = List.unmodifiable(effects);

  final StoryPlayerEvent event;
  final List<StoryPlayerMediaEffect> effects;
  final String scene;
  final StoryPlayerStatus status;
  final int sequence;
  final int firstRetainedSequence;
  final int omittedHistoryCount;
}

final class StoryPlayerHistoryEntry {
  StoryPlayerHistoryEntry({
    required this.sequence,
    required this.event,
    required Iterable<StoryPlayerMediaEffect> effects,
    required this.scene,
  }) : effects = List.unmodifiable(effects);

  final int sequence;
  final StoryPlayerEvent event;
  final List<StoryPlayerMediaEffect> effects;
  final String scene;
}

final class StoryPlayerHistoryPage {
  StoryPlayerHistoryPage({
    required Iterable<StoryPlayerHistoryEntry> entries,
    required this.nextSequence,
    required this.firstRetainedSequence,
    required this.omittedHistoryCount,
  }) : entries = List.unmodifiable(entries);

  final List<StoryPlayerHistoryEntry> entries;
  final int nextSequence;
  final int firstRetainedSequence;
  final int omittedHistoryCount;
}

Uint8List copyPlayerSaveBytes(List<int> bytes) => Uint8List.fromList(bytes);
