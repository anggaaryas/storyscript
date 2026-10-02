import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../bundle_inspector_controller.dart';
import '../../../l10n/app_localizations.dart';

class AssetPanel extends StatelessWidget {
  const AssetPanel({required this.controller, required this.state, super.key});

  final BundleInspectorController controller;
  final BundleInspectorState state;

  @override
  Widget build(BuildContext context) {
    final bundle = state.bundle!;
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
                strings.assetsHeading(bundle.assets.length),
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            for (final asset in bundle.assets)
              ListTile(
                minVerticalPadding: 12,
                title: SelectableText(asset.logicalPath),
                subtitle: Text(strings.assetBytes(asset.size, asset.sha256)),
                trailing: _isImage(asset.logicalPath)
                    ? Tooltip(
                        message: strings.previewAsset(asset.logicalPath),
                        child: IconButton(
                          key: Key('preview-${asset.logicalPath}'),
                          constraints: const BoxConstraints.tightFor(
                            width: 48,
                            height: 48,
                          ),
                          onPressed: () =>
                              controller.previewAsset(asset.logicalPath),
                          icon: const Icon(Icons.preview),
                        ),
                      )
                    : const SizedBox(width: 48, height: 48),
              ),
            if (state.selectedAsset != null) ...[
              const Divider(),
              Padding(
                padding: const EdgeInsets.all(12),
                child: _Preview(state: state),
              ),
            ],
          ],
        ),
      ),
    );
  }

  bool _isImage(String path) {
    final lower = path.toLowerCase();
    return lower.endsWith('.svg') || lower.endsWith('.png');
  }
}

class _Preview extends StatelessWidget {
  const _Preview({required this.state});

  final BundleInspectorState state;

  @override
  Widget build(BuildContext context) {
    final strings = AppLocalizations.of(context);
    if (state.previewError != null) {
      return Semantics(
        label: strings.previewFailed,
        child: Row(
          children: [
            const Icon(Icons.broken_image_outlined),
            const SizedBox(width: 8),
            Expanded(child: SelectableText(state.previewError!)),
          ],
        ),
      );
    }
    final bytes = state.previewBytes;
    if (bytes == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final path = state.selectedAsset!;
    return Semantics(
      image: true,
      label: strings.previewAsset(path),
      child: SizedBox(
        height: 180,
        child: path.toLowerCase().endsWith('.svg')
            ? SvgPicture.memory(bytes, fit: BoxFit.contain)
            : Image.memory(
                bytes,
                fit: BoxFit.contain,
                errorBuilder: (_, error, _) =>
                    Text(strings.assetDecodeFailed(error.toString())),
              ),
      ),
    );
  }
}
