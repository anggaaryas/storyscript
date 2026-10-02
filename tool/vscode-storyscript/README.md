# StoryScript Syntax Highlighting for VS Code

Provides syntax highlighting and language support for `.StoryScript` files.

## Features

- **Workspace localization intelligence**: discovers `StoryScript.toml`, root/includes,
  and complete `<localization.root>/<locale>.ftl` catalogs without a Rust executable.
  Keyed `@"message-id"` completion, catalog `$variable` completion, cross-file
  definitions/references, workspace symbols, versioned atomic message rename and
  append-only synchronization actions use source token / Fluent AST ranges.
- **Advisory catalog diagnostics**: coverage, duplicate/unused IDs, canonical locales,
  malformed FTL, profile restrictions, transitive variable drift/scope/arrays,
  dependency cycles and unsafe paths. Open buffers override disk; watchers rebuild
  closed files. Rename refuses ambiguous IDs, malformed catalogs, collisions and
  disk changes since indexing. Sync never overwrites translations or removes entries.
- Existing scene/actor/variable providers remain available. Editor analysis is not
  a compiler: run `storyscript-bundle localize check --project <root>` before export.
  Fluent terms and numeric exactness remain subject to the normative Rust profile.

## Development verification

```sh
npm ci
npm test
npm run compile
npm run package
npm audit
```

`server/test/localization.test.ts` uses isolated filesystem fixtures, not VS Code
or live infrastructure. The extension owns `.ftl` highlighting and observes config,
source and catalog changes. No CLI binary is bundled or invoked.

- **Syntax Highlighting** for all StoryScript language constructs:
  - Scene definitions (`* scene_name { }`)
  - `* INIT` block
  - Phase tags (`#PREP`, `#STORY`)
  - Engine directives (`@bg`, `@bgm`, `@sfx`, `@actor`, `@start`)
  - Navigation directives (`@choice`, `@jump`, `@end`)
  - Dialogue — portrait form (`ACTOR(emotion, Position): "..."`) and name-only form (`ACTOR: "..."`)
  - Variables (`$variable_name`)
  - Typed declarations in `* INIT` (`$var as integer|string|boolean|decimal = ...`)
  - Standalone STORY variable output (`$variable_name` line)
  - Inline interpolation placeholders (`${variable_name}`) in strings
  - Control flow (`if`, `else`)
  - Arithmetic operators (`+`, `-`, `*`, `/`, `%`)
  - Function-style expressions (`abs(...)`, `rand(...)`, `pick([ ... ])`)
  - List literals (`[a, b, c]`)
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
| Type annotation (`as`) | `keyword.control.type.as` |
| Type names (`integer`, `string`, `boolean`, `decimal`) | `storage.type` |
| Interpolation (`${var}`) | `meta.interpolation` + `variable.other` |
| Strings | `string.quoted.double` |
| Numbers | `constant.numeric` |
| `true` / `false` | `constant.language.boolean` |
| `STOP` | `constant.language.stop` |
| `if` / `else` | `keyword.control.flow` |
| `->` | `keyword.operator.arrow` |
| Scene refs (jump/choice targets) | `entity.name.function.scene-ref` |
