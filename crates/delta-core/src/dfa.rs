//! Deterministic finite automaton: Definition 1.5.
//!
//! > A finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where Q is a finite set called the states,
//! > Σ is a finite set called the alphabet, δ : Q × Σ → Q is the transition function, q₀ ∈ Q is the
//! > start state, and F ⊆ Q is the set of accept states.
//!
//! δ is total by construction: a dense |Q| × |Σ| table, so simulating is one array index per symbol.
//! (Partial DFAs are completed with a trap state by the DSL before a `Dfa` is built.)

use crate::alphabet::Alphabet;
use crate::bitset::BitSet;
use crate::budget::TlError;
use crate::ids::{StateId, SymId};
use crate::state::StateInfo;

/// M = (Q, Σ, δ, q₀, F).
#[derive(Debug, Clone, PartialEq)]
pub struct Dfa {
    /// Q
    pub states: Vec<StateInfo>,
    /// Σ
    pub sigma: Alphabet,
    /// δ as a row-major table: δ(q, a) = delta[q · |Σ| + a].
    pub delta: Vec<StateId>,
    /// q₀
    pub start: StateId,
    /// F
    pub accept: BitSet,
}

impl Dfa {
    /// Builds a DFA and checks that the table really is a total function Q × Σ → Q.
    pub fn new(
        states: Vec<StateInfo>,
        sigma: Alphabet,
        delta: Vec<StateId>,
        start: StateId,
        accept: BitSet,
    ) -> Result<Dfa, TlError> {
        let n = states.len();
        if n == 0 {
            return Err(TlError::Invalid(
                "Q must contain at least the start state".into(),
            ));
        }
        if delta.len() != n * sigma.len() {
            return Err(TlError::Internal(format!(
                "δ table has {} entries, expected |Q|·|Σ| = {}",
                delta.len(),
                n * sigma.len()
            )));
        }
        if delta.iter().any(|q| q.index() >= n) || start.index() >= n {
            return Err(TlError::Internal(
                "δ or q₀ refers to a state outside Q".into(),
            ));
        }
        // `if let Some(x) = opt && cond` (a "let chain") runs only if opt has a value AND cond holds.
        if let Some(&max) = accept.to_vec().last()
            && max >= n
        {
            return Err(TlError::Internal("F is not a subset of Q".into()));
        }
        Ok(Dfa {
            states,
            sigma,
            delta,
            start,
            accept,
        })
    }

    /// |Q|
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// δ(q, a). Never fails for ids that belong to this machine.
    pub fn next(&self, q: StateId, a: SymId) -> StateId {
        let i = q.index() * self.sigma.len() + a.index();
        // `.unwrap_or(q)` would hide bugs elsewhere, but ids always come from this machine.
        self.delta.get(i).copied().unwrap_or(q)
    }

    /// Is q ∈ F?
    pub fn is_accept(&self, q: StateId) -> bool {
        self.accept.contains(q.index())
    }

    /// The name of a state.
    pub fn name(&self, q: StateId) -> &str {
        self.states.get(q.index()).map_or("?", |s| s.name.as_str())
    }
}
