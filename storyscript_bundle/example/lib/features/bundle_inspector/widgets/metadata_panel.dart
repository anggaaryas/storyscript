import 'package:flutter/material.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

class MetadataPanel extends StatelessWidget {
  const MetadataPanel({required this.bundle, super.key});

  final LoadedStoryBundle bundle;

  @override
  Widget build(BuildContext context) {
    final manifest = bundle.manifest;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Verified metadata',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const Divider(),
            _Value(label: 'Project', value: manifest.project.name),
            _Value(label: 'Project ID', value: manifest.project.id),
            _Value(label: 'Project version', value: manifest.project.version),
            _Value(label: 'Format', value: '${manifest.formatVersion}'),
            _Value(label: 'Compiler', value: manifest.compilerVersion),
            _Value(label: 'Schema', value: manifest.schemaSha256),
            _Value(
              label: 'Signer',
              value: bundle.verification.signerKeyId ?? 'Unsigned development',
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
