//! The Delta language (`.delta` files): lexer, parser, checker and evaluator.
//!
//! The grammar is specified in `docs/plan/04-DSL.md` and the diagnostics in `docs/plan/08-ERRORS.md`.
//! The entry points are [`evaluate`] (whole program → `Document`) and [`run_input`].

pub mod ast;
pub mod book;
pub mod build;
pub mod diag;
pub mod document;
pub mod eval;
pub mod lexer;
pub mod parser;

// `pub use` re-exports a name, so callers can write `delta_dsl::evaluate` instead of the full path.
pub use eval::{evaluate, run_input};

#[cfg(test)]
mod tests;
