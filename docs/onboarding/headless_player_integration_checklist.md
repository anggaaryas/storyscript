# Headless Player Integration Checklist

Audience: developers adding source or verified-bundle playback to an app.

## Localization core

- [ ] Follow [localized project checklist](localized_story_project_checklist.md).
- [ ] Use Rust project open/restore with ordered requested preferences; raw source/path
  APIs deliberately display IDs and report no resolved locale.
- [ ] Use verified Rust bundle `_with_locales` constructors and expose resolved locale.
- [ ] Keep locale immutable; export a locale-neutral save and restore a candidate to
  change language, swapping only after success and without PREP/STORY/media replay.
- [ ] Preserve current/pending/history snapshots, exact-number guards and Fluent isolation.
- [ ] Keep application-shell localization in the host, not StoryScript catalogs.
- [ ] Treat old-v1 bundle/save rejection as re-export/restart with no migration.
- [ ] Do not claim locale-aware Dart source/bundle APIs before phases 6–7 ship.

## Initialization and trust

- [ ] Initialize the package's `RustLib` once before player calls.
- [ ] For bundles, provision authenticated raw Ed25519 public keys in a
  `StoryBundleTrustStore`; never trust a key embedded in the archive.
- [ ] Use strict `StoryBundlePlayerLoader` in production. Keep
  `unsignedDevelopment` conspicuous and local-only.
- [ ] Prefer fused bundle bytes/path open or restore when Dart does not need the
  inspector model; use `fromBundle` only for a verified inspector-plus-player flow.
- [ ] Use the original source text/path for source restore.

## Runtime contract

- [ ] Lower `StoryPlayerLimits`/`StoryBundlePlayerLimits` for the host profile;
  no field may be zero or exceed the v1 hard maximum.
- [ ] Render every semantic event and apply ordered effects exactly once.
- [ ] Treat BGM stop as distinct from a missing effect and STORY SFX as a
  one-shot media event.
- [ ] Disable advance while choices are active; send only an available index.
- [ ] Page history explicitly and explain `omittedHistoryCount` and
  `firstRetainedSequence` when the rolling cap truncates old entries.

## Save ownership

- [ ] Store returned bytes opaquely; do not parse or edit them in Dart.
- [ ] Record the app/story release needed for exact-match restore.
- [ ] Encrypt/authenticate saves in host storage if confidentiality or tamper
  resistance is required; SDK validation alone provides neither.
- [ ] Define backup, retention, deletion, cloud/AuthZ, and PII policies.
- [ ] On corrupt, wrong-origin, incompatible, or oversized errors, leave the
  active player untouched and offer a matching-version recovery path.

## Lifecycle and acceptance

- [ ] Dispose players idempotently. A child bundle player may outlive the parent
  `LoadedStoryBundle`; dispose every child to release the final lease.
- [ ] Reject stale asynchronous completions when navigation/load generation changes.
- [ ] Verify restored current event/history and next random result without
  duplicate PREP effects or STORY output.
- [ ] Run the non-UI Rust/Dart suites. Native/Web Flutter integration, widgets,
  accessibility, platform builds, and host storage UX remain human-owned.
