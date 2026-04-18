import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:storyscript_player_core/storyscript_player_core.dart';

// ─── Color Palette (VS Code Dark inspired) ───
class _C {
  static const bg = Color(0xFF1e1e1e);
  static const surface = Color(0xFF252526);
  static const panel = Color(0xFF1e1e1e);
  static const border = Color(0xFF3c3c3c);
  static const tabBar = Color(0xFF2d2d2d);
  static const text = Color(0xFFd4d4d4);
  static const textDim = Color(0xFF858585);
  static const accent = Color(0xFF569cd6);
  static const green = Color(0xFF4ec9b0);
  static const yellow = Color(0xFFdcdcaa);
  static const orange = Color(0xFFce9178);
  static const keyword = Color(0xFFc586c0);
  static const errorRed = Color(0xFFf44747);
  static const selection = Color(0xFF264f78);
  static const buttonBg = Color(0xFF0e639c);
}

final _mono = GoogleFonts.jetBrainsMono(fontSize: 13, color: _C.text);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        scaffoldBackgroundColor: _C.bg,
        colorScheme: const ColorScheme.dark(
          primary: _C.accent,
          surface: _C.surface,
          error: _C.errorRed,
        ),
        textTheme: GoogleFonts.jetBrainsMonoTextTheme(
          ThemeData.dark().textTheme,
        ).apply(
          bodyColor: _C.text,
          displayColor: _C.text,
        ),
      ),
      home: const StoryScriptViewerPage(),
    );
  }
}

// ─── Main Page ───
class StoryScriptViewerPage extends StatefulWidget {
  const StoryScriptViewerPage({super.key});

  @override
  State<StoryScriptViewerPage> createState() => _StoryScriptViewerPageState();
}

class _StoryScriptViewerPageState extends State<StoryScriptViewerPage> {
  final TextEditingController _sourceController = TextEditingController();
  final ScrollController _consoleScroll = ScrollController();
  final FocusNode _keyboardFocusNode = FocusNode(debugLabel: 'keyboard_shortcuts');
  final FocusNode _editorFocusNode = FocusNode(debugLabel: 'story_editor');

  BigInt? _sessionId;
  BridgeState? _state;
  String? _error;
  bool _busy = false;

  @override
  void dispose() {
    final sessionId = _sessionId;
    if (sessionId != null) {
      playerClose(sessionId: sessionId);
    }
    _sourceController.dispose();
    _consoleScroll.dispose();
    _keyboardFocusNode.dispose();
    _editorFocusNode.dispose();
    super.dispose();
  }

  void _setBusy(bool value) {
    if (mounted) setState(() => _busy = value);
  }

  void _setError(String message) {
    if (mounted) setState(() => _error = message);
  }

  void _clearError() {
    if (mounted) setState(() => _error = null);
  }

  void _loadStory() {
    final source = _sourceController.text;
    if (source.trim().isEmpty) {
      _setError('// Error: paste StoryScript source first.');
      return;
    }
    _setBusy(true);
    _clearError();
    try {
      final oldSession = _sessionId;
      if (oldSession != null) playerClose(sessionId: oldSession);
      final newSession = playerOpenRaw(source: source);
      final nextState = playerGetState(sessionId: newSession);
      if (!mounted) return;
      setState(() {
        _sessionId = newSession;
        _state = nextState;
      });
    } catch (err) {
      _setError('// Error: $err');
    } finally {
      _setBusy(false);
    }
  }

  void _advance() {
    final sessionId = _sessionId;
    if (sessionId == null) {
      _setError('// Error: open a story first.');
      return;
    }
    if (_hasPendingChoice) {
      _setError('// Error: choose an option before advancing.');
      return;
    }
    _setBusy(true);
    _clearError();
    try {
      final nextState = playerAdvance(sessionId: sessionId);
      if (!mounted) return;
      setState(() => _state = nextState);
      _scrollToBottom();
    } catch (err) {
      _setError('// Error: $err');
    } finally {
      _setBusy(false);
    }
  }

