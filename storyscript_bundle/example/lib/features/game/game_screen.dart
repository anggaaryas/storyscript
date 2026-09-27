import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:storyscript_bundle/storyscript_bundle_player.dart';

import 'game_controller.dart';

class GameScreen extends StatefulWidget {
  const GameScreen({required this.controller, super.key});

  final GameController controller;

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  @override
  void initState() {
    super.initState();
    widget.controller.start();
  }

  @override
  void dispose() {
    widget.controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    backgroundColor: const Color(0xFFF2EEE6),
    appBar: AppBar(
      title: const Text('Signal at Station Nine'),
      backgroundColor: const Color(0xFF1C2942),
      foregroundColor: Colors.white,
      actions: [
        IconButton(
          tooltip: 'Open demo Bundle Inspector',
          onPressed: () => Navigator.pushNamed(context, '/inspector'),
          icon: const Icon(Icons.inventory_2_outlined),
        ),
      ],
    ),
    body: SafeArea(
      child: AnimatedBuilder(
        animation: widget.controller,
        builder: (context, _) {
          final game = widget.controller;
          if (game.phase == GamePhase.loading || game.phase == GamePhase.idle) {
            return const Center(child: CircularProgressIndicator());
          }
          if (game.phase == GamePhase.failure) {
            return Center(
              child: _GameCard(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Text('Could not verify or start the game.'),
                    const SizedBox(height: 12),
                    Text(
                      game.error ?? 'Unknown error',
                      key: const Key('game-error'),
                    ),
                    const SizedBox(height: 12),
                    FilledButton(
                      onPressed: game.start,
                      child: const Text('Retry'),
                    ),
                  ],
                ),
              ),
            );
          }
          final delta = game.delta!;
          final event = delta.event;
          return Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 720),
              child: ListView(
                key: ValueKey(delta.sequence),
                padding: const EdgeInsets.all(16),
                children: [
                  Text(
                    'CHAPTER ONE  /  A BORROWED DAWN',
                    style: Theme.of(context).textTheme.labelMedium?.copyWith(
                      color: const Color(0xFF596B83),
                      letterSpacing: 1.3,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(height: 14),
                  if (game.background != null)
                    ClipRRect(
                      borderRadius: BorderRadius.circular(12),
                      child: SizedBox(
                        height: 190,
                        child: SvgPicture.memory(
                          game.background!,
                          key: const Key('game-background'),
                          semanticsLabel: 'Station Nine background',
                          fit: BoxFit.cover,
                        ),
                      ),
                    ),
                  const SizedBox(height: 16),
                  _GameCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          delta.scene.replaceAll('_', ' ').toUpperCase(),
                          style: Theme.of(context).textTheme.labelMedium
                              ?.copyWith(
                                color: const Color(0xFF66768A),
                                letterSpacing: 1.2,
                                fontWeight: FontWeight.w600,
                              ),
                        ),
                        const Divider(height: 32),
                        if (game.portrait != null)
                          SizedBox(
                            height: 100,
                            child: SvgPicture.memory(
                              game.portrait!,
                              key: const Key('game-portrait'),
                              semanticsLabel:
                                  '${event.actorName ?? event.actorId} portrait',
                            ),
                          ),
                        if (event.kind == StoryBundlePlayerEventKind.dialogue)
                          Text(
                            event.actorName ?? event.actorId ?? 'Speaker',
                            style: Theme.of(context).textTheme.titleMedium
                                ?.copyWith(
                                  color: const Color(0xFF245A79),
                                  fontWeight: FontWeight.bold,
                                ),
                          ),
                        Text(
                          _eventText(event),
                          key: const Key('game-line'),
                          style: Theme.of(context).textTheme.titleLarge
                              ?.copyWith(
                                fontFamily: 'serif',
                                fontSize: 21,
                                height: 1.55,
                                color: const Color(0xFF233249),
                              ),
                        ),
                        if (game.artWarning != null) ...[
                          const SizedBox(height: 8),
                          Text(game.artWarning!, key: const Key('art-warning')),
                        ],
                        if (game.error != null) ...[
                          const SizedBox(height: 8),
                          Text(game.error!, key: const Key('game-error')),
                        ],
                        const SizedBox(height: 20),
                        if (event.kind == StoryBundlePlayerEventKind.choices &&
                            delta.status == StoryBundlePlayerStatus.active)
                          for (
                            var index = 0;
                            index < event.choices.length;
                            index++
                          )
                            Padding(
                              padding: const EdgeInsets.only(bottom: 8),
                              child: SizedBox(
                                width: double.infinity,
                                child: Semantics(
                                  button: true,
                                  label:
                                      'Option ${index + 1}: ${event.choices[index].text}',
                                  child: FilledButton(
                                    key: Key('game-choice-$index'),
                                    onPressed: game.busy
                                        ? null
                                        : () => game.choose(index),
                                    style: const ButtonStyle(
                                      minimumSize: WidgetStatePropertyAll(
                                        Size(48, 52),
                                      ),
                                    ),
                                    child: Text(event.choices[index].text),
                                  ),
                                ),
                              ),
                            )
                        else if (delta.status !=
                                StoryBundlePlayerStatus.active ||
                            event.kind == StoryBundlePlayerEventKind.end ||
                            event.kind == StoryBundlePlayerEventKind.error)
                          FilledButton.icon(
                            key: const Key('game-restart'),
                            onPressed: game.busy ? null : game.start,
                            icon: const Icon(Icons.replay),
                            label: const Text('Play again'),
                          )
                        else if (delta.status == StoryBundlePlayerStatus.active)
                          FilledButton(
                            key: const Key('game-next'),
                            onPressed: game.busy ? null : game.advance,
                            child: const Text('Continue'),
                          ),
                        const SizedBox(height: 12),
                        Text(
                          event.kind == StoryBundlePlayerEventKind.choices
                              ? 'Choose how the story continues.'
                              : delta.status == StoryBundlePlayerStatus.active
                              ? 'Continue to turn the page.'
                              : 'You can begin the chapter again.',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    ),
  );
}

String _eventText(StoryBundlePlayerEvent event) => switch (event.kind) {
  StoryBundlePlayerEventKind.scene =>
    'Entering ${event.scene ?? 'the station'}…',
  StoryBundlePlayerEventKind.choices => 'What will you do?',
  StoryBundlePlayerEventKind.end => 'The End',
  StoryBundlePlayerEventKind.error =>
    '${event.error?.code ?? 'R_UNKNOWN'}: ${event.error?.message ?? 'Game stopped'}',
  StoryBundlePlayerEventKind.media => 'Media cue',
  _ => event.text ?? '',
};

class _GameCard extends StatelessWidget {
  const _GameCard({required this.child});
  final Widget child;

  @override
  Widget build(BuildContext context) => Card(
    color: const Color(0xFFFFFCF5),
    elevation: 2,
    child: Padding(padding: const EdgeInsets.all(20), child: child),
  );
}
