//! C2 — NFA → DFA, the subset construction (Theorem 1.39).
//!
//! Given N = (Q, Σ, δ, q₀, F), build M = (Q′, Σ, δ′, q₀′, F′) with
//! - Q′ = P(Q),
//! - δ′(R, a) = { q ∈ Q | q ∈ E(δ(r, a)) for some r ∈ R },
//! - q₀′ = E({q₀}),
//! - F′ = { R ∈ Q′ | R contains an accept state of N }.
//!
//! `Reachable` mode (default) builds only the subsets reachable from q₀′ with a worklist, as in
//! Figure 1.44. `Full` mode builds all 2^k subsets, as in Example 1.41 (refused above k = 16).
//! Worst case O(2ⁿ · |Σ| · n), which is unavoidable, so the construction is budgeted.

use super::{ids, set_name};
use crate::bitset::BitSet;
use crate::budget::{Budget, TlError};
use crate::dfa::Dfa;
use crate::ids::StateId;
use crate::nfa::Nfa;
use crate::state::{Origin, StateInfo};
use crate::trace::{Step, Trace};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Which subsets become DFA states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsetMode {
    /// Only subsets reachable from E({q₀}) (Figure 1.44).
    Reachable,
    /// Every subset of Q (Example 1.41).
    Full,
}

/// The largest NFA `Full` mode accepts (2¹⁶ subsets).
pub const FULL_LIMIT: usize = 16;

const RULE: &str = "Theorem 1.39";

