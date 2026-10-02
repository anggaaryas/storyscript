import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

/// App-owned locale state; no story resources or Fluent parsing enter Flutter.
class AppLocaleScope extends InheritedWidget {
  const AppLocaleScope({
    required this.locale,
    required this.busy,
    required this.change,
    required super.child,
    this.error,
    super.key,
  });
  final Locale locale;
  final bool busy;
  final String? error;
  final Future<void> Function(Locale) change;
  static AppLocaleScope? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<AppLocaleScope>();
  @override
  bool updateShouldNotify(AppLocaleScope oldWidget) =>
      locale != oldWidget.locale ||
      busy != oldWidget.busy ||
      error != oldWidget.error;
}

class LocaleSelector extends StatelessWidget {
  const LocaleSelector({super.key});
  @override
  Widget build(BuildContext context) {
    final state = AppLocaleScope.maybeOf(context);
    if (state == null) return const SizedBox.shrink();
    final strings = AppLocalizations.of(context);
    return PopupMenuButton<String>(
      enabled: !state.busy,
      tooltip: strings.language,
      constraints: const BoxConstraints(minWidth: 160),
      onSelected: (language) => state.change(Locale(language)),
      itemBuilder: (_) => [
        PopupMenuItem(value: 'en', height: 48, child: Text(strings.english)),
        PopupMenuItem(value: 'id', height: 48, child: Text(strings.indonesian)),
      ],
      child: Semantics(
        button: true,
        label:
            '${strings.language}: ${state.locale.languageCode == 'id' ? strings.indonesian : strings.english}',
        child: const SizedBox(
          width: 48,
          height: 48,
          child: Icon(Icons.language),
        ),
      ),
    );
  }
}
