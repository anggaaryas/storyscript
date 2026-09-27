import 'package:flutter/material.dart';

import 'features/bundle_inspector/bundle_inspector_controller.dart';
import 'features/bundle_inspector/bundle_inspector_screen.dart';
import 'features/game/game_controller.dart';
import 'features/game/game_screen.dart';

class BundleInspectorApp extends StatelessWidget {
  const BundleInspectorApp({
    required this.controller,
    this.gameController,
    super.key,
  });

  final BundleInspectorController controller;
  final GameController? gameController;

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'StoryScript Example',
    debugShowCheckedModeBanner: false,
    theme: ThemeData(
      colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
      useMaterial3: true,
    ),
    routes: <String, WidgetBuilder>{
      '/': (_) => gameController == null
          ? BundleInspectorScreen(controller: controller)
          : GameScreen(controller: gameController!),
      if (gameController != null)
        '/inspector': (_) => BundleInspectorScreen(controller: controller),
    },
  );
}
