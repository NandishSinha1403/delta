//! States and their provenance.
//!
//! Every state created by a conversion remembers why it exists (docs/plan/01-THEORY.md §0), so the
//! UI can explain it: "this DFA state is E({1}) = {1,3}, the start state (Theorem 1.39)".

use crate::bitset::BitSet;

/// An element of Q: a display name plus where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct StateInfo {
    pub name: String,
    pub origin: Origin,
}

/// Why a state exists.
// An `enum` is a type whose value is exactly one of several variants; variants can carry data
// (like a TypeScript discriminated union).
#[derive(Debug, Clone, PartialEq)]
pub enum Origin {
    /// Written by the user.
    User,
    /// A subset of the NFA's states (Theorem 1.39). The set holds the NFA's state indices.
    Subset(BitSet),
    /// The trap state added to complete a partial DFA.
    Trap,
    /// A state introduced by a construction; the text names the rule, e.g. "Theorem 1.45".
    // `&'static str` is text baked into the program, valid for its whole run ("static lifetime").
    Rule(&'static str),
    /// A state carried over from another machine (keeps that machine's name).
    Copied,
}

impl StateInfo {
    /// A state the user wrote.
    pub fn user(name: impl Into<String>) -> Self {
        // `.into()` converts using the `Into` trait (here: &str or String into String).
        StateInfo {
            name: name.into(),
            origin: Origin::User,
        }
    }

    /// A state created by a construction.
    pub fn rule(name: impl Into<String>, rule: &'static str) -> Self {
        StateInfo {
            name: name.into(),
            origin: Origin::Rule(rule),
        }
    }
}
