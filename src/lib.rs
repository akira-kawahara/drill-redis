//! This library has been created for the purpose of evaluating Rust functionality and performance.
//! As such, it has not been fully tested.
//! 
pub mod server;
pub mod client;
pub mod protocol;
pub mod command;
pub mod db;

/// Dynamic error type.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
/// Result type that return multiple types of errors.
pub type Result<T> = std::result::Result<T, Error>;
/// Void type for channels that do not exchange messages.
enum Void {}