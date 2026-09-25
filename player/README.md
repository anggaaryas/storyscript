# StoryScript Rust Player

`storyscript-player` owns the UI-independent execution model used by source and
verified StoryBundle stories. `adapters::ast` and feature-gated
`adapters::bundle` convert into the same immutable model; the engine implements
INIT, PREP, STORY, logic, collections, choices, jumps, ordered media effects,
transactional limits, session-local ChaCha RNG, and bounded history once.

## APIs

- `StoryPlayer::from_source` / `from_file`: legacy `StepResult` API used by the
  TUI. It retains scene headers, selected-choice markers, and formatted errors.
- `SemanticPlayer`: compact semantic deltas, explicit history pages, source or
  verified-bundle construction, bounded save export, and restore-to-new-session.
- `storybundle-runtime` Cargo feature: accepts only a verified
  `storyscript_bundle::loader::LoadedBundle`; caller-asserted Protobuf is not a
  trust capability.

Saves are permitted at stable boundaries after open, advance, choice, or
restore. Restore does not rerun current PREP or re-flatten current STORY. Saves
require exact runtime/origin/semantic identity and have no migration. They are
validated but not encrypted, authenticated, or anti-cheat.

## Limits

The v1 hard profile is 100,000 operations, logic depth 128, 16,384 pending
events and array elements, 1 MiB rendered text/event, 10,000/8 MiB retained
history, and 16 MiB save bytes. Hosts may lower but never raise these values.
Failed interactions and restores do not publish partial state.

## Tests

```sh
cargo test --manifest-path player/Cargo.toml --no-default-features
cargo test --manifest-path player/Cargo.toml --no-default-features --features storybundle-runtime
cargo test --manifest-path player/Cargo.toml --test cli_help
```

See `docs/contracts/storyplayer_save_v1.md` and
`docs/feature/headless_story_player.md`.
