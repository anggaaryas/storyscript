![storycript logo](image/ascii-logo.png)

![Screenshot](image/ss.png)

# StoryScript (Name not final)
StoryScript (Name not final) is a game-development scripting language and toolchain for prototyping story-driven games, especially visual novels.

This repository includes:

- A language specification and working examples.
- A Rust parser/validator CLI.
- A Rust terminal-based player (TUI) to run StoryScript scenes.
- A VS Code extension for `.StoryScript` syntax highlighting.
- A Flutter plugin integration for the Rust runtime.

## Repository Layout

```text
.
├── PLAN.md                      # Language spec and validation rules
├── example/                     # Example StoryScript files
├── parser/rust/                 # storycript-parser (lexer/parser/validator)
├── player/                      # storycript-player (TUI runtime)
├── storyscript_player_core/     # Flutter plugin integration and WebAssembly bindings
└── tool/vscode-storyscript/     # VS Code language extension
```

## Prerequisites

- Rust toolchain (stable)
- Node.js + npm (for VS Code extension packaging)
- Flutter SDK (optional, for Flutter plugin and Web integration)

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
- Root-only child manifest via `@include [ ... ]` inside `* INIT`
- `#PREP` for state mutation and engine directives (`@bg`, `@bgm`, `@sfx`)
- `#STORY` for narration, dialogue, branching (`if` / `else if` / `else`), transitions, and standalone variable output (`$var`)
- Snapshot `for` and `repeat` loops in both `#PREP` and `#STORY`, with `break`/`continue` nearest-loop behavior
- Nested `@choice` entry groups using `if`, `repeat`, and `for ($item in snapshot $array)` (runtime-expanded choice cap: 9 options)
- Numeric expressions with `+`, `-`, `*`, `/`, `%` (modulo is integer-only)
- Built-ins: `abs(x)`, `rand()`, `rand(min, max)`, `pick(array)`, `pick(count, array)`, `array_push`, `array_pop`, `array_strip`, `array_clear`, `array_contains`, `array_size`, `array_join`, `array_get`, `array_insert`, `array_remove`
- `${var}` inline interpolation in string literals across all phases (`\$` for literal dollar)
- Typed declarations in `* INIT` using `as integer|string|boolean|decimal|array<...>`
- Typed local declarations in `#PREP` using the same shape: `$name as <type> = <expr>`
- Variable type is immutable after declaration

Local scope notes:

- Local declarations are allowed only in `#PREP`
- Locals are visible in both `#PREP` and `#STORY` of the same scene
- Locals are reset every time that scene is entered/re-entered
- Local names cannot collide with globals declared in `* INIT`

Module include notes:

- Only root `* INIT` can use `@include [ ... ]`
- Included child files must not define `* INIT`
- Included child files must define exactly one `* REQUIRE { ... }`
- Include paths are resolved relative to the root script file

Built-in notes:

- `rand()` and `rand(min, max)` are assignment-target driven:
	- integer target -> integer random
	- decimal target -> decimal random
- `rand(min, max)` is inclusive.
- decimal assignment allows integer/decimal bounds for `rand(min, max)`.
- decimal assignment allows integer/decimal candidates for `pick(array)` and decimal array probes in `array_contains`.
- mutating array functions (`array_push`, `array_strip`, `array_clear`, `array_insert`) are valid in `#PREP` statement form and forbidden in `#STORY`.
- collection scalar arguments (`value`, `index`, `count`, `string_separator`) must be literal or `$variable`.

Minimal example:

```StoryScript
* INIT {
	$morale as integer = 50
	$captain_name as string = "Ari"
	$threat_level as decimal = 1.5
	$alerted as boolean = false
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

Minimal modular include example:

```StoryScript
// main.StoryScript
* INIT {
	$system_stability as integer = 100
	$has_admin_key as boolean = true

	@actor TEO "Teona" {
		focus -> "teo_focus.png"
	}
	@actor GIP "Gippie" {
		alert -> "gip_alert.png"
	}

	@include ["modules/minigame_hack.StoryScript"];
	@start hack_sequence_start;
}
```

```StoryScript
// modules/minigame_hack.StoryScript
* REQUIRE {
	$system_stability as integer;
	$has_admin_key as boolean;
	@actor TEO [ focus ];
	@actor GIP [ alert ];
}

* hack_sequence_start {
	#PREP
	$system_stability = $system_stability - 15

	#STORY
	GIP(alert, Center): "Warning! Countermeasures active."
	@end;
}
```

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


## Flutter plugin integration

This repository includes a Flutter plugin wrapper around the Rust runtime at `storyscript_player_core`. It supports WebAssembly bindings for Flutter Web as well as native mobile targets.

### Build the Flutter Web bindings

```bash
cd storyscript_player_core
flutter_rust_bridge_codegen build-web --wasm-pack-rustflags "-Ctarget-feature=+atomics -Clink-args=--shared-memory -Clink-args=--max-memory=1073741824 -Clink-args=--import-memory -Clink-args=--export=__wasm_init_tls -Clink-args=--export=__tls_size -Clink-args=--export=__tls_align -Clink-args=--export=__tls_base"
```

This generates the WebAssembly artifacts required for Flutter Web when using `flutter_rust_bridge`.

### Run the Flutter web app locally

```bash
cd storyscript_player_core
flutter run --web-header=Cross-Origin-Opener-Policy=same-origin --web-header=Cross-Origin-Embedder-Policy=require-corp
```

These headers are necessary for shared-memory WebAssembly builds and allow the web app to load the generated Wasm module correctly.

### Mobile toolchain preparation

For Android and iOS native builds, install the required Rust targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

Then build the Android library with `cargo ndk`:

```bash
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o ../android/app/src/main/jniLibs build --release
```

> Note: Ensure you have Flutter installed and that `wasm-pack` is available in your PATH before running these commands.

## License

See `LICENSE`.

