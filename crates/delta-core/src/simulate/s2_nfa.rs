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

/// "q1 and q2" / "q1, q2 and q3": states for a sentence.
fn list(m: &Nfa, s: &BitSet) -> String {
    let n: Vec<&str> = s
        .to_vec()
        .into_iter()
        .map(|i| m.name(crate::ids::StateId::from_index(i)))
        .collect();
    match n.as_slice() {
        [] => "no state".into(),
        [one] => (*one).to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn without(s: &BitSet, i: usize) -> BitSet {
    let mut out = s.clone();
    out.remove(i);
    out
}

/// One sentence for a step: which fingers move where, which drop, what ε-arrows add.
fn explain_step(m: &Nfa, before: &BitSet, moved: &BitSet, after: &BitSet, a: SymId) -> String {
    let sym = m.sigma.display(a);
    let mut moves = Vec::new();
    let mut drops = Vec::new();
    for q in before.to_vec() {
        let targets = m.step(&BitSet::of([q]), a);
        let name = m.name(crate::ids::StateId::from_index(q));
        if targets.is_empty() {
            drops.push(name.to_string());
        } else {
            moves.push(format!("{name} follows {sym} to {}", list(m, &targets)));
        }
    }
    let mut parts = Vec::new();
    if !moves.is_empty() {
        parts.push(moves.join("; "));
    }
    match drops.as_slice() {
        [] => {}
        [one] => parts.push(format!("{one} has no {sym}-arrow, so its finger drops")),
        many => parts.push(format!(
            "{} have no {sym}-arrow, so their fingers drop",
            many.join(" and ")
        )),
    }
    let mut sentence = parts.join("; ");
    let extra = after.difference(moved);
    if !extra.is_empty() {
        sentence.push_str(&format!("; ε-arrows add {}", list(m, &extra)));
    }
    if after.is_empty() {
        sentence.push_str(". No fingers are left: every branch has died");
    }
    if sentence.is_empty() {
        sentence = format!("No finger can read {sym}");
    }
    // Not capitalised: sentences often start with a state name, and q1 ≠ Q1.
    format!("{sentence}.")
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
            explain: if s0.len() > 1 {
                format!(
                    "One finger starts on {}; ε-arrows give fingers on {} too.",
                    m.name(m.start),
                    list(m, &without(s0, m.start.index()))
                )
            } else {
                format!("One finger starts on the start state {}.", m.name(m.start))
            },
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
                explain: explain_step(m, before, &moved, after, a),
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
        let end = all.last().cloned().unwrap_or_default();
        let on_accept = end.intersection(&m.accept);
        last.explain.push_str(&if accepted {
            format!(
                " The input is used up with a finger on {}, an accept state, so {name} accepts.",
                list(m, &on_accept)
            )
        } else {
            format!(" The input is used up and no finger is on an accept state, so {name} rejects.")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use crate::nfa::Label;
    use crate::state::StateInfo;

    #[test]
    fn explains_each_step_in_plain_english() {
        // The tour's N: strings ending in "ab".
        let mut n = Nfa::empty(Alphabet::from_names(&["a".into(), "b".into()]));
        for q in ["q1", "q2", "q3"] {
            n.add_state(StateInfo::user(q));
        }
        let (a, b) = (SymId(0), SymId(1));
        let q = crate::ids::StateId::from_index;
        n.add_edge(q(0), Label::Sym(a), q(0));
        n.add_edge(q(0), Label::Sym(b), q(0));
        n.add_edge(q(0), Label::Sym(a), q(1));
        n.add_edge(q(1), Label::Sym(b), q(2));
        n.accept = BitSet::of([2]);
        let run = run_view("N", &n, &[a, a, b]);
        let explain: Vec<&str> = run.steps.iter().map(|s| s.explain.as_str()).collect();
        assert_eq!(explain[0], "One finger starts on the start state q1.");
        assert_eq!(
            explain[2],
            "q1 follows a to q1 and q2; q2 has no a-arrow, so its finger drops."
        );
        assert!(explain[3].ends_with("with a finger on q3, an accept state, so N accepts."));
    }
}
