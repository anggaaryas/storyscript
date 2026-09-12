import 'package:flutter/material.dart';

import 'features/bundle_inspector/bundle_inspector_controller.dart';
import 'features/bundle_inspector/bundle_inspector_screen.dart';

class BundleInspectorApp extends StatelessWidget {
  const BundleInspectorApp({required this.controller, super.key});

  final BundleInspectorController controller;

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'StoryBundle Inspector',
    debugShowCheckedModeBanner: false,
    theme: ThemeData(
      colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
      useMaterial3: true,
    ),
    routes: <String, WidgetBuilder>{
      '/': (_) => BundleInspectorScreen(controller: controller),
    },
  );
}
