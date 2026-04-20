enum StoryScriptTokenType {
  comment,
  keyword,
  directive,
  scene,
  sceneReference,
  actor,
  emotion,
  position,
  variable,
  string,
  interpolation,
  number,
  boolean,
  type,
  operator,
  arrow,
  punctuation,
  constant,
  escape,
}

class StoryScriptHighlightToken {
  const StoryScriptHighlightToken({
    required this.start,
    required this.end,
    required this.type,
  });

  final int start;
  final int end;
  final StoryScriptTokenType type;
}

class StoryScriptSyntaxEngine {
  const StoryScriptSyntaxEngine();

  static final RegExp _initBlock = RegExp(r'(\*)(\s+)(INIT)\b');
  static final RegExp _requireBlock = RegExp(r'(\*)(\s+)(REQUIRE)\b');
  static final RegExp _sceneBlock = RegExp(
    r'(\*)(\s+)(?!INIT\b)(?!REQUIRE\b)([a-zA-Z_][a-zA-Z0-9_]*)\b',
  );
  static final RegExp _phaseTag = RegExp(r'(#)(PREP|STORY)\b');

  static final RegExp _directiveActor = RegExp(
    r'(@actor)\s+([A-Z_][A-Z0-9_]*)',
  );
  static final RegExp _directiveStart = RegExp(
    r'(@start)\s+([a-zA-Z_][a-zA-Z0-9_]*)',
  );
  static final RegExp _directiveJump = RegExp(
    r'(@jump)\s+([a-zA-Z_][a-zA-Z0-9_]*)',
  );
  static final RegExp _simpleDirective = RegExp(
    r'@(?:include|bg|bgm|sfx|end|choice)\b',
  );

  static final RegExp _dialoguePortrait = RegExp(
    r'([A-Z_][A-Z0-9_]*)\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*,\s*(Left|Right|Center|L|R|C)\s*\)\s*(:)',
  );
  static final RegExp _dialogueSimple = RegExp(
    r'([A-Z_][A-Z0-9_]*)\s*(:)\s*(?=")',
  );

  static final RegExp _controlFlow = RegExp(
    r'\b(?:logic|return|if|else|for|repeat|break|continue|in|snapshot)\b',
  );
  static final RegExp _asKeyword = RegExp(r'\bas\b');
  static final RegExp _typeName = RegExp(
    r'\b(?:integer|string|boolean|decimal|array)\b',
  );
  static final RegExp _variable = RegExp(r'\$[a-zA-Z_][a-zA-Z0-9_]*');
  static final RegExp _boolean = RegExp(r'\b(?:true|false)\b');
  static final RegExp _stop = RegExp(r'\bSTOP\b');
  static final RegExp _number = RegExp(r'\b[0-9]+(?:\.[0-9]+)?\b');
  static final RegExp _arrow = RegExp(r'->');
  static final RegExp _operator = RegExp(
    r'==|!=|<=|>=|\+=|-=|=|\+|-|\*|/|%|<|>',
  );
  static final RegExp _punctuation = RegExp(r'[{}()\[\],;:]');
  static final RegExp _identifier = RegExp(r'[a-zA-Z_][a-zA-Z0-9_]*');

  List<StoryScriptHighlightToken> tokenize(String source) {
    if (source.isEmpty) {
      return const [];
    }

    final tokens = <StoryScriptHighlightToken>[];
    var index = 0;
    StoryScriptTokenType? previousSignificantType;

    while (index < source.length) {
      if (source.startsWith('//', index)) {
        final lineEnd = source.indexOf('\n', index);
        final end = lineEnd == -1 ? source.length : lineEnd;
        _addToken(tokens, index, end, StoryScriptTokenType.comment);
        previousSignificantType = StoryScriptTokenType.comment;
        index = end;
        continue;
      }

      if (source.codeUnitAt(index) == 0x22) {
        index = _consumeString(source, index, tokens);
        previousSignificantType = StoryScriptTokenType.string;
        continue;
      }

      RegExpMatch? match;

      match = _prefix(_initBlock, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.operator);
        _addGroupToken(tokens, match, 3, StoryScriptTokenType.keyword);
        previousSignificantType = StoryScriptTokenType.keyword;
        index = match.end;
        continue;
      }

      match = _prefix(_requireBlock, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.operator);
        _addGroupToken(tokens, match, 3, StoryScriptTokenType.keyword);
        previousSignificantType = StoryScriptTokenType.keyword;
        index = match.end;
        continue;
      }

