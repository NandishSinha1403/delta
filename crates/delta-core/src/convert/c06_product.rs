//! C6 — the product construction on DFAs (Theorem 1.25 and its footnote).
//!
//! > M = (Q, Σ, δ, q₀, F) where Q = Q₁ × Q₂, δ((r₁, r₂), a) = (δ₁(r₁, a), δ₂(r₂, a)),
//! > q₀ = (q₁, q₂), and F = (F₁ × Q₂) ∪ (Q₁ × F₂). [Theorem 1.25, union]
//! > Setting F = F₁ × F₂ gives a machine for the intersection. [footnote, p. 46]
//!
//! Only pairs reachable from (q₁, q₂) are built (worklist, in breadth-first order), so the result
//! has at most |Q₁|·|Q₂| states. The proof assumes one alphabet; when Σ₁ ≠ Σ₂ both machines are
//! extended to Σ₁ ∪ Σ₂ with a trap state for the symbols they lack (docs/plan/02-CONVERSIONS.md C6).
//! O(|Q₁|·|Q₂|·|Σ|).

use crate::alphabet::Alphabet;
use crate::bitset::BitSet;
use crate::budget::{Budget, TlError};
use crate::dfa::Dfa;
use crate::ids::{StateId, SymId};
use crate::state::StateInfo;
use crate::trace::{Step, Trace};
use std::collections::{HashMap, VecDeque};

/// Which language the product recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductOp {
    /// F = (F₁ × Q₂) ∪ (Q₁ × F₂) — Theorem 1.25.
    Union,
    /// F = F₁ × F₂ — the footnote to Theorem 1.25.
    Intersect,
}

/// One component of a pair: a state of Mᵢ, or the trap added for a symbol Mᵢ lacks.
// `Option<StateId>`: `Some(q)` is a real state, `None` is the trap (it never leaves itself).
type Part = Option<StateId>;

/// One input machine seen through the combined alphabet Σ₁ ∪ Σ₂.
struct Side<'a> {
    m: &'a Dfa,
    /// For each symbol of the combined alphabet, the same symbol in Mᵢ's alphabet (if it has it).
    sym: Vec<Option<SymId>>,
}

impl Side<'_> {
    fn next(&self, r: Part, a: SymId) -> Part {
        let q = r?;
        // `?` on an Option returns None early: a missing symbol sends Mᵢ to its trap.
        let b = self.sym.get(a.index()).copied().flatten()?;
        Some(self.m.next(q, b))
    }

    fn accepts(&self, r: Part) -> bool {
        r.is_some_and(|q| self.m.is_accept(q))
    }

    fn name(&self, r: Part) -> String {
        r.map_or_else(|| "trap".to_string(), |q| self.m.name(q).to_string())
    }
}

/// Mᵢ seen through the combined alphabet.
// `<'a>` names the borrow: the returned Side lives no longer than the DFA it points to.
fn side<'a>(m: &'a Dfa, sigma: &Alphabet) -> Side<'a> {
    Side {
        m,
        sym: sigma.ids().map(|a| m.sigma.get(sigma.name(a))).collect(),
    }
}

/// Σ₁ ∪ Σ₂: Σ₁'s symbols in order, then the symbols only Σ₂ has.
fn combined(s1: &Alphabet, s2: &Alphabet) -> Alphabet {
    let mut names: Vec<String> = s1.ids().map(|a| s1.name(a).to_string()).collect();
    for a in s2.ids() {
        let n = s2.name(a).to_string();
        if !names.contains(&n) {
            names.push(n);
        }
    }
    Alphabet::from_names(&names)
}

