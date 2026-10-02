# StoryPlayer Progress Save v1 Contract

Status: rewritten v1 locale-neutral execution/save and both locale-aware bridges implemented.
Bridges preserve rendered strings; locale selection happens only at open/restore.
Audience: runtime and bridge implementers. The wire authority is
`player/proto/storyplayer/v1/player_save.proto`; the pinned descriptor hash in
`player/proto/storyplayer/v1/schema.sha256` is
`f04474542bd55e32eb2731d8a43b361e7549fd6dea49ba9455590aed7afa9b5d`.
This is a separate contract from the signed StoryBundle v1 archive; save bytes
are **not encrypted**, signed, authenticated, or an anti-cheat mechanism.

## Identity and compatibility

Saves have schema version `1` and runtime version `1`. Unknown versions fail
closed without migration. `Origin.kind` is required: source identifies parser,
compiler, runtime identity, and the shared-runtime semantic SHA-256; bundle also
identifies archive format, compiler, compiled-schema SHA-256, project ID/version,
verified compiled-entry SHA-256, and either the signer key ID or an explicitly
true unsigned-development marker. Bundle and source saves are never
interchangeable even if their semantics match. An exact story/origin mismatch
is an incompatibility, not corrupt data. Source locations, formatting, and
comments are excluded from the semantic fingerprint; executable semantics are
included. The fingerprint is SHA-256 over the ordered execution-owned model's
v1 canonical debug representation. `SourceSpan` omits coordinates from that
representation, which is pinned by the exact runtime identity.
The replacement runtime identity is `storyscript-player/0.1.0:model-v1-localization`.
Prior v1 bundles and saves are deliberately incompatible: no migration. Re-export
bundles with the matching compiler and restart old progress.

## Runtime implementation

The shared source/bundle engine seeds a session-local ChaCha20 generator before
INIT, and uses it during PREP and subsequent story execution. `SessionRng`
captures the v1 algorithm, 32-byte seed, stream and 128-bit word position and
can reconstruct the same next random outcome. The 64-bit block counter permits
word positions below 2^68 only; larger positions are rejected (the generator
would otherwise silently discard the upper bits). This is reproducibility, **not**
cryptographic/security guarantee. Static execution-model indexes are built
without running INIT or entering a scene. The engine's semantic step returns
scene transitions, ordered PREP effects, STORY SFX, resolved portrait paths and
structured errors. Candidate-based interactions provide transactional limits
and bounded rolling history. Source and verified StoryBundle players export and
restore this contract through UI-free Rust/Dart APIs; the legacy source/TUI
adapter remains compatible.

## Stable boundaries and state

A save describes a quiescent public boundary after open, advance, choice, or
completed restore. Ordered `globals` and `locals` carry declared types and
values; `current_scene`, `background`, `bgm`, current semantic event, ordered
pending resolved events (including internal jumps and end), ordered effects,
session RNG seed/stream/128-bit word position, retained history, absolute
sequence metadata, and active/finished/faulted status describe continuation.
Restoring the current scene must not replay its PREP or re-flatten its STORY;
entering a later scene still executes PREP normally. No source, compiled story,
instructions, archive, asset bytes, or host storage reference is embedded.

The semantic event variants are scene transition, narration, dialogue with
actor/emotion/position/resolved portrait, choices, ordered media (background,
BGM path/stop, SFX), end, and structured error. Internal jump events may occur
only in the pending queue. Compact deltas carry current event/effects,
scene/status, sequence, and retained-history truncation metadata; history is
read separately in bounded pages. Legacy TUI scene headers, selected-choice
markers, and narration-form errors are *formatting*, not semantic events.

Narration, dialogue bodies, and choices use an explicit `StoryText` union: plain
resolved text or a locale-neutral `MessageSnapshot` with ID and strictly name-sorted
exact scalar arguments (name, type, value). Keyed saves contain neither rendered
text nor selected locale. Current, pending, and history references use the same
representation. Argument values are captured at event flattening, not recomputed
from later variables. Restore validates IDs/contracts/values against the loaded
story and rerenders in the newly negotiated immutable locale without replaying
PREP/STORY. Arrays are forbidden as message arguments. Numeric arguments must pass
the exact `f64` round-trip guard before Fluent formatting. See
[localization v1](storyscript_localization_v1.md).

Integers are exact `i64`; decimals carry an exact base-10 mantissa string and
scale (trailing zeros are meaningful). Arrays contain scalars of exactly one
declared type and no nested arrays. Ordered repeated records (never maps)
preserve event/effect/history and canonical variable ordering.

## Resource ceilings and validation contract

`contract::HARD_LIMITS` is the v1 hard profile: 100,000 execution operations per
interaction; logic depth 128; 16,384 pending events per scene and elements per
array; 1 MiB rendered text/event; 10,000 history entries and 8 MiB retained
history; 16 MiB encoded/imported save. Hosts may lower each nonzero ceiling,
never raise it. History evicts oldest entries under entry/byte pressure,
tracking absolute first-retained sequence and omitted count; save-time size
trimming may discard only oldest history, never live variables or pending events.
If non-history state alone exceeds the save cap, export must fail atomically.

The wire decoder preflights byte size, requires canonical encoding, checks
required fields and known enum/oneof variants (including explicit truth for
`end`, `bgm_stop`, and `unsigned_development`), rejects duplicate
variable/history records, and validates basic types, sequence, status, and RNG
shape. The restore codec additionally rejects variables unknown to the loaded
story, invalid targets or media paths, and impossible scene/local state.
Invalid Protobuf or impossible state is `R_SAVE_STATE_CORRUPT`; unsupported
versions or exact origin/story mismatch are distinct incompatibilities.
Runtime-budget failures must leave the prior stable checkpoint unchanged and
report scene, code, and resource/actual/limit when relevant. Restore builds a
new candidate session; failed restore must not mutate an existing session.

Host applications own UI, persistence, backups, deletion, and authorization.
Do not log raw saves: progress variables may contain personal data.
