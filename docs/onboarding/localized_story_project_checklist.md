# Localized Story Project Checklist

Audience: authors and integrators shipping rewritten-v1 Rust/Dart localization.

## Authoring and translation

- [ ] Pin compiler `0.1.0-localization.1`; use `init --localized` or add the
  localization section manually. Minimal initialization stays non-localized.
- [ ] Configure canonical default/supported locales and a sandboxed root.
- [ ] Add one `<locale>.ftl` for every configured locale, including default.
- [ ] Use stable unique `@"message-id"` declarations only for narration/dialogue/
  choices; never localize actor names, assets, identifiers or stored state strings.
- [ ] Run `localize extract --json`; inspect include-relative source spans.
- [ ] Run `localize sync --json`; resolve reported conflicts/obsolete IDs manually,
  translate stubs, and preserve existing translator comments/content.
- [ ] Check every locale's transitive argument-name set and site visibility/types.
- [ ] Reject arrays; reserve `NUMBER` for integers/decimals. Test boolean selectors.
- [ ] Confirm number values are exactly binary-representable (`0.5` is safe;
  `0.1`, `2^53 + 1` and `i64::MAX` are unsafe). Keep saved decimal scale exact.
- [ ] Check terms/selects/plurals; use literal named term parameters, not attributes,
  custom functions or datetime formatting.
- [ ] Run non-mutating `localize check --json` as the structural release gate.
  Independently review translation quality; stubs can be structurally valid.

## Export and trust

- [ ] Run both schema contract tests and `schema check`.
- [ ] Export/sign with an external key; strictly verify with authenticated trust.
- [ ] Repeat export with identical inputs/key and compare bytes.
- [ ] Confirm canonical catalog entries are signed, bounded, comment/path-free,
  and correspond exactly to supported locales.
- [ ] Confirm tamper/missing/extra/alias/oversize failures expose no model/player.
- [ ] Preserve separate Inspector/game public-key identities; discard ephemeral
  fixture private keys. Re-export all prior-v1 fixtures; there is no migration.

## Rust host integration

- [ ] Use `SemanticPlayer::from_project` / `restore_project` for catalog-aware
  source playback, not raw source/path APIs that display unresolved IDs.
- [ ] For bundles, use verified `_with_locales` constructors with the
  `storybundle-runtime` feature; never accept caller-asserted translation resources.
- [ ] Pass ordered preferences; surface requested versus resolved locale/fallback.
- [ ] Treat session locale as immutable; save/restore a candidate to change it.
- [ ] Render text as opaque plain text, preserving Fluent isolation marks.
- [ ] Keep story copy in FTL and application-shell copy in host localization.
- [ ] Verify current, pending, choice and history cross-locale rerendering from
  exact snapshots, including repeated sites and no PREP/RNG/media replay.
- [ ] Handle structured number/resolver/limit errors without losing the checkpoint.
- [ ] Store saves opaquely with host-owned confidentiality, retention and deletion.

## Dart host integration

- [ ] Initialize the package's `RustLib` and rebuild matching native/Web bridge
  artifacts after generation (source FRB 2.12.0; bundle FRB 2.13.0).
- [ ] For filesystem projects, use `SourceStoryPlayerLoader.openProject` /
  `restoreProject` with `StoryPlayerLocalePreferences`, not raw source/path methods.
- [ ] For bundles, pass `StoryBundlePlayerLocalePreferences` as named `locales`
  to bytes/path open/restore or existing-bundle `fromBundle`/`restoreFromBundle`.
- [ ] Read immutable `player.locale` / `resolvedLocale`; distinguish null plain
  sessions from raw keyed `hasUnresolvedLocalization` rather than claiming translation.
- [ ] Keep message arguments and FTL out of Dart. Preserve rendered isolation marks
  and compact event/history APIs; treat save bytes as opaque confidential copies.
- [ ] If injecting bundle bindings, implement `LocaleAwareStoryBundlePlayerBindings`
  for explicit preferences. Legacy adapters only accept default preferences.
- [ ] Verify stale-load disposal, parent-first/child-first leases, structured
  errors, and candidate-before-swap locale changes in focused non-UI tests.

## Remaining acceptance

- [ ] Validate workspace editor tools when phase 8 lands; current grammar is lexical.
- [ ] Human-owned localized Flutter widget/golden/integration/accessibility/platform
  acceptance follows phase 9. Do not claim the current Station Nine UI is localized.
- [ ] Follow [recovery](../playbook/localization_release_recovery.md) and
  [QA](../qa-docs/storyscript_localization_v1_qa.md) before release.