  void _choose(int index) {
    final sessionId = _sessionId;
    if (sessionId == null) return;
    _setBusy(true);
    _clearError();
    try {
      final nextState = playerChoose(sessionId: sessionId, index: index);
      if (!mounted) return;
      setState(() => _state = nextState);
      _scrollToBottom();
    } catch (err) {
      _setError('// Error: $err');
    } finally {
      _setBusy(false);
    }
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_consoleScroll.hasClients) {
        _consoleScroll.animateTo(
          _consoleScroll.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  int? _choiceIndexFromKey(LogicalKeyboardKey key) {
    if (key == LogicalKeyboardKey.digit1 || key == LogicalKeyboardKey.numpad1) {
      return 0;
    }
    if (key == LogicalKeyboardKey.digit2 || key == LogicalKeyboardKey.numpad2) {
      return 1;
    }
    if (key == LogicalKeyboardKey.digit3 || key == LogicalKeyboardKey.numpad3) {
      return 2;
    }
    if (key == LogicalKeyboardKey.digit4 || key == LogicalKeyboardKey.numpad4) {
      return 3;
    }
    if (key == LogicalKeyboardKey.digit5 || key == LogicalKeyboardKey.numpad5) {
      return 4;
    }
    if (key == LogicalKeyboardKey.digit6 || key == LogicalKeyboardKey.numpad6) {
      return 5;
    }
    if (key == LogicalKeyboardKey.digit7 || key == LogicalKeyboardKey.numpad7) {
      return 6;
    }
    if (key == LogicalKeyboardKey.digit8 || key == LogicalKeyboardKey.numpad8) {
      return 7;
    }
    if (key == LogicalKeyboardKey.digit9 || key == LogicalKeyboardKey.numpad9) {
      return 8;
    }
    return null;
  }

  KeyEventResult _handleKeyboardShortcut(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) {
      return KeyEventResult.ignored;
    }
    if (_editorFocusNode.hasFocus) {
      return KeyEventResult.ignored;
    }

    final state = _state;
    final key = event.logicalKey;

    if (key == LogicalKeyboardKey.space ||
        key == LogicalKeyboardKey.enter ||
        key == LogicalKeyboardKey.numpadEnter) {
      if (_busy || state == null || state.finished) {
        return KeyEventResult.ignored;
      }
      _advance();
      return KeyEventResult.handled;
    }

    final choiceIndex = _choiceIndexFromKey(key);
    if (choiceIndex == null) {
      return KeyEventResult.ignored;
    }

    final choices = state?.current?.choices;
    if (_busy ||
        state == null ||
        state.finished ||
        choices == null ||
        choiceIndex >= choices.length) {
      return KeyEventResult.ignored;
    }

    _choose(choiceIndex);
    return KeyEventResult.handled;
  }

  bool get _isDesktop =>
      MediaQuery.sizeOf(context).width >= 768;

  bool get _hasPendingChoice {
    final state = _state;
    if (state == null || state.finished) {
      return false;
    }
    final current = state.current;
    return current != null && current.choices.isNotEmpty;
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: _keyboardFocusNode,
      autofocus: true,
      onKeyEvent: _handleKeyboardShortcut,
      child: Scaffold(
        body: SafeArea(
          child: Column(
            children: [
              _buildTitleBar(),
              if (_error != null) _buildErrorBar(),
              Expanded(
                child: _isDesktop ? _buildDesktopLayout() : _buildMobileLayout(),
              ),
            ],
          ),
        ),
      ),
    );
  }

  // ─── Title Bar ───
  Widget _buildTitleBar() {
    return Container(
      height: 36,
      color: _C.tabBar,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          Icon(Icons.auto_stories, size: 16, color: _C.accent),
          const SizedBox(width: 8),
          Text(
            'StoryScript Player',
            style: _mono.copyWith(fontSize: 12, color: _C.textDim),
          ),
          const Spacer(),
          if (_state != null) ...[
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: _state!.finished ? _C.errorRed.withValues(alpha: 0.2) : _C.green.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(3),
              ),
              child: Text(
                _state!.finished ? 'FINISHED' : 'RUNNING',
                style: _mono.copyWith(
                  fontSize: 10,
                  color: _state!.finished ? _C.errorRed : _C.green,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  // ─── Error Bar ───
  Widget _buildErrorBar() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      color: _C.errorRed.withValues(alpha: 0.15),
      child: Row(
        children: [
          const Icon(Icons.error_outline, size: 14, color: _C.errorRed),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              _error!,
              style: _mono.copyWith(fontSize: 12, color: _C.errorRed),
            ),
          ),
          InkWell(
            onTap: _clearError,
            child: const Icon(Icons.close, size: 14, color: _C.errorRed),
          ),
        ],
      ),
    );
  }

