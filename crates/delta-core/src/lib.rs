//! Delta's engine: automata models and the book's algorithms, in plain Rust.
//!
//! This crate has no WebAssembly dependencies, so the same code runs in the browser (through
//! `delta-wasm`), in the native CLI and in tests. Every model and algorithm cites its numbered
//! Definition / Theorem / Lemma in Sipser, *Introduction to the Theory of Computation*, 3rd ed.

// `//!` comments document the enclosing item (here: the whole crate); `///` documents the next item.

// `pub mod x;` declares a module: Rust loads it from `src/x.rs` (or `src/x/mod.rs`).
pub mod alphabet;
pub mod bitset;
pub mod budget;
pub mod convert;
pub mod dfa;
pub mod gnfa;
pub mod ids;
pub mod nfa;
pub mod object;
pub mod oracle;
pub mod regex;
pub mod simulate;
pub mod state;
pub mod trace;
pub mod view;
