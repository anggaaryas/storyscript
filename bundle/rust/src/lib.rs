pub mod archive;
pub mod assets;
pub mod config;
pub mod contract;
pub mod error;
pub mod exporter;
pub mod ir;
pub mod limits;
pub mod loader;
pub mod manifest;
pub mod project;
pub mod schema;
pub mod signing;
pub mod template;
pub mod trust;
pub mod validator;

pub mod proto {
    pub mod storybundle {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/storybundle.v1.rs"));
        }
    }
}

pub use error::{BundleError, ErrorCode, Result};

pub const FORMAT_VERSION: u32 = 1;
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMPILER_VERSION: &str = storyscript_parser::COMPILER_VERSION;
