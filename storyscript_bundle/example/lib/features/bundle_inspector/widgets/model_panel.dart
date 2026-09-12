import 'package:flutter/material.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

class ModelPanel extends StatelessWidget {
  const ModelPanel({required this.bundle, super.key});

  final LoadedStoryBundle bundle;

  @override
  Widget build(BuildContext context) {
    final story = bundle.story;
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.all(8),
              child: Text(
                'Semantic model',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            ListTile(
              title: const Text('Summary'),
              subtitle: Text(
                '${story.scenes.length} scenes • '
                '${story.logicBlocks.length} logic blocks • '
                '${story.initialization.actors.length} actors',
              ),
            ),
            ExpansionTile(
              title: Text('Scenes (${story.scenes.length})'),
              children: [
                for (final scene in story.scenes)
                  ListTile(
                    leading: const Icon(Icons.account_tree_outlined),
                    title: SelectableText(scene.label),
                    subtitle: Text(
                      '${scene.story.statements.length} story statements',
                    ),
                  ),
              ],
            ),
            ExpansionTile(
              title: Text('Actors (${story.initialization.actors.length})'),
              children: [
                for (final actor in story.initialization.actors)
                  ListTile(
                    leading: const Icon(Icons.person_outline),
                    title: SelectableText(actor.id),
                    subtitle: Text('${actor.portraits.length} portraits'),
                  ),
              ],
            ),
            ExpansionTile(
              title: Text('Logic (${story.logicBlocks.length})'),
              children: [
                for (final logic in story.logicBlocks)
                  ListTile(
                    leading: const Icon(Icons.functions),
                    title: SelectableText(logic.name),
                    subtitle: Text('${logic.body.length} statements'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
