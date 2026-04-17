![storycript logo](image/ascii-logo.png)

![Screenshot](image/ss.png)

# storycript-spec
StoryScript is a game-development scripting language and toolchain for prototyping story-driven games, especially visual novels.

This repository includes:

- A language specification and working examples.
- A Rust parser/validator CLI.
- A Rust terminal-based player (TUI) to run StoryScript scenes.
- A VS Code extension for `.StoryScript` syntax highlighting.

## Repository Layout

```text
.
├── PLAN.md                      # Language spec and validation rules
├── example/                     # Example StoryScript files
├── parser/rust/                 # storycript-parser (lexer/parser/validator)
├── player/                      # storycript-player (TUI runtime)
└── tool/vscode-storyscript/     # VS Code language extension
```

## Prerequisites

- Rust toolchain (stable)
- Node.js + npm (for VS Code extension packaging)

## Quick Start

### 1) Validate a StoryScript file with the parser

```bash
cd parser/rust
cargo run -- ../../example/the_last_signal.StoryScript
```

Optional JSON diagnostics output:

```bash
cargo run -- ../../example/the_last_signal.StoryScript --json
```

### 2) Run the interactive player

```bash
cd player
cargo run -- ../example/the_last_signal.StoryScript
```

If no file is passed, the player scans for `.StoryScript` files in the current directory and `../example`.

## Player Controls

- `Up/Down`: navigate chooser or scroll story view
- `Enter`: continue dialogue/narration or open selected story file
- `1-9`: choose choice options
- `Q`: quit player view (or exit app from chooser)

## StoryScript At a Glance

StoryScript uses:

- `* INIT` for global state, actors, and `@start`
- `#PREP` for state mutation and engine directives (`@bg`, `@bgm`, `@sfx`)
- `#STORY` for narration, dialogue, branching, transitions, and standalone variable output (`$var`)
- `${var}` inline interpolation in string literals across all phases (`\$` for literal dollar)

Minimal example:

```StoryScript
* INIT {
	$morale = 50
	@actor CMDR "Commander"
	@start opening
}

* opening {
	#PREP
	@bg "bridge.png"

	#STORY
	"The bridge lights flicker."
	CMDR: "Status report. Morale=${morale}"
	$morale

	@choice {
		"Investigate the signal" -> signal_room
		"Lock down the deck" -> lockdown
	}
}
```

For full syntax and validation rules, see `PLAN.md`.

## VS Code Extension

Path: `tool/vscode-storyscript`

The extension provides syntax highlighting and language configuration for `.StoryScript` files.

### Build `.vsix`

From the extension folder:

```bash
cd tool/vscode-storyscript
npm install
npx vsce package
```

That creates a `.vsix` file in the same folder.

### Install Locally

```bash
code --install-extension storyscript-syntax-0.1.0.vsix
```

You can also open the extension folder in VS Code and use the Extensions UI to install from VSIX.

## Current Status

- Parser and player crates compile successfully with `cargo check`.
- VS Code extension currently focuses on syntax highlighting (no LSP features yet).

## License

See `LICENSE`.