/// Converts an NFA into an equivalent DFA.
pub fn subset_construction(
    n: &Nfa,
    mode: SubsetMode,
    budget: &Budget,
) -> Result<(Dfa, Trace), TlError> {
    let k = n.num_states();
    if mode == SubsetMode::Full && k > FULL_LIMIT {
        return Err(TlError::Invalid(format!(
            "The full power set would have 2^{k} states; the limit is 2^{FULL_LIMIT}. Use the default (reachable subsets)."
        )));
    }
    let names: Vec<&str> = n.states.iter().map(|s| s.name.as_str()).collect();
    let mut trace = Trace::new(budget);

    // Q′: each DFA state is a subset; `index` finds a subset's DFA id.
    let mut subsets: Vec<BitSet> = Vec::new();
    let mut index: FxHashMap<BitSet, StateId> = FxHashMap::default();
    // A closure that captures variables mutably must itself be declared `mut`.
    let mut add = |set: BitSet, subsets: &mut Vec<BitSet>| -> Result<StateId, TlError> {
        if let Some(&id) = index.get(&set) {
            return Ok(id);
        }
        if subsets.len() as u32 >= budget.max_states {
            return Err(TlError::Budget {
                which: "max_states",
                limit: budget.max_states,
            });
        }
        let id = StateId::from_index(subsets.len());
        index.insert(set.clone(), id);
        subsets.push(set);
        Ok(id)
    };

    let start_set = n.eps_closure(&BitSet::of([n.start.index()]));
    let start_name = set_name(&names, &start_set);

    if mode == SubsetMode::Full {
        // Every subset, ordered by size and then element by element (∅, {1}, {2}, …).
        let mut all: Vec<BitSet> = (0u64..(1u64 << k))
            .map(|bits| BitSet::of((0..k).filter(|i| bits & (1 << i) != 0)))
            .collect();
        all.sort_by(|a, b| {
            a.len()
                .cmp(&b.len())
                .then_with(|| a.to_vec().cmp(&b.to_vec()))
        });
        for s in all {
            add(s, &mut subsets)?;
        }
    }
    // `?` after a Result returns the error to our caller immediately, or unwraps the Ok value.
    let start = add(start_set.clone(), &mut subsets)?;

    let n_sigma = n.sigma.len();
    // δ′ is filled in as we go; StateId(u32::MAX) marks "not computed yet".
    let mut delta: Vec<StateId> = Vec::new();
    let unset = StateId(u32::MAX);

    if mode == SubsetMode::Full {
        trace.push(
            Step::new(
                RULE,
                format!(
                    "Q′ = P(Q): all {} subsets of N's {k} states.",
                    subsets.len()
                ),
            )
            .states(ids_of(subsets.len())),
        );
    }
    trace.push(
        Step::new(
            RULE,
            format!(
                "Start state q₀′ = E({{{}}}) = {start_name}: {} plus everything reachable along ε-arrows.",
                n.name(n.start),
                n.name(n.start)
            ),
        )
        .states(if mode == SubsetMode::Full { Vec::new() } else { vec![start] })
        .focus(vec![start]),
    );

    // Worklist (FIFO so states appear in breadth-first order, like reading the figure).
    let mut work: VecDeque<StateId> = match mode {
        SubsetMode::Reachable => VecDeque::from([start]),
        SubsetMode::Full => ids_of(subsets.len()).into(),
    };
    while let Some(r) = work.pop_front() {
        let r_set = subsets.get(r.index()).cloned().unwrap_or_default();
        for a in n.sigma.ids() {
            let moved = n.step(&r_set, a);
            let target = n.eps_closure(&moved);
            let before = subsets.len();
            let t = add(target.clone(), &mut subsets)?;
            let is_new = subsets.len() > before;
            if is_new && mode == SubsetMode::Reachable {
                work.push_back(t);
            }
            if delta.len() < subsets.len() * n_sigma {
                delta.resize(subsets.len() * n_sigma, unset);
            }
            if let Some(slot) = delta.get_mut(r.index() * n_sigma + a.index()) {
                *slot = t;
            }
            if trace.has_room() {
                let parts: Vec<String> = r_set
                    .to_vec()
                    .into_iter()
                    .map(|q| {
                        format!(
                            "δ({}, {})",
                            names.get(q).copied().unwrap_or("?"),
                            n.sigma.display(a)
                        )
                    })
                    .collect();
                let union = if parts.is_empty() {
                    "∅".to_string()
                } else {
                    parts.join(" ∪ ")
                };
                let mut msg = format!(
                    "δ′({}, {}) = E({union}) = E({}) = {}",
                    set_name(&names, &r_set),
                    n.sigma.display(a),
                    set_name(&names, &moved),
                    set_name(&names, &target)
                );
                if is_new {
                    msg.push_str(" (a new state)");
                }
                msg.push('.');
                let mut step = Step::new(RULE, msg).edges(vec![(r, t)]).focus(vec![r, t]);
                if is_new && mode == SubsetMode::Reachable {
                    step.add_states = vec![t];
                }
                trace.push(step);
            }
        }
    }
    delta.resize(subsets.len() * n_sigma, unset);
    if delta.contains(&unset) {
        return Err(TlError::Internal(
            "subset construction left δ′ incomplete".into(),
        ));
    }

    let mut accept = BitSet::new();
    for (i, s) in subsets.iter().enumerate() {
        if n.any_accept(s) {
            accept.insert(i);
        }
    }
    let accept_names: Vec<String> = accept
        .to_vec()
        .into_iter()
        .filter_map(|i| subsets.get(i).map(|s| set_name(&names, s)))
        .collect();
    trace.push(
        Step::new(
            RULE,
            format!(
                "F′ = the subsets containing an accept state of N = {{{}}}.",
                accept_names.join(", ")
            ),
        )
        .focus(ids(&accept)),
    );

    let states = subsets
        .iter()
        .map(|s| StateInfo {
            name: set_name(&names, s),
            origin: Origin::Subset(s.clone()),
        })
        .collect();
    let dfa = Dfa::new(states, n.sigma.clone(), delta, start, accept)?;
    Ok((dfa, trace))
}

