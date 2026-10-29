//! C5 — GNFA → regular expression: CONVERT(G) from the proof of Lemma 1.60 (p. 73).
//!
//! While G has more than two states: pick q_rip ∉ {q_start, q_accept}, remove it, and for every
//! qᵢ ∈ Q′ − {q_accept}, qⱼ ∈ Q′ − {q_start} (qᵢ = qⱼ included) set
//!
//! ```text
//! δ′(qᵢ, qⱼ) = (R₁)(R₂)*(R₃) ∪ (R₄)
//! ```
//!
//! with R₁ = δ(qᵢ, q_rip), R₂ = δ(q_rip, q_rip), R₃ = δ(q_rip, qⱼ), R₄ = δ(qᵢ, qⱼ).
//! With two states left, the answer is the label of the arrow q_start → q_accept.
//!
//! The rip order changes the size of the result, not its language. `faithful` builds the formula
//! literally; otherwise only the identities from p. 66 are applied (labelled "simplified").
//! O(|Q|³) regex constructions; the regex itself can grow exponentially, so it is budgeted.

use crate::budget::{Budget, TlError};
use crate::gnfa::Gnfa;
use crate::ids::StateId;
use crate::object::{Object, RegexObj};
use crate::regex::{ReArena, ReId};
use crate::trace::{Step, Trace};

const RULE: &str = "Lemma 1.60";

/// Which state to rip next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RipOrder {
    /// In state order (q_start and q_accept never ripped).
    Index,
    /// These states first, in this order (the book's choice, e.g. Example 1.66 rips 2 then 1);
    /// any remaining states afterwards in state order.
    Given(Vec<StateId>),
    /// The state with the fewest arrows in × out first ("beyond the book").
    Heuristic,
}

/// Runs CONVERT(G).
pub fn gnfa_to_regex(
    g: &Gnfa,
    order: &RipOrder,
    faithful: bool,
    budget: &Budget,
) -> Result<(RegexObj, Trace), TlError> {
    if let RipOrder::Given(list) = order {
        for &q in list {
            if q == g.start || q == g.accept {
                return Err(TlError::Invalid(
                    "The rip order may only contain states other than q_start and q_accept (Lemma 1.60).".into(),
                ));
            }
        }
    }
    let mut g = g.clone();
    let mut trace = Trace::new(budget);
    // Original names of the states still present, so `Given` orders refer to the input GNFA.
    let mut alive: Vec<String> = g.states.iter().map(|s| s.name.clone()).collect();
    let mut given: Vec<String> = match order {
        RipOrder::Given(list) => list
            .iter()
            .filter_map(|q| alive.get(q.index()).cloned())
            .collect(),
        _ => Vec::new(),
    };

    while g.num_states() > 2 {
        let rip = choose(&g, order, &mut given, &alive);
        let rip_name = g.name(rip).to_string();
        let (next, updates) = rip_state(&g, rip, faithful)?;
        if next.arena.len() as u32 > budget.max_regex_nodes {
            return Err(TlError::Budget {
                which: "max_regex_nodes",
                limit: budget.max_regex_nodes,
            });
        }
        if trace.has_room() {
            let mut msg = format!(
                "Rip {rip_name}. For each pair (qᵢ, qⱼ): δ′(qᵢ, qⱼ) = (R₁)(R₂)*(R₃) ∪ (R₄) with R₂ = δ({rip_name}, {rip_name}) = {}.",
                g.label_text(rip, rip)
            );
            for u in &updates {
                msg.push_str(&format!("\n• δ′({}, {}) = {}", u.0, u.1, u.2));
            }
            trace.push(Step::new(RULE, msg).snapshot(Object::Gnfa(next.clone())));
        }
        alive.remove(rip.index());
        g = next;
    }

    let root = g.label(g.start, g.accept);
    trace.push(Step::new(
        RULE,
        format!(
            "Two states remain, so the answer is the label of {} → {}: {}.",
            g.name(g.start),
            g.name(g.accept),
            g.arena.print(root, &g.sigma)
        ),
    ));
    Ok((
        RegexObj {
            arena: g.arena,
            root,
            sigma: g.sigma,
        },
        trace,
    ))
}

fn choose(g: &Gnfa, order: &RipOrder, given: &mut Vec<String>, alive: &[String]) -> StateId {
    let candidates: Vec<StateId> = (0..g.num_states())
        .map(StateId::from_index)
        .filter(|&q| q != g.start && q != g.accept)
        .collect();
    let first = candidates.first().copied().unwrap_or(StateId(1));
    match order {
        RipOrder::Index => first,
        RipOrder::Given(_) => {
            while !given.is_empty() {
                let name = given.remove(0);
                if let Some(i) = alive.iter().position(|n| *n == name) {
                    return StateId::from_index(i);
                }
            }
            first
        }
        RipOrder::Heuristic => {
            let empty = g.arena.empty();
            let cost = |q: StateId| {
                let ins = (0..g.num_states())
                    .filter(|&i| i != q.index() && g.label(StateId::from_index(i), q) != empty)
                    .count();
                let outs = (0..g.num_states())
                    .filter(|&j| j != q.index() && g.label(q, StateId::from_index(j)) != empty)
                    .count();
                (ins * outs, q.index())
            };
            candidates
                .into_iter()
                .min_by_key(|&q| cost(q))
                .unwrap_or(first)
        }
    }
}

