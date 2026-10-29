//! Brute-force oracles: deliberately naive implementations of each *acceptance definition*,
//! straight from the book with no optimizations (docs/plan/06-ROADMAP-AND-TESTING.md §3.3).
//!
//! The fast algorithms are checked against these on every string up to some length. Slow and
//! obviously correct beats fast and clever when the job is catching bugs.

use crate::dfa::Dfa;
use crate::ids::{StateId, SymId};
use crate::nfa::{Label, Nfa};
use crate::regex::{Re, ReArena, ReId};

/// Definition on p. 40, followed literally.
pub fn dfa_accepts(m: &Dfa, w: &[SymId]) -> bool {
    let mut r = m.start;
    for &a in w {
        r = m.next(r, a);
    }
    m.is_accept(r)
}

/// Definition on p. 54: is there a way to write w as y₁…yₘ (yᵢ ∈ Σ_ε) with a matching run?
/// Searched over (state, position) pairs, so ε-loops cannot trap it.
pub fn nfa_accepts(m: &Nfa, w: &[SymId]) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![(m.start, 0usize)];
    while let Some((q, i)) = stack.pop() {
        if !seen.insert((q, i)) {
            continue;
        }
        if i == w.len() && m.accept.contains(q.index()) {
            return true;
        }
        for &(label, r) in m.edges(q) {
            match label {
                Label::Eps => stack.push((r, i)),
                Label::Sym(a) => {
                    if w.get(i) == Some(&a) {
                        stack.push((r, i + 1));
                    }
                }
            }
        }
    }
    false
}

/// Definition 1.52's meaning, by brute force: does `w` belong to L(R)?
pub fn regex_matches(arena: &ReArena, id: ReId, w: &[SymId]) -> bool {
    match arena.get(id) {
        Re::Empty => false,
        Re::Eps => w.is_empty(),
        Re::Sym(a) => w == [*a],
        Re::Union(kids) => kids.iter().any(|&k| regex_matches(arena, k, w)),
        Re::Concat(kids) => concat_matches(arena, kids, w),
        Re::Star(inner) => {
            // w ∈ R* iff w = ε, or w = xy with x ≠ ε, x ∈ R and y ∈ R*.
            w.is_empty()
                || (1..=w.len()).any(|i| {
                    regex_matches(arena, *inner, &w[..i]) && regex_matches(arena, id, &w[i..])
                })
        }
    }
}

fn concat_matches(arena: &ReArena, kids: &[ReId], w: &[SymId]) -> bool {
    // `match slice { [] => …, [first, rest @ ..] => … }` splits a slice into its head and tail.
    match kids {
        [] => w.is_empty(),
        [first, rest @ ..] => (0..=w.len())
            .any(|i| regex_matches(arena, *first, &w[..i]) && concat_matches(arena, rest, &w[i..])),
    }
}

/// Every string over an alphabet of `k` symbols with length ≤ `max_len`, in shortlex order.
pub fn all_strings(k: usize, max_len: usize) -> Vec<Vec<SymId>> {
    let mut out = vec![Vec::new()];
    let mut frontier = vec![Vec::new()];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for w in &frontier {
            for a in 0..k {
                // `w.clone()` copies the Vec so the original stays usable for the next symbol.
                let mut v = w.clone();
                v.push(SymId::from_index(a));
                next.push(v);
            }
        }
        out.extend(next.iter().cloned());
        frontier = next;
    }
    out
}

/// Convenience for tests: the state with a given name.
pub fn state_named(names: &[String], name: &str) -> Option<StateId> {
    names
        .iter()
        .position(|n| n == name)
        .map(StateId::from_index)
}
