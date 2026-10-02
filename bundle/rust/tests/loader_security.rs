mod common;

use prost::Message;
use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};
use storyscript_bundle::manifest::{COMPILED_PATH, MANIFEST_PATH};
use storyscript_bundle::proto::storybundle::v1 as pb;

use common::{TestBundle, read_entries, write_entries};

#[test]
fn localized_catalogs_are_verified_before_exposure_and_reads_are_bounded() {
    let bundle = TestBundle::localized();
    let loaded = load(
        &bundle.bytes,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .unwrap();
    assert_eq!(
        loaded
            .catalogs()
            .iter()
            .map(|c| c.locale.as_str())
            .collect::<Vec<_>>(),
        ["en", "id"]
    );
    assert!(loaded.read_catalog("id", 1 << 20).unwrap().contains("Halo"));
    assert!(loaded.read_catalog("id", 1).is_err());
    assert!(loaded.read_catalog("fr", 1 << 20).is_err());
    for mode in ["tampered", "missing", "extra", "alias"] {
        let mut entries = read_entries(&bundle.bytes);
        match mode {
            "tampered" => {
                entries
                    .iter_mut()
                    .find(|(p, _)| p == "localization/en.ftl")
                    .unwrap()
                    .1[0] ^= 1;
            }
            "missing" => entries.retain(|(p, _)| p != "localization/en.ftl"),
            "extra" => entries.push(("localization/fr.ftl".into(), b"extra = Extra\n".to_vec())),
            "alias" => entries.push(("localization/EN.ftl".into(), b"extra = Alias\n".to_vec())),
            _ => unreachable!(),
        }
        assert!(
            load(
                &write_entries(&entries),
                &bundle.trust_store(),
                VerificationPolicy::Strict,
                ResourceLimits::HARD
            )
            .is_err(),
            "{mode}"
        );
    }
    let source = loaded.read_catalog("en", 1 << 20).unwrap();
    let noncanonical = bundle.rewrite_payload(
        "localization/en.ftl",
        format!("# forbidden comment\n{source}").into_bytes(),
    );
    assert!(
        load(
            &noncanonical,
            &bundle.trust_store(),
            VerificationPolicy::Strict,
            ResourceLimits::HARD
        )
        .is_err()
    );
    let wrong_variables = bundle.rewrite_payload(
        "localization/en.ftl",
        source.replace("$name", "$ready").into_bytes(),
    );
    assert!(
        load(
            &wrong_variables,
            &bundle.trust_store(),
            VerificationPolicy::Strict,
            ResourceLimits::HARD
        )
        .is_err()
    );
}

#[test]
fn valid_signed_bundle_round_trips_model_and_bounded_asset() {
    let bundle = TestBundle::new();
    let loaded = load(
        &bundle.bytes,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect("strict load");

    assert_eq!(loaded.story().scenes.len(), 2);
    assert_eq!(loaded.assets().len(), 5);
    assert!(!loaded.verification_status().is_unsigned_development);
    let portrait = loaded
        .read_asset("portraits/hero.svg", 1024)
        .expect("bounded asset read");
    assert!(portrait.starts_with(b"<svg"));
    assert!(loaded.read_asset("portraits/hero.svg", 2).is_err());
}

#[test]
fn unsafe_duplicate_and_case_colliding_names_are_rejected() {
    let bundle = TestBundle::new();
    for unsafe_path in ["../escape", "/absolute", "assets\\alias.svg"] {
        let mut entries = read_entries(&bundle.bytes);
        entries.push((unsafe_path.to_string(), b"bad".to_vec()));
        let error = load(
            &write_entries(&entries),
            &bundle.trust_store(),
            VerificationPolicy::Strict,
            ResourceLimits::HARD,
        )
        .expect_err("unsafe path must fail");
        assert_eq!(error.code().to_string(), "B_ARCHIVE_INVALID");
    }

    let mut duplicate = read_entries(&bundle.bytes);
    let duplicate_data = duplicate
        .iter()
        .find(|(path, _)| path == "assets/audio/click.m3u8")
        .unwrap()
        .1
        .clone();
    duplicate.push(("assets/audio/other.m3u8".to_string(), duplicate_data));
    let mut duplicate_bytes = write_entries(&duplicate);
    replace_all_same_length(
        &mut duplicate_bytes,
        b"assets/audio/other.m3u8",
        b"assets/audio/click.m3u8",
    );
    let error = load(
        &duplicate_bytes,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("duplicate must fail");
    assert!(matches!(
        error.code().to_string().as_str(),
        "B_PATH_COLLISION" | "B_MANIFEST_MALFORMED"
    ));

    let mut collision = read_entries(&bundle.bytes);
    collision.push(("ASSETS/PORTRAITS/HERO.SVG".to_string(), b"alias".to_vec()));
    let error = load(
        &write_entries(&collision),
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("case collision must fail");
    assert_eq!(error.code().to_string(), "B_PATH_COLLISION");
}

fn replace_all_same_length(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let mut offset = 0;
    while let Some(index) = bytes[offset..]
        .windows(from.len())
        .position(|window| window == from)
    {
        let start = offset + index;
        bytes[start..start + from.len()].copy_from_slice(to);
        offset = start + to.len();
    }
}

#[test]
fn unknown_entries_and_malformed_manifest_fail_before_model_exposure() {
    let bundle = TestBundle::new();
    let mut entries = read_entries(&bundle.bytes);
    entries.push(("assets/unlisted.bin".to_string(), b"extra".to_vec()));
    let error = load(
        &write_entries(&entries),
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("unlisted entry must fail");
    assert_eq!(error.code().to_string(), "B_MANIFEST_MALFORMED");

    let mut source_entries = read_entries(&bundle.bytes);
    source_entries.push(("assets/raw.StoryScript".to_string(), b"source".to_vec()));
    let error = load(
        &write_entries(&source_entries),
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("raw source entry must fail");
    assert_eq!(error.code().to_string(), "B_ARCHIVE_INVALID");

    let malformed = bundle.rewrite(|path, bytes| {
        if path == MANIFEST_PATH {
            bytes[0] = b'[';
        }
        true
    });
    let error = load(
        &malformed,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("malformed manifest must fail");
    assert_eq!(error.code().to_string(), "B_MANIFEST_MALFORMED");
}

#[test]
fn signed_manifest_and_compiled_project_metadata_must_match() {
    let bundle = TestBundle::new();
    let entries = read_entries(&bundle.bytes);
    let compiled = entries
        .iter()
        .find(|(path, _)| path == COMPILED_PATH)
        .unwrap();
    let mut story = pb::CompiledStory::decode(compiled.1.as_slice()).expect("compiled story");
    story.project.as_mut().unwrap().name = "Different".to_string();
    let mismatched = bundle.rewrite_payload(COMPILED_PATH, story.encode_to_vec());
    let error = load(
        &mismatched,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("metadata mismatch must fail");
    assert_eq!(error.code().to_string(), "B_SEMANTIC_VIOLATION");
}

#[test]
fn authenticated_malformed_and_semantically_invalid_protobuf_are_rejected() {
    let bundle = TestBundle::new();
    let malformed = bundle.rewrite_payload(COMPILED_PATH, vec![0xff, 0xff, 0xff]);
    let error = load(
        &malformed,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("malformed Protobuf must fail");
    assert_eq!(error.code().to_string(), "B_PROTOBUF_DECODE");

    let invalid =
        bundle.rewrite_payload(COMPILED_PATH, pb::CompiledStory::default().encode_to_vec());
    let error = load(
        &invalid,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("invalid semantic model must fail");
    assert_eq!(error.code().to_string(), "B_SEMANTIC_VIOLATION");
}