/// G′ = G without q_rip, plus the non-∅ labels that changed, for the trace message.
#[allow(clippy::type_complexity)]
fn rip_state(
    g: &Gnfa,
    rip: StateId,
    faithful: bool,
) -> Result<(Gnfa, Vec<(String, String, String)>), TlError> {
    let n = g.num_states();
    let keep: Vec<StateId> = (0..n)
        .map(StateId::from_index)
        .filter(|&q| q != rip)
        .collect();
    let mut arena: ReArena = g.arena.clone();
    let empty = arena.empty();
    let index_of = |q: StateId| keep.iter().position(|&k| k == q).map(StateId::from_index);
    let (Some(start), Some(accept)) = (index_of(g.start), index_of(g.accept)) else {
        // `let … else { … }` binds the pattern or runs the else block, which must leave the function.
        return Err(TlError::Internal("tried to rip q_start or q_accept".into()));
    };
    let m = keep.len();
    let mut label = vec![empty; m * m];
    let mut updates = Vec::new();
    let r2 = g.label(rip, rip);
    for (ni, &qi) in keep.iter().enumerate() {
        if qi == g.accept {
            continue;
        }
        for (nj, &qj) in keep.iter().enumerate() {
            if qj == g.start {
                continue;
            }
            let (r1, r3, r4) = (g.label(qi, rip), g.label(rip, qj), g.label(qi, qj));
            let new: ReId = if faithful {
                let s = arena.star_raw(r2);
                let c = arena.concat_raw(r1, s);
                let c = arena.concat_raw(c, r3);
                arena.union_raw(c, r4)
            } else {
                let s = arena.star(r2);
                let c = arena.concat_all(&[r1, s, r3]);
                arena.union(c, r4)
            };
            if let Some(slot) = label.get_mut(ni * m + nj) {
                *slot = new;
            }
            if new != r4 && new != empty {
                updates.push((
                    g.name(qi).to_string(),
                    g.name(qj).to_string(),
                    arena.print(new, &g.sigma),
                ));
            }
        }
    }
    let states = keep
        .iter()
        .filter_map(|q| g.states.get(q.index()).cloned())
        .collect();
    Ok((
        Gnfa {
            states,
            sigma: g.sigma.clone(),
            start,
            accept,
            label,
            arena,
        },
        updates,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use crate::bitset::BitSet;
    use crate::convert::c04_to_gnfa::dfa_to_gnfa;
    use crate::dfa::Dfa;
    use crate::oracle;
    use crate::state::StateInfo;

    fn example_1_66() -> Dfa {
        let s = StateId;
        Dfa::new(
            vec![StateInfo::user("1"), StateInfo::user("2")],
            Alphabet::from_names(&["a".into(), "b".into()]),
            vec![s(0), s(1), s(1), s(1)],
            s(0),
            BitSet::of([1]),
        )
        .unwrap()
    }

    #[test]
    fn example_1_66_rip_2_then_1() {
        let (g, _) = dfa_to_gnfa(&example_1_66(), &Budget::default());
        // In the GNFA, state "1" is index 1 and "2" is index 2.
        let order = RipOrder::Given(vec![StateId(2), StateId(1)]);
        let (r, trace) = gnfa_to_regex(&g, &order, false, &Budget::default()).unwrap();
        assert_eq!(r.text(), "a*b(a∪b)*");
        // After ripping 2, the arrow 1 → q_accept is labelled b(a∪b)*.
        assert!(
            trace.steps[0].message.contains("δ′(1, qaccept) = b(a∪b)*"),
            "{}",
            trace.steps[0].message
        );
    }

    #[test]
    fn faithful_mode_keeps_the_formula_literal_but_means_the_same() {
        let d = example_1_66();
        let (g, _) = dfa_to_gnfa(&d, &Budget::default());
        let order = RipOrder::Given(vec![StateId(2), StateId(1)]);
        let (r, _) = gnfa_to_regex(&g, &order, true, &Budget::default()).unwrap();
        assert!(
            r.text().contains('∅'),
            "faithful output should keep ∅ terms: {}",
            r.text()
        );
        for w in oracle::all_strings(2, 6) {
            assert_eq!(
                oracle::regex_matches(&r.arena, r.root, &w),
                oracle::dfa_accepts(&d, &w),
                "{w:?}"
            );
        }
    }

    #[test]
    fn rip_order_may_not_contain_start_or_accept() {
        let (g, _) = dfa_to_gnfa(&example_1_66(), &Budget::default());
        let err = gnfa_to_regex(
            &g,
            &RipOrder::Given(vec![StateId(0)]),
            false,
            &Budget::default(),
        );
        assert!(err.is_err());
    }
}
