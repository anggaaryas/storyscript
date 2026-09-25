# storyscript-bundle

Rust library and CLI for deterministic signed StoryScript bundles. Version `0.1.0`
implements the v1 source project, Protobuf IR, ZIP manifest/signature, export,
inspection, strict verification, and archive-backed asset APIs.

## Commands

```bash
cargo run --manifest-path bundle/rust/Cargo.toml -- init ./project \
  --id com.example.project --name "Example Project"
cargo run --manifest-path bundle/rust/Cargo.toml -- schema check
cargo run --manifest-path bundle/rust/Cargo.toml -- export \
  --project ./project --output /tmp/story.storybundle \
  --signing-key /secure/ed25519-private.pem
cargo run --manifest-path bundle/rust/Cargo.toml -- inspect /tmp/story.storybundle --json
cargo run --manifest-path bundle/rust/Cargo.toml -- verify /tmp/story.storybundle \
  --public-key /secure/ed25519-public.pem --json
```

`init` creates a minimal compilable project and requires a target path that does
not already exist. Project ID and display name are explicit; the command never
derives identity from a host path or overwrites existing content.

See `docs/contracts/storybundle_v1.md` for the normative contract,
`docs/feature/storybundle_export_loading.md` for flows, and
`docs/playbook/storybundle_signing_verification.md` for operations.

## Library boundaries

- `project::compile` produces deterministic semantic IR and a closed asset graph.
- `exporter::export` accepts a caller-owned `BundleSigner` and returns bytes.
- `loader::load` requires a host `TrustStore`; `VerificationPolicy::Strict` is the
  default. Unsigned development is explicit and visible in status.
- `LoadedBundle::read_asset` performs bounded archive-backed reads.

Hard limits are 100 MiB archive/uncompressed total, 64 MiB per asset, 16 MiB IR,
1 MiB manifest, 4,096 entries, 1,024-byte paths, and depth 128. Callers can lower
but not raise them.

Bundles contain no raw source but are not encrypted and provide no DRM or content
confidentiality. Private keys must remain outside the project/repository. Tests use
ephemeral keys only.

Flutter packaging and the verified headless player bridge live in
`storyscript_bundle/`; the archive crate remains authoritative for verification
and exposes only verified `LoadedBundle` capabilities to that runtime. Progress
saves use the separate contract in `docs/contracts/storyplayer_save_v1.md` and
are not entries in, or authenticated by, the StoryBundle archive.
