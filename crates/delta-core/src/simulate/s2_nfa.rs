//! S2 — running an NFA by tracking the set of active states (the book's "fingers", §1.2).
//!
//! S₀ = E({q₀}); Sᵢ₊₁ = E(⋃_{q ∈ Sᵢ} δ(q, wᵢ₊₁)); accept iff Sₙ ∩ F ≠ ∅. O(n · |δ|).

use super::{RunStepView, RunView};
use crate::bitset::BitSet;
use crate::ids::SymId;
use crate::nfa::Nfa;

/// S₀ … Sₙ, each ε-closed.
pub fn sets(m: &Nfa, input: &[SymId]) -> Vec<BitSet> {
    let mut out = Vec::with_capacity(input.len() + 1);
    let mut s = m.eps_closure(&BitSet::of([m.start.index()]));
    out.push(s.clone());
    for &a in input {
        s = m.eps_closure(&m.step(&s, a));
        out.push(s.clone());
    }
    out
}

/// Does N accept `input`?
pub fn accepts(m: &Nfa, input: &[SymId]) -> bool {
    sets(m, input).last().is_some_and(|s| m.any_accept(s))
}

fn names(m: &Nfa, s: &BitSet) -> String {
    if s.is_empty() {
        return "∅".into();
    }
    let parts: Vec<&str> = s
        .to_vec()
        .into_iter()
        .map(|i| m.name(crate::ids::StateId::from_index(i)))
        .collect();
    format!("{{{}}}", parts.join(", "))
}

/// The run as a step-by-step view for the UI.
pub fn run_view(name: &str, m: &Nfa, input: &[SymId]) -> RunView {
    let all = sets(m, input);
    let ids = |s: &BitSet| {
        s.to_vec()
            .into_iter()
            .map(|i| i as u32)
            .collect::<Vec<u32>>()
    };
    let mut steps = Vec::new();
    if let Some(s0) = all.first() {
        steps.push(RunStepView {
            pos: 0,
            active: ids(s0),
            message: format!("Start: E({{{}}}) = {}.", m.name(m.start), names(m, s0)),
        });
    }
    for (i, (pair, &a)) in all.windows(2).zip(input).enumerate() {
        if let [before, after] = pair {
            let moved = m.step(before, a);
            let mut msg = format!(
                "Read {}: δ({}, {}) = {}, then E(…) = {}.",
                m.sigma.display(a),
                names(m, before),
                m.sigma.display(a),
                names(m, &moved),
                names(m, after)
            );
            if after.is_empty() {
                msg.push_str(" No fingers left: every branch has died.");
            }
            steps.push(RunStepView {
                pos: (i + 1) as u32,
                active: ids(after),
                message: msg,
            });
        }
    }
    let accepted = accepts(m, input);
    if let Some(last) = steps.last_mut() {
        last.message.push_str(if accepted {
            " Some finger is on an accept state: accept."
        } else {
            " No finger is on an accept state: reject."
        });
    }
    RunView {
        object: name.to_string(),
        kind: "nfa".into(),
        input: input.iter().map(|&a| m.sigma.display(a)).collect(),
        steps,
        accepted,
    }
}
