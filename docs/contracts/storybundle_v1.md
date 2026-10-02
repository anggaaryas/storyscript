# StoryBundle v1 Contract

Status: normative for format version `1`.

## Security and confidentiality boundary

A `.storybundle` is a signed, deterministic delivery container. It contains no raw
`.StoryScript` source, comments, source paths, include manifests, REQUIRE blocks, or
line/column metadata. It is **not encryption or DRM**: semantic text and assets can
be recovered by anyone who can read a valid bundle.

Consumers must treat every byte and every envelope field as untrusted until the
complete verification pipeline succeeds. Strict signature verification is the
default policy. Unsigned loading is permitted only through an explicitly named
development policy and must remain visible in returned verification metadata.

## Canonical archive layout

The filename extension is `.storybundle`. The container is a ZIP archive with this
exact logical shape:

```text
META-INF/storybundle/manifest.json
META-INF/storybundle/signature.ed25519
compiled/story.pb
localization/<canonical-locale>.ftl...
assets/<normalized-logical-path>...
```

- `META-INF/storybundle/` is reserved. No other entry may use that prefix.
- Exactly one manifest and compiled payload entry exists. Production/strict bundles
  also contain exactly one signature; explicit unsigned-development fixtures omit it.
- Every non-metadata entry is listed exactly once in the signed manifest.
- Every listed entry exists exactly once. Unknown and unlisted entries are rejected.
- Directory, symlink, device, encrypted, absolute, traversing, and ambiguous
  case-folded entries are rejected.
- Raw StoryScript/source-map entries are forbidden regardless of filename case.
- Asset logical names use UTF-8, `/` separators, no empty/`.`/`..` component, and
  Unicode NFC normalization. They are sorted by normalized logical path.

Writers use fixed ZIP timestamps (`1980-01-01T00:00:00Z`), regular-file Unix mode
`0644`, no host extra fields or comments, and the ZIP `stored` method for every v1
entry. A future format version may define deterministic compression; v1 prioritizes
cross-implementation reproducibility. Readers trust neither ZIP-declared sizes nor
compression ratios.

## Manifest and signature

`manifest.json` is UTF-8 JSON serialized with fixed field order, no insignificant
whitespace, no floating-point values, and a final newline. It records:

- format version `1`;
- exact StoryScript compiler SemVer;
- lowercase SHA-256 Protobuf descriptor fingerprint;
- project ID, name, and SemVer;
- resource profile and all hard/lowered limits;
- signer key ID;
- ordered entry records containing logical name, type, compression method,
  compressed size, uncompressed size, and lowercase SHA-256 digest.

The signer key ID is `SHA-256(raw 32-byte Ed25519 public key)`, lowercase hex.
The signature is Ed25519 over:

```text
SHA-256("StoryBundle-v1\0" || exact_manifest_bytes)
```

`signature.ed25519` contains exactly 64 raw signature bytes. The host supplies a
key-ID-to-public-key trust store. No certificate PKI or remote trust service is part
of v1. A signing key path is a CLI input and is never valid in `StoryScript.toml`.

## Compiled semantic IR

The canonical schema is
`bundle/proto/storybundle/v1/compiled_story.proto`. Its descriptor fingerprint is
checked into `bundle/proto/storybundle/v1/schema.sha256`. Rust and Dart generated
types must derive from that schema and pinned generators.

The IR represents all parser AST semantic variants, including globals and exact
types, actors/portraits, logic, PREP/STORY branches and loops, choices, dialogue
forms, positions, BGM path versus STOP, statements, calls, arrays, and recursive
expressions. The otherwise parser-unreachable STORY SFX variant remains represented
defensively so future parser reachability cannot silently lose semantics.

Invariants not expressible by proto3 are mandatory:

- all required message and oneof fields are present;
- enum values are known and not `UNSPECIFIED`;
- project identifiers and names, variable/actor/scene names, jump/choice targets,
  and the start scene are non-empty;
- scene labels are unique and the start scene exists;
- recursive semantic nesting does not exceed the active resource profile;
- decimals are exact base-10 strings without exponent notation; scale is
  significant (`1.0` and `1.00` remain distinct);
- interpolated strings are ordered literal/variable segments. The lexer marker for
  source `\$` becomes literal `$`; `${...}` source spelling is not retained;
