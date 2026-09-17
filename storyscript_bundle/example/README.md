# StoryBundle Inspector

Single-route example for the `storyscript_bundle` package. It strictly loads the
bundled signed fixture using its public test key, then shows verification status,
exact compiler/schema metadata, semantic model counts/tree, normalized assets,
and bounded SVG/PNG previews. It does not execute StoryScript or play media.

```bash
flutter run -d macos
```

For Web, generate Wasm from the parent package and serve COOP/COEP headers:

```bash
cd ..
flutter_rust_bridge_codegen build-web --output ../example/web
cd example
flutter run -d chrome \
  --web-header=Cross-Origin-Opener-Policy=same-origin \
  --web-header=Cross-Origin-Embedder-Policy=require-corp
```

The fixture private key was generated outside the repository and discarded. Only
the raw public test key is committed. Tests mutate fixture bytes at runtime for
tamper cases.
