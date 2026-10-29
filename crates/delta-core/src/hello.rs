//! Phase 0 "hello": a hard-coded DFA, returned as its formal 5-tuple.
//!
//! The machine is M1 from Figure 1.6 of Sipser (3rd ed.): strings with at least one 1 and an
//! even number of 0s after the last 1. Its tuple follows Definition 1.5, (Q, Σ, δ, q₀, F).
//!
//! This is a *display* shape that only proves the Rust → WASM → Worker → React pipeline. The real
//! `Dfa` type (dense δ-table, `StateId`s) arrives in Phase 1.

// `use` brings names from other crates/modules into scope, like `import` in TypeScript.
use serde::Serialize;
use ts_rs::TS;

/// One entry of δ : Q × Σ → Q, written δ(from, symbol) = to.
// `#[derive(...)]` asks the compiler to generate code for us: `Debug` (printing with {:?}),
// `Clone` (.clone()), `PartialEq` (==), `Serialize` (to JSON), `TS` (a TypeScript type).
// `#[ts(export)]` writes that TypeScript type to web/src/engine/generated/ when `cargo test` runs.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
// `pub struct` declares a public record type with named fields (like a TS interface, but it is data).
pub struct Transition {
    // `String` is an owned, growable piece of UTF-8 text (as opposed to a borrowed `&str`).
    pub from: String,
    pub symbol: String,
    pub to: String,
}

/// A DFA as the formal 5-tuple of Definition 1.5: (Q, Σ, δ, q₀, F).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DfaTuple {
    /// The machine's name, e.g. "M1".
    pub name: String,
    /// Q, the finite set of states.
    // `Vec<T>` is a growable list of T values (like a TS array).
    pub states: Vec<String>,
    /// Σ, the input alphabet.
    pub sigma: Vec<String>,
    /// δ, the transition function, one entry per (state, symbol) pair, so it is total.
    pub delta: Vec<Transition>,
    /// q₀ ∈ Q, the start state.
    pub start: String,
    /// F ⊆ Q, the accept states.
    pub accept: Vec<String>,
}

/// Builds the tuple of M1 (Sipser, Figure 1.6).
// `fn name() -> Type` declares a function; the last expression without `;` is the return value.
pub fn hello_dfa() -> DfaTuple {
    // A closure `|a, b| expr` is an inline anonymous function (like `(a, b) => expr` in TS).
    // `&str` is a borrowed view of some text; `.to_string()` copies it into an owned `String`.
    let t = |from: &str, symbol: &str, to: &str| Transition {
        from: from.to_string(),
        symbol: symbol.to_string(),
        to: to.to_string(),
    };

    DfaTuple {
        name: "M1".to_string(),
        // `vec![...]` is a macro (the `!` marks macros) that builds a Vec from a list.
        states: vec!["q1".to_string(), "q2".to_string(), "q3".to_string()],
        sigma: vec!["0".to_string(), "1".to_string()],
        delta: vec![
            t("q1", "0", "q1"),
            t("q1", "1", "q2"),
            t("q2", "0", "q3"),
            t("q2", "1", "q2"),
            t("q3", "0", "q2"),
            t("q3", "1", "q2"),
        ],
        start: "q1".to_string(),
        accept: vec!["q2".to_string()],
    }
}

/// The tuple of M1 as a JSON string: the shape that crosses the WASM boundary.
///
/// Errors are values, never panics (docs/plan/08-ERRORS.md §1).
// `Result<T, E>` is either `Ok(value)` or `Err(error)`: Rust's way of returning errors.
// `serde_json::to_string` returns a Result, and we hand it straight back to the caller.
pub fn hello_json() -> Result<String, serde_json::Error> {
    serde_json::to_string(&hello_dfa())
    // `&value` passes a borrowed reference: the function may read `value` but does not take it.
}

// `#[cfg(test)]` compiles this module only for `cargo test`, never into the real library.
#[cfg(test)]
mod tests {
    // `super::*` imports everything from the parent module (the code above).
    use super::*;

    // `#[test]` marks a function that `cargo test` runs; a panic (failed assert) fails the test.
    #[test]
    fn m1_delta_is_total() {
        let m = hello_dfa();
        // `.len()` is the number of elements; `assert_eq!` panics with both values if they differ.
        assert_eq!(m.delta.len(), m.states.len() * m.sigma.len());
        // `for x in &list` loops over borrowed elements without consuming the list.
        for q in &m.states {
            for a in &m.sigma {
                // `.iter()` walks the elements; `.filter(...)` keeps matches; `.count()` counts them.
                let n = m
                    .delta
                    .iter()
                    .filter(|tr| &tr.from == q && &tr.symbol == a)
                    .count();
                assert_eq!(n, 1, "δ({q}, {a}) must be defined exactly once");
            }
        }
    }

    #[test]
    fn m1_serializes_to_the_expected_json() {
        let json = hello_json().unwrap();
        assert!(json.starts_with(r#"{"name":"M1","states":["q1","q2","q3"],"sigma":["0","1"]"#));
        assert!(json.ends_with(r#""start":"q1","accept":["q2"]}"#));
    }
}
