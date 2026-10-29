//! C1 — DFA → NFA (p. 47: "every deterministic finite automaton is automatically a
//! nondeterministic finite automaton").
//!
//! δ_N(q, a) = {δ_D(q, a)} for a ∈ Σ and δ_N(q, ε) = ∅. Q, Σ, q₀, F are unchanged. O(|Q|·|Σ|).
//! It exists so NFA-only constructions (union, concat, star, GNFA) can be applied to a DFA.

use crate::budget::Budget;
use crate::dfa::Dfa;
use crate::ids::StateId;
use crate::nfa::{Label, Nfa};
use crate::state::{Origin, StateInfo};
use crate::trace::{Step, Trace};

/// Reads the DFA as an NFA.
pub fn dfa_to_nfa(m: &Dfa, budget: &Budget) -> (Nfa, Trace) {
    let mut n = Nfa::empty(m.sigma.clone());
    for s in &m.states {
        // A struct literal with `..s.clone()` copies the remaining fields from another value.
        n.add_state(StateInfo {
            origin: if s.origin == Origin::Trap {
                Origin::Trap
            } else {
                Origin::Copied
            },
            ..s.clone()
        });
    }
    for q in 0..m.num_states() {
        let q = StateId::from_index(q);
        for a in m.sigma.ids() {
            n.add_edge(q, Label::Sym(a), m.next(q, a));
        }
    }
    n.start = m.start;
    n.accept = m.accept.clone();

    let mut trace = Trace::new(budget);
    let all: Vec<StateId> = (0..n.num_states()).map(StateId::from_index).collect();
    let edges = all
        .iter()
        .flat_map(|&q| n.edges(q).iter().map(move |&(_, r)| (q, r)))
        .collect();
    trace.push(
        Step::new(
            "p. 47",
            "Every DFA is an NFA: δ_N(q, a) = {δ_D(q, a)} and δ_N(q, ε) = ∅. Nothing else changes.",
        )
        .states(all)
        .edges(edges),
    );
    (n, trace)
}