  // ─── Desktop: side-by-side ───
  Widget _buildDesktopLayout() {
    return Row(
      children: [
        Expanded(flex: 5, child: _buildEditorPanel()),
        Container(width: 1, color: _C.border),
        Expanded(flex: 5, child: _buildViewerPanel()),
      ],
    );
  }

  // ─── Mobile: editor only, with FAB to open viewer ───
  Widget _buildMobileLayout() {
    return Stack(
      children: [
        _buildEditorPanel(),
        Positioned(
          right: 12,
          bottom: 12,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (_state != null)
                _fabButton(
                  icon: Icons.terminal,
                  label: 'Console',
                  onTap: () => Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => _MobileViewerPage(
                        state: _state,
                        busy: _busy,
                        hasPendingChoice: _hasPendingChoice,
                        consoleScroll: _consoleScroll,
                        onAdvance: _advance,
                        onChoose: _choose,
                        onRestart: _loadStory,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _fabButton({
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Material(
        color: _C.buttonBg,
        borderRadius: BorderRadius.circular(6),
        child: InkWell(
          borderRadius: BorderRadius.circular(6),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(icon, size: 16, color: Colors.white),
                const SizedBox(width: 6),
                Text(label, style: _mono.copyWith(fontSize: 12, color: Colors.white)),
              ],
            ),
          ),
        ),
      ),
    );
  }

  // ─── Editor Panel ───
  Widget _buildEditorPanel() {
    return Column(
      children: [
        _panelTab(
          icon: Icons.edit_note,
          label: 'editor.storyscript',
          trailing: _buildEditorActions(),
        ),
        Expanded(
          child: Container(
            color: _C.panel,
            child: TextField(
              controller: _sourceController,
              focusNode: _editorFocusNode,
              maxLines: null,
              expands: true,
              textAlignVertical: TextAlignVertical.top,
              onTapOutside: (_) {
                _editorFocusNode.unfocus();
                _keyboardFocusNode.requestFocus();
              },
              style: _mono.copyWith(fontSize: 13, height: 1.6),
              cursorColor: _C.accent,
              decoration: InputDecoration(
                contentPadding: const EdgeInsets.all(16),
                border: InputBorder.none,
                hintText:
                    'script "demo" {\n'
                    '  scene start {\n'
                    '    narration "Hello from StoryScript"\n'
                    '  }\n'
                    '}',
                hintStyle: _mono.copyWith(color: _C.textDim.withValues(alpha: 0.5)),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildEditorActions() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _toolbarButton(
          icon: Icons.play_arrow,
          tooltip: 'Run',
          color: _C.green,
          onTap: _busy ? null : _loadStory,
        ),
        _toolbarButton(
          icon: Icons.skip_next,
          tooltip: 'Advance',
          color: _C.accent,
          onTap: _busy || _state == null || _state!.finished || _hasPendingChoice
              ? null
              : _advance,
        ),
        _toolbarButton(
          icon: Icons.replay,
          tooltip: 'Restart',
          color: _C.yellow,
          onTap: _busy ? null : _loadStory,
        ),
        if (_busy)
          const Padding(
            padding: EdgeInsets.only(left: 8),
            child: SizedBox(
              width: 14,
              height: 14,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: _C.accent,
              ),
            ),
          ),
      ],
    );
  }

  // ─── Viewer Panel (console style) ───
  Widget _buildViewerPanel() {
    return Column(
      children: [
        _panelTab(
          icon: Icons.terminal,
          label: 'output',
          trailing: _buildViewerActions(),
        ),
        Expanded(
          child: Container(
            color: _C.panel,
            child: _buildConsoleContent(),
          ),
        ),
        if (_state != null) _buildVariableBar(),
      ],
    );
  }

  Widget _buildViewerActions() {
    if (!_isDesktop) return const SizedBox.shrink();
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _toolbarButton(
          icon: Icons.skip_next,
          tooltip: 'Advance',
          color: _C.accent,
          onTap: _busy || _state == null || _state!.finished || _hasPendingChoice
              ? null
              : _advance,
        ),
        _toolbarButton(
          icon: Icons.replay,
          tooltip: 'Restart',
          color: _C.yellow,
          onTap: _busy ? null : _loadStory,
        ),
      ],
    );
  }

  Widget _buildConsoleContent() {
    final state = _state;
    if (state == null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.terminal, size: 48, color: _C.textDim.withValues(alpha: 0.3)),
            const SizedBox(height: 12),
            Text(
              'Waiting for input...',
              style: _mono.copyWith(color: _C.textDim),
            ),
            const SizedBox(height: 4),
            Text(
              'Write StoryScript in the editor and press Run',
              style: _mono.copyWith(fontSize: 11, color: _C.textDim.withValues(alpha: 0.6)),
            ),
          ],
        ),
      );
    }

    final history = state.history;
    final current = state.current;

    return ListView(
      controller: _consoleScroll,
      padding: const EdgeInsets.all(12),
      children: [
        // Session header
        _consoleLine('> session started', color: _C.green),
        _consoleLine(
          '  script: "${state.scriptName}"  scene: "${state.scene}"',
          color: _C.textDim,
        ),
        const SizedBox(height: 8),

        // History
        for (final step in history) _buildHistoryStep(step),

        // Current step
        if (current != null) ...[
          const Divider(color: _C.border, height: 16),
          _buildCurrentStep(current, state.finished),
        ],

        // Choices
        if (current != null && current.choices.isNotEmpty && !state.finished)
          _buildChoices(current.choices),

        // Finished marker
        if (state.finished) ...[
          const SizedBox(height: 12),
          _consoleLine('> story finished', color: _C.keyword),
          _consoleLine('  press Restart to play again', color: _C.textDim),
        ],
      ],
    );
  }

