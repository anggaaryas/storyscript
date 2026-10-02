import 'package:flutter/material.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import '../../../l10n/app_localizations.dart';

class ModelPanel extends StatelessWidget {
  const ModelPanel({required this.bundle, super.key});

  final LoadedStoryBundle bundle;

  @override
  Widget build(BuildContext context) {
    final story = bundle.story;
    final strings = AppLocalizations.of(context);
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.all(8),
              child: Text(
                strings.semanticModel,
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            ListTile(
              title: Text(strings.summary),
              subtitle: Text(
                strings.modelSummary(
                  story.scenes.length,
                  story.logicBlocks.length,
                  story.initialization.actors.length,
                ),
              ),
            ),
            ExpansionTile(
              title: Text(strings.scenesHeading(story.scenes.length)),
              children: [
                for (final scene in story.scenes)
                  ListTile(
                    leading: const Icon(Icons.account_tree_outlined),
                    title: SelectableText(scene.label),
                    subtitle: Text(
                      strings.storyStatements(scene.story.statements.length),
                    ),
                  ),
              ],
            ),
            ExpansionTile(
              title: Text(
                strings.actorsHeading(story.initialization.actors.length),
              ),
              children: [
                for (final actor in story.initialization.actors)
                  ListTile(
                    leading: const Icon(Icons.person_outline),
                    title: SelectableText(actor.id),
                    subtitle: Text(strings.portraits(actor.portraits.length)),
                  ),
              ],
            ),
            ExpansionTile(
              title: Text(strings.logicHeading(story.logicBlocks.length)),
              children: [
                for (final logic in story.logicBlocks)
                  ListTile(
                    leading: const Icon(Icons.functions),
                    title: SelectableText(logic.name),
                    subtitle: Text(strings.statements(logic.body.length)),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