- Protobuf maps are forbidden in signed/deterministic data.
- Narration, dialogue bodies, and choice labels use `StoryText`: exactly one plain
  interpolation or message reference with a stable ID and ordered scalar argument
  types. Optional localization metadata contains the default and complete supported
  locale set, not catalog bodies. Actor names, assets, and string data stay neutral.
- Catalog entries have distinct manifest type `catalog`, reserved canonical paths,
  signed digests, and canonical semantic FTL without author comments or host paths.
  See [localization v1](storyscript_localization_v1.md) for the required profile.

## `StoryScript.toml`

```toml
[project]
id = "example.story"
name = "Example Story"
version = "1.0.0"
entry = "story/main.StoryScript"
compiler-version = "0.1.0-localization.1"

[assets]
root = "assets"
literal-policy = "required"
dynamic-files = ["portraits/hero-happy.png"]
dynamic-globs = ["backgrounds/chapter-*-*.png"]
```

The compiler pin must exactly equal the exporter parser version. Entry, asset root,
dynamic files, and globs are project-relative and may not be absolute or traverse a
parent. Literal asset discovery is mandatory in v1. Dynamic templates require one
or more explicit compatible file/glob matches; a key path is never persisted here.
Unknown configuration fields are errors.

## Resource profile

| Resource | Hard maximum |
| --- | ---: |
| Archive bytes | 100 MiB |
| Total uncompressed bytes | 100 MiB |
| One entry | 64 MiB |
| Compiled Protobuf | 16 MiB |
| Manifest | 1 MiB |
| Entries | 4,096 |
| UTF-8 logical path | 1,024 bytes |
| Semantic nesting | 128 |
| Declared locales | 64 |
| Message IDs | 100,000 |
| Canonical catalog | 16 MiB |
| Message ID UTF-8 | 256 bytes |
| Rendered event | 1 MiB |

Hosts may lower but never raise these values. Limits apply before allocation where
observable and again while reading/decompressing/decoding.

## Verification order

1. Preflight archive byte length.
2. Parse a bounded ZIP directory and validate every logical name/type/declaration.
3. Parse the bounded manifest as untrusted data.
4. Resolve signer key ID in the host trust store and verify the signature.
5. Require exact format, compiler, and schema identities.
6. Validate every entry's sizes and SHA-256 while reading under limits.
7. Decode the bounded Protobuf payload and enforce semantic invariants.
8. Validate complete canonical catalogs and their typed contracts, then expose
   verified metadata/model and archive-backed bounded asset/catalog reads.

No extraction to the filesystem occurs. Failure at any step exposes no trusted
model, player, catalog, or asset bytes. Catalogs also remain subject to the stricter
archive-wide 100 MiB and per-entry 64 MiB limits.

## Flutter bridge contract

Rust remains authoritative for archive, signature, compatibility, digest,
Protobuf, semantic, and asset-closure validation. The Flutter package receives
only verified manifest/verification metadata, normalized asset descriptors, and
one encoded `CompiledStory` payload. Dart decodes that payload with generated
types from this schema; it does not bridge or maintain a second hand-written AST.
Generic Inspector loading never eagerly transfers catalog bodies to Dart. Locale
metadata is part of the generated model; a Rust player uses a verified catalog lease.

The archive is held by an FRB-owned opaque resource. Asset reads are explicit and
bounded. Disposal is idempotent; reads after disposal return
`B_RESOURCE_DISPOSED`. If verified child players exist, parent disposal releases
only that handle and the final player lease releases the shared Rust bytes. Web accepts bytes only. Native
platforms additionally expose a path convenience API. Dart may lower limits and
preflight observable byte sizes, but Rust always enforces the final profile.

## Compatibility and rollback

v1 intentionally has no compatibility range or automatic migration. The
localization descriptor replacement invalidates prior v1 bundles and saves. Re-export,
re-sign, and deploy a matching toolchain/artifact set; restart incompatible progress.
There is no migration path. The compiler identity is `0.1.0-localization.1`.
Compiler or schema mismatch requires rebuilding with the exact exporter/loader pair. Operators
roll back to the last matching exporter, schema fingerprint, loader, and trust-store
configuration. Registry publication remains outside this contract. Runtime
execution and progress saves are a separate exact-origin contract documented in
`docs/contracts/storyplayer_save_v1.md`; save bytes are not archive entries and
are not covered by the bundle signature. Flutter package version `0.1.0`
implements exactly this v1 archive contract.
