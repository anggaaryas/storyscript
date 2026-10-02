import 'package:flutter/material.dart';

import 'features/bundle_inspector/bundle_inspector_controller.dart';
import 'features/bundle_inspector/bundle_inspector_screen.dart';
import 'features/game/game_controller.dart';
import 'features/game/game_screen.dart';
import 'l10n/app_localizations.dart';
import 'localization/locale_scope.dart';

class BundleInspectorApp extends StatefulWidget {
  const BundleInspectorApp({
    required this.controller,
    this.gameController,
    super.key,
  });

  final BundleInspectorController controller;
  final GameController? gameController;

  @override
  State<BundleInspectorApp> createState() => _BundleInspectorAppState();
}

class _BundleInspectorAppState extends State<BundleInspectorApp> {
  late Locale _locale;
  bool _switching = false;
  String? _switchError;
  @override
  void initState() {
    super.initState();
    final preferences = WidgetsBinding.instance.platformDispatcher.locales;
    _locale = Locale(
      preferences
          .map((l) => l.languageCode)
          .firstWhere(
            (code) => code == 'en' || code == 'id',
            orElse: () => 'en',
          ),
    );
    widget.gameController?.setInitialLocales(
      preferences.map((l) => l.toLanguageTag()).toList(),
    );
  }

  Future<void> _change(Locale locale) async {
    if (_switching || widget.gameController?.busy == true) return;
    setState(() {
      _switching = true;
      _switchError = null;
    });
    final success =
        await widget.gameController?.changeLocales([locale.toLanguageTag()]) ??
        true;
    if (!mounted) return;
    setState(() {
      if (success) {
        _locale = locale;
      } else {
        _switchError = widget.gameController?.error;
      }
      _switching = false;
    });
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.gameController ?? widget.controller,
    builder: (context, _) => MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      locale: _locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
        useMaterial3: true,
      ),
      routes: <String, WidgetBuilder>{
        '/': (_) => widget.gameController == null
            ? BundleInspectorScreen(controller: widget.controller)
            : GameScreen(controller: widget.gameController!),
        if (widget.gameController != null)
          '/inspector': (_) =>
              BundleInspectorScreen(controller: widget.controller),
      },
      builder: (context, child) => AppLocaleScope(
        locale: _locale,
        busy: _switching || widget.gameController?.busy == true,
        error: _switchError,
        change: _change,
        child: Column(
          children: [
            if (_switching || _switchError != null)
              Material(
                child: SafeArea(
                  bottom: false,
                  child: Semantics(
                    liveRegion: true,
                    child: Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        _switching
                            ? AppLocalizations.of(context).switchingLanguage
                            : AppLocalizations.of(
                                context,
                              ).switchFailed(_switchError!),
                      ),
                    ),
                  ),
                ),
              ),
            Expanded(child: child!),
          ],
        ),
      ),
    ),
  );
}
