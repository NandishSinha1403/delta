//! Generalized NFA: Definition 1.64, always kept in the special form of p. 70.
//!
//! > A generalized nondeterministic finite automaton is a 5-tuple (Q, Σ, δ, q_start, q_accept),
//! > where δ : (Q − {q_accept}) × (Q − {q_start}) → R is the transition function.
//!
//! The special form is complete, so δ is a dense |Q| × |Q| matrix of regex ids; ∅ means "no arrow".
//! Entries outside δ's domain (into q_start, out of q_accept) are always ∅.

use crate::alphabet::Alphabet;
use crate::ids::StateId;
use crate::regex::{ReArena, ReId};
use crate::state::StateInfo;

/// G = (Q, Σ, δ, q_start, q_accept).
#[derive(Debug, Clone)]
pub struct Gnfa {
    pub states: Vec<StateInfo>,
    pub sigma: Alphabet,
    pub start: StateId,
    pub accept: StateId,
    /// label[i · |Q| + j] = δ(qᵢ, qⱼ).
    pub label: Vec<ReId>,
    /// Where the labels' expressions live.
    pub arena: ReArena,
}

impl Gnfa {
    /// |Q|
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// δ(qᵢ, qⱼ) (∅ outside the domain).
    pub fn label(&self, i: StateId, j: StateId) -> ReId {
        let n = self.num_states();
        self.label
            .get(i.index() * n + j.index())
            .copied()
            .unwrap_or(self.arena.empty())
    }

    /// Sets δ(qᵢ, qⱼ). Ignored outside δ's domain, keeping the special form.
    pub fn set_label(&mut self, i: StateId, j: StateId, r: ReId) {
        if i == self.accept || j == self.start {
            return;
        }
        let n = self.num_states();
        if let Some(slot) = self.label.get_mut(i.index() * n + j.index()) {
            *slot = r;
        }
    }

    /// The name of a state.
    pub fn name(&self, q: StateId) -> &str {
        self.states.get(q.index()).map_or("?", |s| s.name.as_str())
    }

    /// The label printed with minimal parentheses.
    pub fn label_text(&self, i: StateId, j: StateId) -> String {
        self.arena.print(self.label(i, j), &self.sigma)
    }
}
