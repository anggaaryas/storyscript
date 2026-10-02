import 'package:flutter/material.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';
import '../../../l10n/app_localizations.dart';

class MetadataPanel extends StatelessWidget {
  const MetadataPanel({required this.bundle, super.key});

  final LoadedStoryBundle bundle;

  @override
  Widget build(BuildContext context) {
    final manifest = bundle.manifest;
    final strings = AppLocalizations.of(context);
    final localization = bundle.story.localization;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              strings.verifiedMetadata,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const Divider(),
            _Value(label: strings.project, value: manifest.project.name),
            _Value(label: strings.projectId, value: manifest.project.id),
            _Value(
              label: strings.projectVersion,
              value: manifest.project.version,
            ),
            _Value(label: strings.format, value: '${manifest.formatVersion}'),
            _Value(label: strings.compiler, value: manifest.compilerVersion),
            _Value(label: strings.schema, value: manifest.schemaSha256),
            _Value(
              label: strings.defaultLocale,
              value: localization.defaultLocale.isEmpty
                  ? strings.notLocalized
                  : localization.defaultLocale,
            ),
            _Value(
              label: strings.supportedLocaleHeading,
              value: localization.supportedLocales.isEmpty
                  ? strings.notLocalized
                  : localization.supportedLocales.join(', '),
            ),
            _Value(
              label: strings.signer,
              value: bundle.verification.signerKeyId ?? strings.unsignedBundle,
            ),
          ],
        ),
      ),
    );
  }
}

class _Value extends StatelessWidget {
  const _Value({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 4),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        SelectableText(value),
      ],
    ),
  );
}
