use std::path::PathBuf;

fn main() {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir")).join("proto");
    let schema = root.join("storyplayer/v1/player_save.proto");
    let descriptor = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"))
        .join("storyplayer_descriptor.bin");
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path().expect("vendored protoc"));
    config.file_descriptor_set_path(descriptor);
    config
        .compile_protos(&[schema], &[root])
        .expect("compile StoryPlayer save schema");
    println!("cargo:rerun-if-changed=proto/storyplayer/v1/player_save.proto");
    println!("cargo:rerun-if-changed=proto/storyplayer/v1/schema.sha256");
}
