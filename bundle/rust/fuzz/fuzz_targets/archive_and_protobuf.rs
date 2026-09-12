#![no_main]

use libfuzzer_sys::fuzz_target;
use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{self, VerificationPolicy};
use storyscript_bundle::proto::storybundle::v1::CompiledStory;
use storyscript_bundle::trust::TrustStore;

fuzz_target!(|data: &[u8]| {
    use prost::Message;

    let _ = loader::inspect(data, ResourceLimits::HARD);
    let _ = loader::load(
        data,
        &TrustStore::new(),
        VerificationPolicy::UnsignedDevelopment,
        ResourceLimits::HARD,
    );
    if data.len() <= ResourceLimits::HARD.max_compiled_ir_bytes as usize {
        let _ = CompiledStory::decode(data);
    }
});
