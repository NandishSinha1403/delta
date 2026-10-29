//! C4 — DFA or NFA → GNFA in special form (proof of Lemma 1.60, p. 71).
//!
//! 1. Add a new start state with an ε-arrow to the old start state.
//! 2. Add a new accept state with ε-arrows from the old accept states.
//! 3. Merge parallel arrows into one arrow labelled by the union of their labels.
//! 4. Every remaining pair (qᵢ, qⱼ) with qᵢ ≠ q_accept, qⱼ ≠ q_start gets the label ∅.
//!
//! O(|Q|² + |δ|). Labels are built faithfully (no simplification).

use super::fresh_name;
use crate::budget::Budget;
use crate::convert::c01_dfa_to_nfa::dfa_to_nfa;
use crate::dfa::Dfa;
use crate::gnfa::Gnfa;
use crate::ids::StateId;
use crate::nfa::{Label, Nfa};
use crate::object::Object;
use crate::regex::ReArena;
use crate::state::{Origin, StateInfo};
use crate::trace::{Step, Trace};

const RULE: &str = "Lemma 1.60";

/// Converts a DFA (read as an NFA first, p. 47) into a GNFA.
pub fn dfa_to_gnfa(m: &Dfa, budget: &Budget) -> (Gnfa, Trace) {
    let (n, _) = dfa_to_nfa(m, budget);
    nfa_to_gnfa(&n, budget)
}

/// Converts an NFA into a GNFA in special form.
pub fn nfa_to_gnfa(n: &Nfa, budget: &Budget) -> (Gnfa, Trace) {
    let taken: Vec<String> = n.states.iter().map(|s| s.name.clone()).collect();
    let start_name = fresh_name("qstart", &taken);
    let accept_name = fresh_name("qaccept", &taken);

    // Layout: index 0 = q_start, 1..=k = N's states, k+1 = q_accept.
    let k = n.num_states();
    let mut states = vec![StateInfo::rule(
        start_name.clone(),
        "Lemma 1.60: new start state",
    )];
    states.extend(n.states.iter().map(|s| StateInfo {
        origin: Origin::Copied,
        ..s.clone()
    }));
    states.push(StateInfo::rule(
        accept_name.clone(),
        "Lemma 1.60: new accept state",
    ));
    let size = k + 2;
    let start = StateId(0);
    let accept = StateId::from_index(k + 1);
    let shift = |q: StateId| StateId::from_index(q.index() + 1);

    let arena = ReArena::new();
    let empty = arena.empty();
    let mut g = Gnfa {
        states,
        sigma: n.sigma.clone(),
        start,
        accept,
        label: vec![empty; size * size],
        arena,
    };
    let mut trace = Trace::new(budget);

    // Step 1: new start state.
    let eps = g.arena.eps();
    g.set_label(start, shift(n.start), eps);
    // Step 3 (done now so snapshots show merged arrows): union of parallel labels.
    let mut merged = 0;
    for q in 0..k {
        let qi = StateId::from_index(q);
        let mut edges = n.edges(qi).to_vec();
        edges.sort();
        for (_, r) in edges.iter() {
            let current = g.label(shift(qi), shift(*r));
            if current != empty {
                continue;
            }
            // Fold every arrow qi → r into one union, in the book's order (ε first, then Σ).
            let labels: Vec<Label> = edges
                .iter()
                .filter(|(_, t)| t == r)
                .map(|(l, _)| *l)
                .collect();
            if labels.len() > 1 {
                merged += 1;
            }
            let mut acc = None;
            for l in labels {
                let x = match l {
                    Label::Eps => g.arena.eps(),
                    Label::Sym(a) => g.arena.sym(a),
                };
                // `Option::map_or` lets us fold: the first label starts the union.
                acc = Some(acc.map_or(x, |prev| g.arena.union_raw(prev, x)));
            }
            if let Some(re) = acc {
                g.set_label(shift(qi), shift(*r), re);
            }
        }
    }
    trace.push(
        Step::new(
            RULE,
            format!(
                "Add a new start state {start_name} with an ε-arrow to the old start state {}{}.",
                n.name(n.start),
                if merged > 0 {
                    format!("; merge parallel arrows into one arrow labelled with their union ({merged} merged)")
                } else {
                    String::new()
                }
            ),
        )
        .focus(vec![start, shift(n.start)])
        .snapshot(Object::Gnfa(g.clone())),
    );

    // Step 2: new accept state.
    let mut finals = Vec::new();
    for f in n.accept.to_vec() {
        let fq = StateId::from_index(f);
        g.set_label(shift(fq), accept, eps);
        finals.push(n.name(fq).to_string());
    }
    trace.push(
        Step::new(
            RULE,
            format!(
                "Add a new accept state {accept_name} with ε-arrows from the old accept states {{{}}}. They stop being accept states.",
                finals.join(", ")
            ),
        )
        .focus(vec![accept])
        .snapshot(Object::Gnfa(g.clone())),
    );

    // Step 4: ∅ everywhere else (already the default).
    trace.push(
        Step::new(
            RULE,
            "Every other pair of states gets an arrow labelled ∅ (hidden in the diagram, as in Figure 1.67). The GNFA is now in special form.",
        )
        .snapshot(Object::Gnfa(g.clone())),
    );
    (g, trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use crate::bitset::BitSet;
    use crate::ids::SymId;

    #[test]
    fn special_form_of_example_1_66() {
        // 1-a→1, 1-b→2, 2-a,b→2, F = {2}
        let sigma = Alphabet::from_names(&["a".into(), "b".into()]);
        let s = StateId;
        let m = Dfa::new(
            vec![StateInfo::user("1"), StateInfo::user("2")],
            sigma,
            vec![s(0), s(1), s(1), s(1)],
            s(0),
            BitSet::of([1]),
        )
        .unwrap();
        let (g, trace) = dfa_to_gnfa(&m, &Budget::default());
        assert_eq!(g.num_states(), 4);
        assert_eq!(g.label_text(s(0), s(1)), "ε");
        assert_eq!(g.label_text(s(1), s(1)), "a");
        assert_eq!(g.label_text(s(1), s(2)), "b");
        assert_eq!(g.label_text(s(2), s(2)), "a∪b");
        assert_eq!(g.label_text(s(2), s(3)), "ε");
        assert_eq!(g.label_text(s(0), s(3)), "∅");
        // nothing may enter q_start or leave q_accept
        assert_eq!(g.label_text(s(1), s(0)), "∅");
        assert_eq!(g.label_text(s(3), s(1)), "∅");
        assert_eq!(trace.steps.len(), 3);
        let _ = SymId(0);
    }
}