/// Builds the product of `m1` and `m2` (reachable pairs only), with a trace of every pair and arrow.
pub fn product(
    m1: &Dfa,
    m2: &Dfa,
    op: ProductOp,
    budget: &Budget,
) -> Result<(Dfa, Trace), TlError> {
    let sigma = combined(&m1.sigma, &m2.sigma);
    let (s1, s2) = (side(m1, &sigma), side(m2, &sigma));
    let rule = "Theorem 1.25";
    let accepting = |p: (Part, Part)| match op {
        ProductOp::Union => s1.accepts(p.0) || s2.accepts(p.1),
        ProductOp::Intersect => s1.accepts(p.0) && s2.accepts(p.1),
    };

    let mut trace = Trace::new(budget);
    let mut states: Vec<StateInfo> = Vec::new();
    let mut pairs: Vec<(Part, Part)> = Vec::new();
    let mut index: HashMap<(Part, Part), StateId> = HashMap::new();
    let mut accept = BitSet::new();
    let mut queue = VecDeque::new();
    // δ is filled row by row as pairs are processed; rows are in state order.
    let mut rows: Vec<Vec<StateId>> = Vec::new();

    // A closure that adds a pair as a new state (or finds it) — returns (id, was it new?).
    let mut intern = |p: (Part, Part),
                      states: &mut Vec<StateInfo>,
                      pairs: &mut Vec<(Part, Part)>,
                      accept: &mut BitSet|
     -> Result<(StateId, bool), TlError> {
        if let Some(&id) = index.get(&p) {
            return Ok((id, false));
        }
        if states.len() as u32 >= budget.max_states {
            return Err(TlError::Budget {
                which: "max_states",
                limit: budget.max_states,
            });
        }
        let id = StateId::from_index(states.len());
        states.push(StateInfo::rule(
            format!("({},{})", s1.name(p.0), s2.name(p.1)),
            rule,
        ));
        pairs.push(p);
        if accepting(p) {
            accept.insert(id.index());
        }
        index.insert(p, id);
        Ok((id, true))
    };

    let start_pair = (Some(m1.start), Some(m2.start));
    let (start, _) = intern(start_pair, &mut states, &mut pairs, &mut accept)?;
    queue.push_back(start);
    trace.push(
        Step::new(
            rule,
            format!(
                "Start state q₀ = (q₁, q₂) = {}.",
                states.get(start.index()).map_or("?", |s| s.name.as_str())
            ),
        )
        .states(vec![start]),
    );

    while let Some(from) = queue.pop_front() {
        let Some(&(r1, r2)) = pairs.get(from.index()) else {
            continue;
        };
        let mut row = Vec::with_capacity(sigma.len());
        for a in sigma.ids() {
            let to_pair = (s1.next(r1, a), s2.next(r2, a));
            let (to, new) = intern(to_pair, &mut states, &mut pairs, &mut accept)?;
            row.push(to);
            if new {
                queue.push_back(to);
            }
            if trace.has_room() {
                let name = |q: StateId| states.get(q.index()).map_or("?", |s| s.name.as_str());
                let msg = format!(
                    "δ({}, {}) = (δ₁({}, {a_name}), δ₂({}, {a_name})) = {}{}",
                    name(from),
                    sigma.name(a),
                    s1.name(r1),
                    s2.name(r2),
                    name(to),
                    if new { " (a new state)." } else { "." },
                    a_name = sigma.name(a),
                );
                let step = Step::new(rule, msg).edges(vec![(from, to)]);
                let step = if new {
                    step.states(vec![to]).focus(vec![from, to])
                } else {
                    step.focus(vec![from, to])
                };
                trace.push(step);
            } else {
                trace.truncated = true;
            }
        }
        rows.push(row);
    }

    let names: Vec<String> = accept
        .to_vec()
        .into_iter()
        .filter_map(|i| states.get(i).map(|s| s.name.clone()))
        .collect();
    let formula = match op {
        ProductOp::Union => "F = (F₁ × Q₂) ∪ (Q₁ × F₂): the pairs where either machine accepts",
        ProductOp::Intersect => {
            "F = F₁ × F₂ (footnote to Theorem 1.25): the pairs where both machines accept"
        }
    };
    let set = if names.is_empty() {
        "∅".to_string()
    } else {
        format!("{{{}}}", names.join(", "))
    };
    trace.push(
        Step::new(rule, format!("{formula}, so F = {set}.")).focus(
            accept
                .to_vec()
                .into_iter()
                .map(StateId::from_index)
                .collect(),
        ),
    );

    let delta: Vec<StateId> = rows.into_iter().flatten().collect();
    let dfa = Dfa::new(states, sigma, delta, start, accept)?;
    Ok((dfa, trace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle;

    /// A DFA over `sigma` from a table of target indices, row per state.
    fn dfa(sigma: &[&str], names: &[&str], table: &[&[usize]], accept: &[usize]) -> Dfa {
        let sigma = Alphabet::from_names(&sigma.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        let states = names.iter().map(|n| StateInfo::user(*n)).collect();
        let delta = table
            .iter()
            .flat_map(|row| row.iter().map(|&t| StateId::from_index(t)))
            .collect();
        Dfa::new(
            states,
            sigma,
            delta,
            StateId(0),
            BitSet::of(accept.iter().copied()),
        )
        .unwrap()
    }

    /// Strings over {0,1} with an even number of 0s.
    fn even_zeros() -> Dfa {
        dfa(&["0", "1"], &["e", "o"], &[&[1, 0], &[0, 1]], &[0])
    }

    /// Strings over {0,1} that end in 1.
    fn ends_in_one() -> Dfa {
        dfa(&["0", "1"], &["x", "y"], &[&[0, 1], &[0, 1]], &[1])
    }

    #[test]
    fn union_and_intersection_match_the_oracle_up_to_length_8() {
        let (m1, m2) = (even_zeros(), ends_in_one());
        let (u, _) = product(&m1, &m2, ProductOp::Union, &Budget::default()).unwrap();
        let (i, _) = product(&m1, &m2, ProductOp::Intersect, &Budget::default()).unwrap();
        for w in oracle::all_strings(2, 8) {
            let (a, b) = (oracle::dfa_accepts(&m1, &w), oracle::dfa_accepts(&m2, &w));
            assert_eq!(oracle::dfa_accepts(&u, &w), a || b, "union on {w:?}");
            assert_eq!(oracle::dfa_accepts(&i, &w), a && b, "intersection on {w:?}");
        }
    }

    #[test]
    fn builds_the_book_tuple() {
        let (u, trace) = product(
            &even_zeros(),
            &ends_in_one(),
            ProductOp::Union,
            &Budget::default(),
        )
        .unwrap();
        // All four pairs are reachable; the start is (q₁, q₂).
        assert_eq!(u.num_states(), 4);
        assert_eq!(u.name(u.start), "(e,x)");
        let names: Vec<&str> = (0..4).map(|i| u.name(StateId::from_index(i))).collect();
        assert_eq!(names, ["(e,x)", "(o,x)", "(e,y)", "(o,y)"]);
        // F = (F₁ × Q₂) ∪ (Q₁ × F₂): (e,x), (e,y), (o,y).
        assert_eq!(u.accept.to_vec(), vec![0, 2, 3]);
        // start + |Q|·|Σ| arrows + the F step.
        assert_eq!(trace.steps.len(), 1 + 8 + 1);
    }

    #[test]
    fn keeps_only_reachable_pairs() {
        // M1 never leaves its start; M2 has an unreachable state. 1 × 2 reachable pairs, not 2 × 3.
        let m1 = dfa(&["a"], &["p", "dead"], &[&[0], &[1]], &[0]);
        let m2 = dfa(&["a"], &["s", "t", "u"], &[&[1], &[0], &[2]], &[1]);
        let (i, _) = product(&m1, &m2, ProductOp::Intersect, &Budget::default()).unwrap();
        assert_eq!(i.num_states(), 2);
    }

    #[test]
    fn different_alphabets_extend_with_a_trap() {
        // L1 = a*, L2 = b* over different alphabets; union over {a, b} is a* ∪ b*.
        let m1 = dfa(&["a"], &["p"], &[&[0]], &[0]);
        let m2 = dfa(&["b"], &["s"], &[&[0]], &[0]);
        let (u, _) = product(&m1, &m2, ProductOp::Union, &Budget::default()).unwrap();
        assert_eq!(u.sigma.len(), 2);
        let a = |w: &str| -> Vec<SymId> {
            w.chars()
                .map(|c| u.sigma.get(&c.to_string()).unwrap())
                .collect()
        };
        for (w, expect) in [
            ("", true),
            ("aaa", true),
            ("bb", true),
            ("ab", false),
            ("ba", false),
        ] {
            assert_eq!(oracle::dfa_accepts(&u, &a(w)), expect, "{w}");
        }
        assert!(u.states.iter().any(|s| s.name == "(trap,s)"));
    }
}
