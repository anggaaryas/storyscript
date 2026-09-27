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
route contains at least 1,500 words. Different reactor/archive/dawn backgrounds
and Dot portraits are read from the verified archive on demand.

The toolbar opens the **separate, original Bundle Inspector**, which still
loads `assets/demo.storybundle` using `assets/demo_public_key.txt`. That fixture
is also used by Rust regression tests and has not been replaced. The inspector
shows trust status, metadata, model tree, and image previews; it does not play
the chapter. No save slots, audio playback, or external content loading are
implemented in this example.

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
restarting Flutter. If the browser still reports a content-hash mismatch, run
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
Flutter integration test. Discard the external private test key afterward.
