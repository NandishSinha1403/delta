//! Delta's engine: automata models and the book's algorithms, in plain Rust.
//!
//! This crate has no WebAssembly dependencies, so the same code runs in the browser (through
//! `delta-wasm`), in the native CLI and in tests.

// `//!` comments document the enclosing item (here: the whole crate); `///` documents the next item.

// `pub mod hello;` declares a module: Rust loads it from `src/hello.rs` and `pub` makes it public.
pub mod hello;