  Widget _buildHistoryStep(BridgeStep step) {
    final actor = step.actorName;
    final hasActor = actor != null && actor.isNotEmpty;
    final text = step.text ?? '';

    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                hasActor ? '[$actor]' : '[${step.kind}]',
                style: _mono.copyWith(
                  fontSize: 12,
                  color: hasActor ? _C.accent : _C.keyword,
                ),
              ),
              if (step.emotion != null && step.emotion!.isNotEmpty) ...[
                const SizedBox(width: 6),
                Text(
                  '(${step.emotion})',
                  style: _mono.copyWith(fontSize: 11, color: _C.yellow),
                ),
              ],
            ],
          ),
          if (text.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(left: 4),
              child: Text(text, style: _mono.copyWith(color: _C.text.withValues(alpha: 0.7))),
            ),
        ],
      ),
    );
  }

  Widget _buildCurrentStep(BridgeStep step, bool finished) {
    final actor = step.actorName;
    final hasActor = actor != null && actor.isNotEmpty;
    final text = step.text ?? '';

    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: _C.selection.withValues(alpha: 0.4),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: _C.accent.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                hasActor ? '[$actor]' : '[${step.kind}]',
                style: _mono.copyWith(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: hasActor ? _C.accent : _C.keyword,
                ),
              ),
              if (step.emotion != null && step.emotion!.isNotEmpty) ...[
                const SizedBox(width: 6),
                Text(
                  '(${step.emotion})',
                  style: _mono.copyWith(fontSize: 11, color: _C.yellow),
                ),
              ],
              const SizedBox(width: 6),
              Text(
                step.kind,
                style: _mono.copyWith(fontSize: 10, color: _C.textDim),
              ),
            ],
          ),
          if (text.isNotEmpty) ...[
            const SizedBox(height: 6),
            Text(text, style: _mono.copyWith(fontSize: 14, color: _C.text)),
          ],
        ],
      ),
    );
  }

  Widget _buildChoices(List<BridgeChoice> choices) {
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '> select an option:',
            style: _mono.copyWith(fontSize: 12, color: _C.green),
          ),
          const SizedBox(height: 6),
          for (var i = 0; i < choices.length; i++)
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: InkWell(
                onTap: _busy ? null : () => _choose(i),
                borderRadius: BorderRadius.circular(4),
                hoverColor: _C.selection,
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                  decoration: BoxDecoration(
                    color: _C.surface,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: _C.border),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 22,
                        height: 22,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(
                          color: _C.accent.withValues(alpha: 0.2),
                          borderRadius: BorderRadius.circular(3),
                        ),
                        child: Text(
                          '${i + 1}',
                          style: _mono.copyWith(fontSize: 12, color: _C.accent),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Text(
                          choices[i].text,
                          style: _mono.copyWith(color: _C.orange),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildVariableBar() {
    final vars = _state!.variables;
    if (vars.isEmpty) return const SizedBox.shrink();

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: const BoxDecoration(
        color: _C.tabBar,
        border: Border(top: BorderSide(color: _C.border)),
      ),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            Text('vars ', style: _mono.copyWith(fontSize: 11, color: _C.textDim)),
            for (final v in vars)
              Container(
                margin: const EdgeInsets.only(right: 8),
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: _C.surface,
                  borderRadius: BorderRadius.circular(3),
                  border: Border.all(color: _C.border),
                ),
                child: Text(
                  '${v.name}: ${v.value}',
                  style: _mono.copyWith(fontSize: 11, color: _C.green),
                ),
              ),
          ],
        ),
      ),
    );
  }

  // ─── Shared Widgets ───

  Widget _panelTab({
    required IconData icon,
    required String label,
    Widget? trailing,
  }) {
    return Container(
      height: 34,
      color: _C.tabBar,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      child: Row(
        children: [
          Icon(icon, size: 14, color: _C.textDim),
          const SizedBox(width: 6),
          Text(label, style: _mono.copyWith(fontSize: 12, color: _C.text)),
          const Spacer(),
          if (trailing != null) trailing,
        ],
      ),
    );
  }

  Widget _toolbarButton({
    required IconData icon,
    required String tooltip,
    required Color color,
    VoidCallback? onTap,
  }) {
    final disabled = onTap == null;
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(4),
          child: Icon(
            icon,
            size: 16,
            color: disabled ? _C.textDim.withValues(alpha: 0.4) : color,
          ),
        ),
      ),
    );
  }

  Widget _consoleLine(String text, {Color? color}) {
    return Text(text, style: _mono.copyWith(fontSize: 12, color: color ?? _C.text));
  }
}

