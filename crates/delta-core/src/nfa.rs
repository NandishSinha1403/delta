//! Nondeterministic finite automaton: Definition 1.37.
//!
//! > A nondeterministic finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where ...
//! > δ : Q × Σ_ε → P(Q) is the transition function ...
//!
//! δ is stored as per-state lists of outgoing edges; δ(q, a) is the set of their targets.

use crate::alphabet::Alphabet;
use crate::bitset::BitSet;
use crate::ids::{StateId, SymId};
use crate::state::StateInfo;

/// An arrow label: a symbol of Σ, or ε (Σ_ε = Σ ∪ {ε}).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Label {
    Eps,
    Sym(SymId),
}

/// N = (Q, Σ, δ, q₀, F).
#[derive(Debug, Clone, PartialEq)]
pub struct Nfa {
    pub states: Vec<StateInfo>,
    pub sigma: Alphabet,
    /// delta[q] = every (label, target) arrow leaving q.
    pub delta: Vec<Vec<(Label, StateId)>>,
    pub start: StateId,
    pub accept: BitSet,
    /// Layout hint only (not part of the tuple): the states built for each sub-expression, when
    /// the NFA came from a regular expression (Lemma 1.55). Drawn as nested boxes.
    pub groups: Vec<StateGroup>,
}

/// The states one sub-expression's construction produced, e.g. all states of `(0∪1)*`.
#[derive(Debug, Clone, PartialEq)]
pub struct StateGroup {
    /// The sub-expression, printed with minimal parentheses.
    pub label: String,
    pub states: Vec<StateId>,
    /// The enclosing group (groups nest like the expression).
    pub parent: Option<usize>,
}

impl Nfa {
    /// An NFA with no states yet; add them with `add_state`.
    pub fn empty(sigma: Alphabet) -> Nfa {
        Nfa {
            states: Vec::new(),
            sigma,
            delta: Vec::new(),
            start: StateId(0),
            accept: BitSet::new(),
            groups: Vec::new(),
        }
    }

    /// Adds a state and returns its id.
    pub fn add_state(&mut self, info: StateInfo) -> StateId {
        let id = StateId::from_index(self.states.len());
        self.states.push(info);
        self.delta.push(Vec::new());
        id
    }

    /// Adds the arrow `from -label-> to` (δ returns a set, so duplicates are skipped).
    pub fn add_edge(&mut self, from: StateId, label: Label, to: StateId) {
        if let Some(edges) = self.delta.get_mut(from.index())
            && !edges.contains(&(label, to))
        {
            edges.push((label, to));
        }
    }

    /// |Q|
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// The name of a state.
    pub fn name(&self, q: StateId) -> &str {
        self.states.get(q.index()).map_or("?", |s| s.name.as_str())
    }

    /// The arrows leaving q.
    pub fn edges(&self, q: StateId) -> &[(Label, StateId)] {
        self.delta.get(q.index()).map_or(&[], Vec::as_slice)
    }

    /// E(R): every state reachable from R along zero or more ε-arrows, R included (p. 56).
    pub fn eps_closure(&self, set: &BitSet) -> BitSet {
        let mut seen = set.clone();
        let mut stack = set.to_vec();
        // `while let Some(x) = stack.pop()` loops until the Vec is empty, taking the last item each time.
        while let Some(q) = stack.pop() {
            for &(label, r) in self.edges(StateId::from_index(q)) {
                if label == Label::Eps && seen.insert(r.index()) {
                    stack.push(r.index());
                }
            }
        }
        seen
    }

    /// ⋃_{r ∈ R} δ(r, a), without ε-closure.
    pub fn step(&self, set: &BitSet, a: SymId) -> BitSet {
        let mut out = BitSet::new();
        for q in set.to_vec() {
            for &(label, r) in self.edges(StateId::from_index(q)) {
                if label == Label::Sym(a) {
                    out.insert(r.index());
                }
            }
        }
        out
    }

    /// Does R contain an accept state?
    pub fn any_accept(&self, set: &BitSet) -> bool {
        set.intersects(&self.accept)
    }

    /// Is this NFA actually a DFA: no ε-arrows and exactly one arrow per (q, a)? (I304)
    pub fn is_deterministic_and_total(&self) -> bool {
        self.delta.iter().all(|edges| {
            !edges.iter().any(|(l, _)| *l == Label::Eps)
                && self
                    .sigma
                    .ids()
                    .all(|a| edges.iter().filter(|(l, _)| *l == Label::Sym(a)).count() == 1)
        })
    }
}
