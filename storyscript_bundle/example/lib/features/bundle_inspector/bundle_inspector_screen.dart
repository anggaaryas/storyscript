import 'package:flutter/material.dart';

import 'bundle_inspector_controller.dart';
import 'widgets/asset_panel.dart';
import 'widgets/metadata_panel.dart';
import 'widgets/model_panel.dart';
import '../../l10n/app_localizations.dart';
import '../../localization/locale_scope.dart';

class BundleInspectorScreen extends StatelessWidget {
  const BundleInspectorScreen({required this.controller, super.key});

  static const double wideBreakpoint = 768;
  final BundleInspectorController controller;

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(AppLocalizations.of(context).inspectorTitle),
      actions: const [LocaleSelector()],
    ),
    body: SafeArea(
      child: AnimatedBuilder(
        animation: controller,
        builder: (context, _) {
          final state = controller.state;
          return Column(
            children: [
              _Toolbar(controller: controller, state: state),
              Expanded(
                child: _Body(controller: controller, state: state),
              ),
            ],
          );
        },
      ),
    ),
  );
}

class _Toolbar extends StatelessWidget {
  const _Toolbar({required this.controller, required this.state});

  final BundleInspectorController controller;
  final BundleInspectorState state;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
    child: Wrap(
      spacing: 12,
      runSpacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        Semantics(
          button: true,
          label: AppLocalizations.of(context).loadSignedBundle,
          child: FilledButton.icon(
            key: const Key('load-bundle'),
            onPressed: state.phase == BundleInspectorPhase.loading
                ? null
                : controller.loadFixture,
            icon: const Icon(Icons.file_open),
            label: Text(
              state.phase == BundleInspectorPhase.verified
                  ? AppLocalizations.of(context).reload
                  : AppLocalizations.of(context).load,
            ),
            style: const ButtonStyle(
              minimumSize: WidgetStatePropertyAll(Size(48, 48)),
            ),
          ),
        ),
        OutlinedButton.icon(
          key: const Key('release-bundle'),
          onPressed: state.bundle == null ? null : controller.releaseBundle,
          icon: const Icon(Icons.delete_outline),
          label: Text(AppLocalizations.of(context).release),
          style: const ButtonStyle(
            minimumSize: WidgetStatePropertyAll(Size(48, 48)),
          ),
        ),
        _Status(state: state),
      ],
    ),
  );
}

class _Status extends StatelessWidget {
  const _Status({required this.state});

  final BundleInspectorState state;

  @override
  Widget build(BuildContext context) {
    final strings = AppLocalizations.of(context);
    final (icon, text) = switch (state.phase) {
      BundleInspectorPhase.empty => (Icons.info_outline, strings.noBundle),
      BundleInspectorPhase.loading => (Icons.hourglass_top, strings.loading),
      BundleInspectorPhase.verified => (
        state.bundle!.verification.isUnsignedDevelopment
            ? Icons.warning_amber
            : Icons.verified_user,
        state.bundle!.verification.isUnsignedDevelopment
            ? strings.unsignedBundle
            : strings.trustedSignature,
      ),
      BundleInspectorPhase.failure => (Icons.error_outline, strings.loadFailed),
      BundleInspectorPhase.disposed => (
        Icons.delete_outline,
        strings.bundleDisposed,
      ),
    };
    return Semantics(
      liveRegion: true,
      label: text,
      child: Chip(
        key: const Key('trust-status'),
        avatar: Icon(icon, size: 20),
        label: Text(text),
      ),
    );
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.controller, required this.state});

  final BundleInspectorController controller;
  final BundleInspectorState state;

  @override
  Widget build(BuildContext context) {
    final strings = AppLocalizations.of(context);
    if (state.phase == BundleInspectorPhase.loading) {
      return Center(
        child: Semantics(
          label: strings.verifyingBundle,
          child: const CircularProgressIndicator(),
        ),
      );
    }
    if (state.phase == BundleInspectorPhase.failure) {
      return Center(
        child: Card(
          margin: const EdgeInsets.all(24),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.error_outline, size: 40),
                const SizedBox(height: 12),
                Text(
                  state.errorCode ?? 'B_UNKNOWN',
                  key: const Key('error-code'),
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                SelectableText(state.errorMessage ?? strings.unknownError),
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: controller.loadFixture,
                  style: const ButtonStyle(
                    minimumSize: WidgetStatePropertyAll(Size(48, 48)),
                  ),
                  child: Text(strings.retry),
                ),
              ],
            ),
          ),
        ),
      );
    }
    final bundle = state.bundle;
    if (bundle == null) {
      return Center(
        child: Text(
          state.phase == BundleInspectorPhase.disposed
              ? strings.resourceReleased
              : strings.loadToInspect,
          textAlign: TextAlign.center,
        ),
      );
    }
    return LayoutBuilder(
      builder: (context, constraints) {
        final panels = <Widget>[
          MetadataPanel(bundle: bundle),
          ModelPanel(bundle: bundle),
          AssetPanel(controller: controller, state: state),
        ];
        if (constraints.maxWidth >= BundleInspectorScreen.wideBreakpoint) {
          return Row(
            key: const Key('wide-layout'),
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              for (final panel in panels)
                Expanded(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.all(8),
                    child: panel,
                  ),
                ),
            ],
          );
        }
        return ListView(
          key: const Key('compact-layout'),
          padding: const EdgeInsets.all(8),
          children: [for (final panel in panels) panel],
        );
      },
    );
  }
}
