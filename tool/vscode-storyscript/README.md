# StoryScript Syntax Highlighting for VS Code

Provides syntax highlighting and language support for `.StoryScript` files.

## Features

- **Syntax Highlighting** for all StoryScript language constructs:
  - Scene definitions (`* scene_name { }`)
  - `* INIT` block
  - Phase tags (`#PREP`, `#STORY`)
  - Engine directives (`@bg`, `@bgm`, `@sfx`, `@actor`, `@start`)
  - Navigation directives (`@choice`, `@jump`, `@end`)
  - Dialogue — portrait form (`ACTOR(emotion, Position): "..."`) and name-only form (`ACTOR: "..."`)
  - Variables (`$variable_name`)
  - Standalone STORY variable output (`$variable_name` line)
  - Inline interpolation placeholders (`${variable_name}`) in strings
  - Control flow (`if`, `else`)
  - Choice arrows (`"Label" -> target_scene`)
  - String literals, numbers, booleans
  - Comments (`// ...`)

- **Language Configuration**:
  - Auto-closing brackets and quotes
  - Comment toggling (`Ctrl+/` / `Cmd+/`)
  - Code folding for blocks and phases
  - Smart indentation

## Installation

### From Source (Development)

1. Copy or symlink the `vscode-storyscript` folder into your VS Code extensions directory:
   - **macOS**: `~/.vscode/extensions/`
   - **Linux**: `~/.vscode/extensions/`
   - **Windows**: `%USERPROFILE%\.vscode\extensions\`
2. Restart VS Code.
3. Open any `.StoryScript` file — syntax highlighting will activate automatically.

### Quick Install (macOS/Linux)

```bash
ln -s "$(pwd)" ~/.vscode/extensions/storyscript-syntax
```

Then restart VS Code.

## Scope Reference

| Element | TextMate Scope |
| :--- | :--- |
| Comments | `comment.line.double-slash` |
| `* INIT` | `keyword.control.init` |
| `* scene_name` | `entity.name.function.scene` |
| `#PREP` / `#STORY` | `keyword.control.phase` |
| `@bg`, `@bgm`, etc. | `keyword.control.directive.*` |
| `@choice`, `@jump`, `@end` | `keyword.control.directive.*` |
| Actor ID | `entity.name.type.actor-id` |
| Emotion key | `variable.other.emotion-key` |
| Position (`Left`, `Right`, etc.) | `constant.language.position` |
| Variables (`$var`) | `variable.other` |
| Interpolation (`${var}`) | `meta.interpolation` + `variable.other` |
| Strings | `string.quoted.double` |
| Numbers | `constant.numeric` |
| `true` / `false` | `constant.language.boolean` |
| `STOP` | `constant.language.stop` |
| `if` / `else` | `keyword.control.flow` |
| `->` | `keyword.operator.arrow` |
| Scene refs (jump/choice targets) | `entity.name.function.scene-ref` |