fn ids_of(n: usize) -> Vec<StateId> {
    (0..n).map(StateId::from_index).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use crate::nfa::Label;
    use crate::oracle;

    /// Example 1.41: N₄ with Q = {1,2,3}, Σ = {a,b}, start 1, F = {1};
    /// 1-b→2, 1-ε→3, 2-a→2, 2-a→3, 2-b→3, 3-a→1.
    fn n4() -> Nfa {
        let mut n = Nfa::empty(Alphabet::from_names(&["a".into(), "b".into()]));
        for name in ["1", "2", "3"] {
            n.add_state(StateInfo::user(name));
        }
        let (a, b) = (
            Label::Sym(crate::ids::SymId(0)),
            Label::Sym(crate::ids::SymId(1)),
        );
        let s = StateId;
        n.add_edge(s(0), b, s(1));
        n.add_edge(s(0), Label::Eps, s(2));
        n.add_edge(s(1), a, s(1));
        n.add_edge(s(1), a, s(2));
        n.add_edge(s(1), b, s(2));
        n.add_edge(s(2), a, s(0));
        n.start = s(0);
        n.accept = BitSet::of([0]);
        n
    }

    fn names(d: &Dfa) -> Vec<String> {
        d.states.iter().map(|s| s.name.clone()).collect()
    }

    #[test]
    fn example_1_41_full_power_set() {
        let (d, _) = subset_construction(&n4(), SubsetMode::Full, &Budget::default()).unwrap();
        assert_eq!(d.num_states(), 8);
        assert_eq!(d.name(d.start), "{1,3}");
        let mut acc: Vec<&str> = d
            .accept
            .to_vec()
            .iter()
            .map(|&i| d.name(StateId::from_index(i)))
            .collect();
        acc.sort();
        assert_eq!(acc, vec!["{1,2,3}", "{1,2}", "{1,3}", "{1}"]);
    }

    #[test]
    fn example_1_41_reachable_matches_figure_1_44() {
        let (d, trace) =
            subset_construction(&n4(), SubsetMode::Reachable, &Budget::default()).unwrap();
        let mut got = names(&d);
        got.sort();
        let mut want = vec!["{1,3}", "{2}", "{2,3}", "{3}", "∅", "{1,2,3}"];
        want.sort();
        assert_eq!(got, want);
        let nm = names(&d);
        let id = |x: &str| oracle::state_named(&nm, x).unwrap();
        let (a, b) = (crate::ids::SymId(0), crate::ids::SymId(1));
        let expect = [
            ("{1,3}", a, "{1,3}"),
            ("{1,3}", b, "{2}"),
            ("{2}", a, "{2,3}"),
            ("{2}", b, "{3}"),
            ("{2,3}", a, "{1,2,3}"),
            ("{2,3}", b, "{3}"),
            ("{3}", a, "{1,3}"),
            ("{3}", b, "∅"),
            ("{1,2,3}", a, "{1,2,3}"),
            ("{1,2,3}", b, "{2,3}"),
            ("∅", a, "∅"),
            ("∅", b, "∅"),
        ];
        for (from, sym, to) in expect {
            assert_eq!(d.name(d.next(id(from), sym)), to, "δ′({from}, {sym:?})");
        }
        let mut acc: Vec<&str> = d
            .accept
            .to_vec()
            .iter()
            .map(|&i| d.name(StateId::from_index(i)))
            .collect();
        acc.sort();
        assert_eq!(acc, vec!["{1,2,3}", "{1,3}"]);
        // start step + one step per (state, symbol) + the F′ step
        assert_eq!(trace.steps.len(), 1 + 6 * 2 + 1);
    }

    #[test]
    fn agrees_with_the_nfa_on_all_short_strings() {
        let n = n4();
        let (d, _) = subset_construction(&n, SubsetMode::Reachable, &Budget::default()).unwrap();
        for w in oracle::all_strings(2, 7) {
            assert_eq!(
                oracle::dfa_accepts(&d, &w),
                oracle::nfa_accepts(&n, &w),
                "{w:?}"
            );
        }
    }

    #[test]
    fn respects_the_state_budget() {
        let budget = Budget {
            max_states: 3,
            ..Budget::default()
        };
        let err = subset_construction(&n4(), SubsetMode::Reachable, &budget).unwrap_err();
        assert!(matches!(
            err,
            TlError::Budget {
                which: "max_states",
                ..
            }
        ));
    }
}
