//! S1 — running a DFA (acceptance as defined on p. 40).
//!
//! M accepts w = w₁…wₙ if there are states r₀, …, rₙ with r₀ = q₀, δ(rᵢ, wᵢ₊₁) = rᵢ₊₁, and rₙ ∈ F.
//! The run is that sequence r₀…rₙ: exactly what the diagram highlights. O(n).

use super::{RunStepView, RunView};
use crate::dfa::Dfa;
use crate::ids::{StateId, SymId};

/// The states r₀…rₙ visited on `input`.
pub fn path(m: &Dfa, input: &[SymId]) -> Vec<StateId> {
    let mut path = Vec::with_capacity(input.len() + 1);
    let mut q = m.start;
    path.push(q);
    for &a in input {
        q = m.next(q, a);
        path.push(q);
    }
    path
}

/// Does M accept `input`?
pub fn accepts(m: &Dfa, input: &[SymId]) -> bool {
    // `.last()` returns an Option; `is_some_and(f)` is true when it is Some and f holds.
    path(m, input).last().is_some_and(|&q| m.is_accept(q))
}

/// The run as a step-by-step view for the UI.
pub fn run_view(name: &str, m: &Dfa, input: &[SymId]) -> RunView {
    let p = path(m, input);
    let mut steps = vec![RunStepView {
        pos: 0,
        active: vec![m.start.0],
        message: format!("Start in q₀ = {}.", m.name(m.start)),
    }];
    // `.windows(2)` yields overlapping pairs [r₀, r₁], [r₁, r₂], …
    for (i, (pair, &a)) in p.windows(2).zip(input).enumerate() {
        if let [from, to] = pair {
            steps.push(RunStepView {
                pos: (i + 1) as u32,
                active: vec![to.0],
                message: format!(
                    "Read {}: δ({}, {}) = {}.",
                    m.sigma.display(a),
                    m.name(*from),
                    m.sigma.display(a),
                    m.name(*to)
                ),
            });
        }
    }
    let accepted = accepts(m, input);
    if let Some(last) = steps.last_mut() {
        let end = p.last().copied().unwrap_or(m.start);
        last.message.push_str(&if accepted {
            format!(" End in {} ∈ F: accept.", m.name(end))
        } else {
            format!(" End in {} ∉ F: reject.", m.name(end))
        });
    }
    RunView {
        object: name.to_string(),
        kind: "dfa".into(),
        input: input.iter().map(|&a| m.sigma.display(a)).collect(),
        steps,
        accepted,
    }
}