      match = _prefix(_sceneBlock, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.operator);
        _addGroupToken(tokens, match, 3, StoryScriptTokenType.scene);
        previousSignificantType = StoryScriptTokenType.scene;
        index = match.end;
        continue;
      }

      match = _prefix(_phaseTag, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.punctuation);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.keyword);
        previousSignificantType = StoryScriptTokenType.keyword;
        index = match.end;
        continue;
      }

      match = _prefix(_directiveActor, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.directive);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.actor);
        previousSignificantType = StoryScriptTokenType.actor;
        index = match.end;
        continue;
      }

      match = _prefix(_directiveStart, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.directive);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.sceneReference);
        previousSignificantType = StoryScriptTokenType.sceneReference;
        index = match.end;
        continue;
      }

      match = _prefix(_directiveJump, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.directive);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.sceneReference);
        previousSignificantType = StoryScriptTokenType.sceneReference;
        index = match.end;
        continue;
      }

      match = _prefix(_simpleDirective, source, index);
      if (match != null) {
        _addToken(
          tokens,
          match.start,
          match.end,
          StoryScriptTokenType.directive,
        );
        previousSignificantType = StoryScriptTokenType.directive;
        index = match.end;
        continue;
      }

      match = _prefix(_dialoguePortrait, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.actor);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.emotion);
        _addGroupToken(tokens, match, 3, StoryScriptTokenType.position);
        _addGroupToken(tokens, match, 4, StoryScriptTokenType.punctuation);
        previousSignificantType = StoryScriptTokenType.actor;
        index = match.end;
        continue;
      }

      match = _prefix(_dialogueSimple, source, index);
      if (match != null) {
        _addGroupToken(tokens, match, 1, StoryScriptTokenType.actor);
        _addGroupToken(tokens, match, 2, StoryScriptTokenType.punctuation);
        previousSignificantType = StoryScriptTokenType.actor;
        index = match.end;
        continue;
      }

      match = _prefix(_controlFlow, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.keyword);
        previousSignificantType = StoryScriptTokenType.keyword;
        index = match.end;
        continue;
      }

      match = _prefix(_asKeyword, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.keyword);
        previousSignificantType = StoryScriptTokenType.keyword;
        index = match.end;
        continue;
      }

      match = _prefix(_typeName, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.type);
        previousSignificantType = StoryScriptTokenType.type;
        index = match.end;
        continue;
      }

      match = _prefix(_variable, source, index);
      if (match != null) {
        _addToken(
          tokens,
          match.start,
          match.end,
          StoryScriptTokenType.variable,
        );
        previousSignificantType = StoryScriptTokenType.variable;
        index = match.end;
        continue;
      }

      match = _prefix(_boolean, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.boolean);
        previousSignificantType = StoryScriptTokenType.boolean;
        index = match.end;
        continue;
      }

      match = _prefix(_stop, source, index);
      if (match != null) {
        _addToken(
          tokens,
          match.start,
          match.end,
          StoryScriptTokenType.constant,
        );
        previousSignificantType = StoryScriptTokenType.constant;
        index = match.end;
        continue;
      }

      match = _prefix(_number, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.number);
        previousSignificantType = StoryScriptTokenType.number;
        index = match.end;
        continue;
      }

      match = _prefix(_arrow, source, index);
      if (match != null) {
        _addToken(tokens, match.start, match.end, StoryScriptTokenType.arrow);
        previousSignificantType = StoryScriptTokenType.arrow;
        index = match.end;
        continue;
      }

      match = _prefix(_operator, source, index);
      if (match != null) {
        _addToken(
          tokens,
          match.start,
          match.end,
          StoryScriptTokenType.operator,
        );
        previousSignificantType = StoryScriptTokenType.operator;
        index = match.end;
        continue;
      }

      match = _prefix(_punctuation, source, index);
      if (match != null) {
        _addToken(
          tokens,
          match.start,
          match.end,
          StoryScriptTokenType.punctuation,
        );
        previousSignificantType = StoryScriptTokenType.punctuation;
        index = match.end;
        continue;
      }

      match = _prefix(_identifier, source, index);
      if (match != null) {
        if (previousSignificantType == StoryScriptTokenType.arrow) {
          _addToken(
            tokens,
            match.start,
            match.end,
            StoryScriptTokenType.sceneReference,
          );
          previousSignificantType = StoryScriptTokenType.sceneReference;
        }
        index = match.end;
        continue;
      }

      index += 1;
    }

    return tokens;
  }

  static int _consumeString(
    String source,
    int start,
    List<StoryScriptHighlightToken> tokens,
  ) {
    var cursor = start + 1;
    var segmentStart = start;

    while (cursor < source.length) {
      final char = source.codeUnitAt(cursor);

      if (char == 0x5C) {
        if (cursor + 1 < source.length &&
            source.codeUnitAt(cursor + 1) == 0x24) {
          _addToken(tokens, segmentStart, cursor, StoryScriptTokenType.string);
          _addToken(tokens, cursor, cursor + 2, StoryScriptTokenType.escape);
          cursor += 2;
          segmentStart = cursor;
          continue;
        }

        cursor += (cursor + 1 < source.length) ? 2 : 1;
        continue;
      }

      if (char == 0x24 &&
          cursor + 1 < source.length &&
          source.codeUnitAt(cursor + 1) == 0x7B) {
        _addToken(tokens, segmentStart, cursor, StoryScriptTokenType.string);
        _addToken(
          tokens,
          cursor,
          cursor + 2,
          StoryScriptTokenType.interpolation,
        );
        cursor += 2;

        final variableMatch = _prefix(_identifier, source, cursor);
        if (variableMatch != null) {
          _addToken(
            tokens,
            variableMatch.start,
            variableMatch.end,
            StoryScriptTokenType.variable,
          );
          cursor = variableMatch.end;
        }

        while (cursor < source.length && source.codeUnitAt(cursor) != 0x7D) {
          cursor += 1;
        }

        if (cursor < source.length && source.codeUnitAt(cursor) == 0x7D) {
          _addToken(
            tokens,
            cursor,
            cursor + 1,
            StoryScriptTokenType.interpolation,
          );
          cursor += 1;
        }

        segmentStart = cursor;
        continue;
      }

      if (char == 0x22) {
        cursor += 1;
        _addToken(tokens, segmentStart, cursor, StoryScriptTokenType.string);
        return cursor;
      }

      cursor += 1;
    }

    _addToken(tokens, segmentStart, cursor, StoryScriptTokenType.string);
    return cursor;
  }

  static RegExpMatch? _prefix(RegExp regex, String source, int index) {
    return regex.matchAsPrefix(source, index) as RegExpMatch?;
  }

  static void _addGroupToken(
    List<StoryScriptHighlightToken> tokens,
    RegExpMatch match,
    int group,
    StoryScriptTokenType type,
  ) {
    final full = match.group(0);
    final capture = match.group(group);

    if (full == null || full.isEmpty || capture == null || capture.isEmpty) {
      return;
    }

    final relativeStart = full.indexOf(capture);
    if (relativeStart < 0) {
      return;
    }

    final start = match.start + relativeStart;
    final end = start + capture.length;
    _addToken(tokens, start, end, type);
  }

  static void _addToken(
    List<StoryScriptHighlightToken> tokens,
    int start,
    int end,
    StoryScriptTokenType type,
  ) {
    if (end <= start) {
      return;
    }

    tokens.add(StoryScriptHighlightToken(start: start, end: end, type: type));
  }
}