// ─── Mobile Viewer Page ───
class _MobileViewerPage extends StatelessWidget {
  final BridgeState? state;
  final bool busy;
  final bool hasPendingChoice;
  final ScrollController consoleScroll;
  final VoidCallback onAdvance;
  final void Function(int) onChoose;
  final VoidCallback onRestart;

  const _MobileViewerPage({
    required this.state,
    required this.busy,
    required this.hasPendingChoice,
    required this.consoleScroll,
    required this.onAdvance,
    required this.onChoose,
    required this.onRestart,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: _C.bg,
      body: SafeArea(
        child: Column(
          children: [
            // Title bar
            Container(
              height: 36,
              color: _C.tabBar,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                children: [
                  InkWell(
                    onTap: () => Navigator.of(context).pop(),
                    child: const Icon(Icons.arrow_back, size: 18, color: _C.text),
                  ),
                  const SizedBox(width: 8),
                  const Icon(Icons.terminal, size: 14, color: _C.textDim),
                  const SizedBox(width: 6),
                  Text(
                    'output',
                    style: _mono.copyWith(fontSize: 12, color: _C.text),
                  ),
                  const Spacer(),
                  if (state != null) ...[
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: state!.finished
                            ? _C.errorRed.withValues(alpha: 0.2)
                            : _C.green.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(3),
                      ),
                      child: Text(
                        state!.finished ? 'FINISHED' : 'RUNNING',
                        style: _mono.copyWith(
                          fontSize: 10,
                          color: state!.finished ? _C.errorRed : _C.green,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            // Console output
            Expanded(
              child: Container(
                color: _C.panel,
                child: _buildMobileConsole(context),
              ),
            ),
            // Bottom action bar
            Container(
              color: _C.tabBar,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                children: [
                  _mobileAction(
                    icon: Icons.skip_next,
                    label: 'Advance',
                    color: _C.accent,
                    onTap: busy || state == null || state!.finished || hasPendingChoice
                        ? null
                        : onAdvance,
                  ),
                  const SizedBox(width: 8),
                  _mobileAction(
                    icon: Icons.replay,
                    label: 'Restart',
                    color: _C.yellow,
                    onTap: busy ? null : onRestart,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _mobileAction({
    required IconData icon,
    required String label,
    required Color color,
    VoidCallback? onTap,
  }) {
    final disabled = onTap == null;
    return Expanded(
      child: Material(
        color: disabled ? _C.surface : color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(6),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(6),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 10),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(icon, size: 16, color: disabled ? _C.textDim : color),
                const SizedBox(width: 6),
                Text(
                  label,
                  style: _mono.copyWith(
                    fontSize: 12,
                    color: disabled ? _C.textDim : color,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildMobileConsole(BuildContext context) {
    final s = state;
    if (s == null) {
      return Center(
        child: Text(
          'No story loaded.',
          style: _mono.copyWith(color: _C.textDim),
        ),
      );
    }

    final history = s.history;
    final current = s.current;

    return ListView(
      controller: consoleScroll,
      padding: const EdgeInsets.all(12),
      children: [
        _consoleLine('> session started', color: _C.green),
        _consoleLine(
          '  script: "${s.scriptName}"  scene: "${s.scene}"',
          color: _C.textDim,
        ),
        const SizedBox(height: 8),
        for (final step in history) _buildHistoryStep(step),
        if (current != null) ...[
          const Divider(color: _C.border, height: 16),
          _buildCurrentStep(current, s.finished),
        ],
        if (current != null && current.choices.isNotEmpty && !s.finished)
          _buildChoices(current.choices),
        if (s.finished) ...[
          const SizedBox(height: 12),
          _consoleLine('> story finished', color: _C.keyword),
        ],
      ],
    );
  }

  Widget _buildHistoryStep(BridgeStep step) {
    final actor = step.actorName;
    final hasActor = actor != null && actor.isNotEmpty;
    final text = step.text ?? '';
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            hasActor ? '[$actor]' : '[${step.kind}]',
            style: _mono.copyWith(
              fontSize: 12,
              color: hasActor ? _C.accent : _C.keyword,
            ),
          ),
          if (text.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(left: 4),
              child: Text(text, style: _mono.copyWith(color: _C.text.withValues(alpha: 0.7))),
            ),
        ],
      ),
    );
  }

  Widget _buildCurrentStep(BridgeStep step, bool finished) {
    final actor = step.actorName;
    final hasActor = actor != null && actor.isNotEmpty;
    final text = step.text ?? '';
    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: _C.selection.withValues(alpha: 0.4),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: _C.accent.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                hasActor ? '[$actor]' : '[${step.kind}]',
                style: _mono.copyWith(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: hasActor ? _C.accent : _C.keyword,
                ),
              ),
              const SizedBox(width: 6),
              Text(step.kind, style: _mono.copyWith(fontSize: 10, color: _C.textDim)),
            ],
          ),
          if (text.isNotEmpty) ...[
            const SizedBox(height: 6),
            Text(text, style: _mono.copyWith(fontSize: 14, color: _C.text)),
          ],
        ],
      ),
    );
  }

  Widget _buildChoices(List<BridgeChoice> choices) {
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _consoleLine('> select an option:', color: _C.green),
          const SizedBox(height: 6),
          for (var i = 0; i < choices.length; i++)
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: InkWell(
                onTap: busy ? null : () => onChoose(i),
                borderRadius: BorderRadius.circular(4),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                  decoration: BoxDecoration(
                    color: _C.surface,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: _C.border),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 22,
                        height: 22,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(
                          color: _C.accent.withValues(alpha: 0.2),
                          borderRadius: BorderRadius.circular(3),
                        ),
                        child: Text(
                          '${i + 1}',
                          style: _mono.copyWith(fontSize: 12, color: _C.accent),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Text(
                          choices[i].text,
                          style: _mono.copyWith(color: _C.orange),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _consoleLine(String text, {Color? color}) {
    return Text(text, style: _mono.copyWith(fontSize: 12, color: color ?? _C.text));
  }
}
