pub mod ast;
pub mod compiler;
pub mod diagnostic;
pub mod interpolation;
pub mod lexer;
pub mod localization;
pub mod parser;
pub mod token;
pub mod validator;

/// Exact StoryScript compiler identity embedded in compiled artifacts.
pub const COMPILER_VERSION: &str = "0.1.0-localization.1";
