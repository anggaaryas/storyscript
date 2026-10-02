# StoryScript playable example

Run the Flutter example to read **Signal at Station Nine: A Borrowed Dawn**, an
illustrated, branching light-novel chapter. The checked-in authoring project
is `game/`: its `StoryScript.toml`, `story/main.StoryScript`, and SVG assets
compile into `assets/station_nine.storybundle`. The app strictly verifies that signed bundle
with `assets/station_nine_public_key.txt`, then lets the Rust headless player
execute the script. Read one beat at a time, scroll longer pages, select choices,
reach an ending, and select **Play again** to start a fresh session. The chapter
introduces Neri and Dot and branches between repairing the coolant line and
retrieving an override that saves the station at a personal cost. Each tested
English route contains at least 1,500 words. Different reactor/archive/dawn backgrounds
and Dot portraits are read from the verified archive on demand.

The toolbar opens the **separate, original Bundle Inspector**, which still
loads `assets/demo.storybundle` using `assets/demo_public_key.txt`. That fixture
is also used by Rust regression tests and has not been replaced. The inspector
shows trust status, metadata, model tree, and image previews; it does not play
the chapter. No save slots, audio playback, or external content loading are
implemented in this example.

## English / Indonesian ownership and switching

The 86 keyed narration/dialogue/choice sites live in `game/story/main.StoryScript`;
complete `game/localization/en.ftl` and `id.ftl` resources are strict build inputs.
The resident-count select and stability interpolation exercise exact scalar
snapshots. Actor names, IDs, scenes, assets and variable data remain locale-neutral.

Flutter shell copy lives separately in `lib/l10n/app_{en,id}.arb`, generated with
`flutter gen-l10n`. `app.dart` initializes ordered platform preferences, owns the
English/Indonesian language selector across both routes, and only publishes a new
shell locale after the game switch succeeds. Unsupported preferences fall back to
English, with requested/resolved story metadata displayed in the game.

The main app retains one strictly verified game bundle. `GameController` exports
an opaque locale-neutral save, restores a candidate from that same bundle lease,
and publishes it before disposing the old player. Failed restore retains the old
locale/event/progress/artwork. Restore does not replay PREP or media; artwork stays
at the saved checkpoint. No FTL, argument parsing, rich-text interpretation,
network translations or persistent language setting exists in Flutter.

Agent-owned verification (no UI runner):

```sh
flutter gen-l10n
flutter analyze
flutter test test/game_controller_locale_test.dart
```

The following existing widget/integration commands are **human-owned**, not agent
checkpoints. Humans must adapt the tests for generated shell localization and
check narrow/wide layouts, 200% text, long Indonesian choices, keyboard/semantics,
48x48 targets, busy/error feedback, fallback and failed-switch rollback on native/Web.

```bash
cd storyscript_bundle/example
flutter run -d macos
flutter test test
flutter test integration_test/storybundle_loading_test.dart -d macos
```

For Web, generate Wasm from the parent package and serve COOP/COEP headers:

```bash
cd storyscript_bundle
flutter_rust_bridge_codegen build-web --output ../example/web
cd example
flutter run -d chrome \
  --web-header=Cross-Origin-Opener-Policy=same-origin \
  --web-header=Cross-Origin-Embedder-Policy=require-corp
```

After changing the Rust bridge, regenerate the bindings and rebuild Wasm before
restarting Flutter. Old-v1 bundles/saves are invalidated with **no migration**:
re-export/re-sign and restart incompatible progress. Saves are **not encrypted**.
If the browser still reports a content-hash mismatch, run
`flutter clean` from `storyscript_bundle/example`, restart `flutter run`, and
hard-reload the page so it fetches the new Wasm file.

## Rebuilding the game

The game fixture's private test key was generated outside the repository and
discarded. To change the source, generate a **new** Ed25519 PKCS#8 key outside
the repository, export using the CLI, verify with its public PEM, and replace
the checked-in *raw 32-byte public key in hex* together with the bundle. Never
ship a test key as a production trust root or commit a private key.

```bash
openssl genpkey -algorithm ED25519 -out /secure/new-demo-private.pem
openssl pkey -in /secure/new-demo-private.pem -pubout -out /secure/new-demo-public.pem
cargo run --manifest-path bundle/rust/Cargo.toml -- export \
  --project storyscript_bundle/example/game \
  --output storyscript_bundle/example/assets/station_nine.storybundle \
  --signing-key /secure/new-demo-private.pem
cargo run --manifest-path bundle/rust/Cargo.toml -- verify \
  storyscript_bundle/example/assets/station_nine.storybundle \
  --public-key /secure/new-demo-public.pem --json
openssl pkey -in /secure/new-demo-private.pem -pubout -outform DER | \
  python3 -c 'import sys; print(sys.stdin.buffer.read()[-32:].hex())'
```

Run commands from the repository root; put the final printed hex value in
`station_nine_public_key.txt`, then rerun the Rust bundle-runtime test and
human-owned Flutter integration test. Discard the external private test key afterward.
