import 'package:flutter/material.dart';

import 'code_highlighter_engine.dart';

class StoryScriptCodeHighlighterTheme {
  const StoryScriptCodeHighlighterTheme({
    // Base text — default light foreground on dark backgrounds.
    this.baseStyle = const TextStyle(
      color: Color(0xFFD4D4D4),
      fontFamily: 'monospace',
      fontSize: 14,
      height: 1.6,
    ),
    // Comments — muted green, italic.
    this.commentStyle = const TextStyle(
      color: Color(0xFF6A9955),
      fontStyle: FontStyle.italic,
    ),
    // Keywords — purple/magenta (logic, return, if, else, for, repeat, as, …).
    this.keywordStyle = const TextStyle(color: Color(0xFFC586C0)),
    // Directives — accent blue for @bg, @actor, @start, ….
    this.directiveStyle = const TextStyle(color: Color(0xFF569CD6)),
    // Scene names — teal, bold (type/class-name colour).
    this.sceneStyle = const TextStyle(
      color: Color(0xFF4EC9B0),
      fontWeight: FontWeight.w600,
    ),
    // Scene references in jump/choice targets.
    this.sceneReferenceStyle = const TextStyle(color: Color(0xFF4EC9B0)),
    // Actor IDs — teal bold (same family as scene/type names).
    this.actorStyle = const TextStyle(
      color: Color(0xFF4EC9B0),
      fontWeight: FontWeight.w600,
    ),
    // Emotion keys — light-blue (parameter/field colour).
    this.emotionStyle = const TextStyle(color: Color(0xFF9CDCFE)),
    // Position constants (Left, Right, Center) — yellow-green (enum member).
    this.positionStyle = const TextStyle(color: Color(0xFFDCDCAA)),
    // Variables ($var) — light-blue.
    this.variableStyle = const TextStyle(color: Color(0xFF9CDCFE)),
    // Strings — warm orange tone for string literals.
    this.stringStyle = const TextStyle(color: Color(0xFFCE9178)),
    // Numbers — muted light-green.
    this.numberStyle = const TextStyle(color: Color(0xFFB5CEA8)),
    // Booleans (true/false) — same blue as keywords.
    this.booleanStyle = const TextStyle(color: Color(0xFF569CD6)),
    // Type names (integer, string, boolean, decimal, array) — teal.
    this.typeStyle = const TextStyle(color: Color(0xFF4EC9B0)),
    // Operators — plain foreground.
    this.operatorStyle = const TextStyle(color: Color(0xFFD4D4D4)),
    // Arrow (→) — plain foreground.
    this.arrowStyle = const TextStyle(color: Color(0xFFD4D4D4)),
    // Punctuation — plain foreground.
    this.punctuationStyle = const TextStyle(color: Color(0xFFD4D4D4)),
    // Special constants (STOP) — bright blue.
    this.constantStyle = const TextStyle(color: Color(0xFF4FC1FF)),
    // Interpolation delimiters ${ } — accent blue.
    this.interpolationStyle = const TextStyle(color: Color(0xFF569CD6)),
    // Escape sequence \$ — gold.
    this.escapeStyle = const TextStyle(color: Color(0xFFD7BA7D)),
  });

  final TextStyle baseStyle;
  final TextStyle commentStyle;
  final TextStyle keywordStyle;
  final TextStyle directiveStyle;
  final TextStyle sceneStyle;
  final TextStyle sceneReferenceStyle;
  final TextStyle actorStyle;
  final TextStyle emotionStyle;
  final TextStyle positionStyle;
  final TextStyle variableStyle;
  final TextStyle stringStyle;
  final TextStyle numberStyle;
  final TextStyle booleanStyle;
  final TextStyle typeStyle;
  final TextStyle operatorStyle;
  final TextStyle arrowStyle;
  final TextStyle punctuationStyle;
  final TextStyle constantStyle;
  final TextStyle interpolationStyle;
  final TextStyle escapeStyle;

  static const StoryScriptCodeHighlighterTheme defaults =
      StoryScriptCodeHighlighterTheme();

  TextStyle resolve(StoryScriptTokenType tokenType, TextStyle fallbackBase) {
    return fallbackBase.merge(_styleFor(tokenType));
  }

  TextStyle _styleFor(StoryScriptTokenType tokenType) {
    switch (tokenType) {
      case StoryScriptTokenType.comment:
        return commentStyle;
      case StoryScriptTokenType.keyword:
        return keywordStyle;
      case StoryScriptTokenType.directive:
        return directiveStyle;
      case StoryScriptTokenType.scene:
        return sceneStyle;
      case StoryScriptTokenType.sceneReference:
        return sceneReferenceStyle;
      case StoryScriptTokenType.actor:
        return actorStyle;
      case StoryScriptTokenType.emotion:
        return emotionStyle;
      case StoryScriptTokenType.position:
        return positionStyle;
      case StoryScriptTokenType.variable:
        return variableStyle;
      case StoryScriptTokenType.string:
        return stringStyle;
      case StoryScriptTokenType.interpolation:
        return interpolationStyle;
      case StoryScriptTokenType.number:
        return numberStyle;
      case StoryScriptTokenType.boolean:
        return booleanStyle;
      case StoryScriptTokenType.type:
        return typeStyle;
      case StoryScriptTokenType.operator:
        return operatorStyle;
      case StoryScriptTokenType.arrow:
        return arrowStyle;
      case StoryScriptTokenType.punctuation:
        return punctuationStyle;
      case StoryScriptTokenType.constant:
        return constantStyle;
      case StoryScriptTokenType.escape:
        return escapeStyle;
    }
  }
}
