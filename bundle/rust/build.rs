use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let proto_root = manifest_dir.join("../proto");
    let proto = proto_root.join("storybundle/v1/compiled_story.proto");
    let descriptor = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"))
        .join("storybundle_descriptor.bin");
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");

    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc);
    config.file_descriptor_set_path(descriptor);
    config
        .compile_protos(&[proto], &[proto_root])
        .expect("compile StoryBundle protobuf schema");

    println!("cargo:rerun-if-changed=../proto/storybundle/v1/compiled_story.proto");
    println!("cargo:rerun-if-changed=../proto/storybundle/v1/schema.sha256");
}
