//! Locates Room's modules for builds that need them in the consuming crate.
//!
//! Use this package as a build dependency pinned to a Git revision. Keeping
//! the engine modules in one crate lets the runtime isolate Room's static state.
//! This package does not compile the engine or its desktop platform.

/// Absolute repository path for resolving the engine modules in `room/src`.
///
/// Asset files may contain Git LFS pointers. Validate them separately before
/// starting the engine.
pub const SOURCE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

#[cfg(test)]
mod tests;
