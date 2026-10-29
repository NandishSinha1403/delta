//! C7 — the NFA closure constructions, as user commands (Theorems 1.45, 1.47, 1.49).
//!
//! 1. Union (Theorem 1.45): Q = {q₀} ∪ Q₁ ∪ Q₂, a new start q₀ with ε-arrows to q₁ and q₂;
//!    F = F₁ ∪ F₂.
//! 2. Concatenation (Theorem 1.47): Q = Q₁ ∪ Q₂, start q₁, ε-arrows from every state of F₁ to q₂;
//!    F = F₂.
//! 3. Star (Theorem 1.49): Q = {q₀} ∪ Q₁, a new start q₀ that is also accepting with an ε-arrow to
//!    q₁, and ε-arrows from every state of F₁ back to q₁; F = {q₀} ∪ F₁. Making the old start
//!    accepting instead is wrong (Exercise 1.15), which is why q₀ is new.
//!
//! The arrow-wiring helpers below are the same ones C3 (Lemma 1.55) uses, so `union(N1, N2)` and
//! `nfa(R1 ∪ R2)` build their machines identically. The proofs assume Q₁ and Q₂ are disjoint, so
//! N₂'s states are renamed with primes where names clash (q1 → q1'). If Σ₁ ≠ Σ₂ the result is over
//! Σ₁ ∪ Σ₂: a missing symbol simply has no arrows (δ returns ∅). O(|N₁| + |N₂|).

use super::fresh_name;
use crate::alphabet::Alphabet;
use crate::bitset::BitSet;
use crate::budget::{Budget, TlError};
use crate::ids::StateId;
use crate::nfa::{Label, Nfa};
use crate::state::{Origin, StateInfo};
use crate::trace::{Step, Trace};

/// A machine inside a bigger NFA: its start state and accept states.
pub(crate) struct Frag {
    pub start: StateId,
    pub accept: Vec<StateId>,
}

/// Theorem 1.45: ε-arrows from the new start `q0` to both starts; F = F₁ ∪ F₂.
pub(crate) fn union_arrows(
    nfa: &mut Nfa,
    q0: StateId,
    n1: &Frag,
    n2: &Frag,
) -> (Frag, Vec<(StateId, StateId)>) {
    nfa.add_edge(q0, Label::Eps, n1.start);
    nfa.add_edge(q0, Label::Eps, n2.start);
    let mut accept = n1.accept.clone();
    accept.extend(n2.accept.iter().copied());
    (
        Frag { start: q0, accept },
        vec![(q0, n1.start), (q0, n2.start)],
    )
}

/// Theorem 1.47: ε-arrows from every state of F₁ to q₂; start q₁, F = F₂.
pub(crate) fn concat_arrows(
    nfa: &mut Nfa,
    n1: &Frag,
    n2: &Frag,
) -> (Frag, Vec<(StateId, StateId)>) {
    let mut edges = Vec::new();
    for &f in &n1.accept {
        nfa.add_edge(f, Label::Eps, n2.start);
        edges.push((f, n2.start));
    }
    (
        Frag {
            start: n1.start,
            accept: n2.accept.clone(),
        },
        edges,
    )
}

/// Theorem 1.49: new accepting start `q0` with an ε-arrow to q₁; ε-arrows from F₁ back to q₁;
/// F = {q₀} ∪ F₁.
pub(crate) fn star_arrows(
    nfa: &mut Nfa,
    q0: StateId,
    n1: &Frag,
) -> (Frag, Vec<(StateId, StateId)>) {
    nfa.add_edge(q0, Label::Eps, n1.start);
    let mut edges = vec![(q0, n1.start)];
    for &f in &n1.accept {
        nfa.add_edge(f, Label::Eps, n1.start);
        edges.push((f, n1.start));
    }
    let mut accept = vec![q0];
    accept.extend(n1.accept.iter().copied());
    (Frag { start: q0, accept }, edges)
}

/// The NFA being assembled, with the names already taken (so copies stay disjoint).
struct Assembly<'a> {
    nfa: Nfa,
    taken: Vec<String>,
    budget: &'a Budget,
    trace: Trace,
}

impl Assembly<'_> {
    fn new(sigma: Alphabet, budget: &Budget) -> Assembly<'_> {
        Assembly {
            nfa: Nfa::empty(sigma),
            taken: Vec::new(),
            budget,
            trace: Trace::new(budget),
        }
    }

    fn add(&mut self, name: &str, origin: Origin) -> Result<StateId, TlError> {
        if self.nfa.num_states() as u32 >= self.budget.max_states {
            return Err(TlError::Budget {
                which: "max_states",
                limit: self.budget.max_states,
            });
        }
        let name = fresh_name(name, &self.taken);
        self.taken.push(name.clone());
        Ok(self.nfa.add_state(StateInfo { name, origin }))
    }

    /// Copies `src` in (renaming clashes with primes); records one step that adds it.
    fn copy(&mut self, src: &Nfa, label: &str) -> Result<Frag, TlError> {
        let base = self.nfa.num_states();
        let mut renamed = Vec::new();
        for s in &src.states {
            let q = self.add(&s.name, Origin::Copied)?;
            if self.nfa.name(q) != s.name {
                renamed.push(format!("{} → {}", s.name, self.nfa.name(q)));
            }
        }
        let at = |q: StateId| StateId::from_index(base + q.index());
        let mut edges = Vec::new();
        for (i, out) in src.delta.iter().enumerate() {
            let from = at(StateId::from_index(i));
            for &(label, to) in out {
                let label = match label {
                    Label::Eps => Label::Eps,
                    Label::Sym(a) => match self.nfa.sigma.get(src.sigma.name(a)) {
                        Some(b) => Label::Sym(b),
                        None => continue,
                    },
                };
                self.nfa.add_edge(from, label, at(to));
                edges.push((from, at(to)));
            }
        }
        let frag = Frag {
            start: at(src.start),
            accept: src
                .accept
                .to_vec()
                .into_iter()
                .map(|i| at(StateId::from_index(i)))
                .collect(),
        };
        let mut msg = format!(
            "Copy {label}: start {}, F = {}.",
            self.nfa.name(frag.start),
            self.names(&frag.accept)
        );
        if !renamed.is_empty() {
            msg.push_str(&format!(
                " Renamed to keep the states disjoint: {}.",
                renamed.join(", ")
            ));
        }
        let states = (base..self.nfa.num_states())
            .map(StateId::from_index)
            .collect();
        self.record(Step::new("Copy", msg).states(states).edges(edges), &frag);
        Ok(frag)
    }

    fn names(&self, v: &[StateId]) -> String {
        if v.is_empty() {
            return "∅".to_string();
        }
        let n: Vec<&str> = v.iter().map(|&q| self.nfa.name(q)).collect();
        format!("{{{}}}", n.join(", "))
    }

    fn record(&mut self, step: Step, frag: &Frag) {
        self.trace
            .push(step.status(frag.start, frag.accept.clone()));
    }

    fn finish(mut self, frag: Frag) -> (Nfa, Trace) {
        self.nfa.start = frag.start;
        self.nfa.accept = BitSet::of(frag.accept.iter().map(|q| q.index()));
        (self.nfa, self.trace)
    }
}

/// Σ₁ ∪ Σ₂, Σ₁'s symbols first.
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

/// Theorem 1.45: an NFA recognizing L(N₁) ∪ L(N₂).
pub fn nfa_union(n1: &Nfa, n2: &Nfa, budget: &Budget) -> Result<(Nfa, Trace), TlError> {
    let mut a = Assembly::new(combined(&n1.sigma, &n2.sigma), budget);
    // The new start is created first so it is named q0 (Q = {q₀} ∪ Q₁ ∪ Q₂).
    let q0 = a.add("q0", Origin::Rule("Theorem 1.45"))?;
    a.trace.push(
        Step::new(
            "Theorem 1.45",
            format!("New start state {}.", a.nfa.name(q0)),
        )
        .states(vec![q0]),
    );
    let f1 = a.copy(n1, "N₁")?;
    let f2 = a.copy(n2, "N₂")?;
    let (frag, edges) = union_arrows(&mut a.nfa, q0, &f1, &f2);
    let msg = format!(
        "ε-arrows from {} to q₁ = {} and q₂ = {}; F = F₁ ∪ F₂ = {}.",
        a.nfa.name(q0),
        a.nfa.name(f1.start),
        a.nfa.name(f2.start),
        a.names(&frag.accept)
    );
    a.record(
        Step::new("Theorem 1.45", msg).edges(edges).focus(vec![q0]),
        &frag,
    );
    Ok(a.finish(frag))
}

/// Theorem 1.47: an NFA recognizing L(N₁) ∘ L(N₂).
pub fn nfa_concat(n1: &Nfa, n2: &Nfa, budget: &Budget) -> Result<(Nfa, Trace), TlError> {
    let mut a = Assembly::new(combined(&n1.sigma, &n2.sigma), budget);
    let f1 = a.copy(n1, "N₁")?;
    let f2 = a.copy(n2, "N₂")?;
    let (frag, edges) = concat_arrows(&mut a.nfa, &f1, &f2);
    let msg = format!(
        "ε-arrows from F₁ = {} to q₂ = {}; the start stays q₁ = {}; F = F₂ = {}.",
        a.names(&f1.accept),
        a.nfa.name(f2.start),
        a.nfa.name(frag.start),
        a.names(&frag.accept)
    );
    let focus = f1.accept.iter().copied().chain([f2.start]).collect();
    a.record(
        Step::new("Theorem 1.47", msg).edges(edges).focus(focus),
        &frag,
    );
    Ok(a.finish(frag))
}

/// Theorem 1.49: an NFA recognizing L(N₁)*.
pub fn nfa_star(n1: &Nfa, budget: &Budget) -> Result<(Nfa, Trace), TlError> {
    let mut a = Assembly::new(n1.sigma.clone(), budget);
    let q0 = a.add("q0", Origin::Rule("Theorem 1.49"))?;
    a.trace.push(
        Step::new(
            "Theorem 1.49",
            format!(
                "New start state {} that is also accepting, so ε ∈ L(N₁)*. (Making N₁'s old start accepting instead is wrong: Exercise 1.15.)",
                a.nfa.name(q0)
            ),
        )
        .states(vec![q0]),
    );
    let f1 = a.copy(n1, "N₁")?;
    let (frag, edges) = star_arrows(&mut a.nfa, q0, &f1);
    let msg = format!(
        "ε-arrow from {} to q₁ = {}; ε-arrows from F₁ = {} back to q₁; F = {{q₀}} ∪ F₁ = {}.",
        a.nfa.name(q0),
        a.nfa.name(f1.start),
        a.names(&f1.accept),
        a.names(&frag.accept)
    );
    a.record(
        Step::new("Theorem 1.49", msg).edges(edges).focus(vec![q0]),
        &frag,
    );
    Ok(a.finish(frag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::SymId;
    use crate::oracle;

    /// An NFA from (from, label, to) arrows; label "" is ε.
    fn nfa(
        sigma: &[&str],
        names: &[&str],
        arrows: &[(usize, &str, usize)],
        accept: &[usize],
    ) -> Nfa {
        let sigma = Alphabet::from_names(&sigma.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        let mut n = Nfa::empty(sigma);
        for name in names {
            n.add_state(StateInfo::user(*name));
        }
        for &(f, l, t) in arrows {
            let label = if l.is_empty() {
                Label::Eps
            } else {
                Label::Sym(n.sigma.get(l).unwrap())
            };
            n.add_edge(StateId::from_index(f), label, StateId::from_index(t));
        }
        n.accept = BitSet::of(accept.iter().copied());
        n
    }

    /// Example 1.38's N₁ shape over {0,1}: strings containing 11.
    fn contains_11() -> Nfa {
        nfa(
            &["0", "1"],
            &["q1", "q2", "q3"],
            &[
                (0, "0", 0),
                (0, "1", 0),
                (0, "1", 1),
                (1, "1", 2),
                (2, "0", 2),
                (2, "1", 2),
            ],
            &[2],
        )
    }

    /// Strings over {0,1} of odd length.
    fn odd_length() -> Nfa {
        nfa(
            &["0", "1"],
            &["q1", "q2"],
            &[(0, "0", 1), (0, "1", 1), (1, "0", 0), (1, "1", 0)],
            &[1],
        )
    }

    /// Exactly the string "1" — its start is not accepting, so a naive star would get ε wrong.
    fn just_one() -> Nfa {
        nfa(&["0", "1"], &["q1", "q2"], &[(0, "1", 1)], &[1])
    }

    /// Is `w` in L(N₁) ∘ L(N₂)? By trying every split (straight from the definition, p. 44).
    fn in_concat(n1: &Nfa, n2: &Nfa, w: &[SymId]) -> bool {
        (0..=w.len()).any(|i| oracle::nfa_accepts(n1, &w[..i]) && oracle::nfa_accepts(n2, &w[i..]))
    }

    /// Is `w` in L(N)*? ε, or a nonempty prefix in L(N) followed by a string in L(N)* (p. 44).
    fn in_star(n: &Nfa, w: &[SymId]) -> bool {
        w.is_empty()
            || (1..=w.len()).any(|i| oracle::nfa_accepts(n, &w[..i]) && in_star(n, &w[i..]))
    }

    #[test]
    fn union_matches_the_oracle_up_to_length_8() {
        let (n1, n2) = (contains_11(), odd_length());
        let (u, _) = nfa_union(&n1, &n2, &Budget::default()).unwrap();
        // Q = {q₀} ∪ Q₁ ∪ Q₂, with N₂'s names primed.
        assert_eq!(u.num_states(), 1 + 3 + 2);
        assert_eq!(u.name(u.start), "q0");
        assert_eq!(u.name(StateId(4)), "q1'");
        for w in oracle::all_strings(2, 8) {
            let expect = oracle::nfa_accepts(&n1, &w) || oracle::nfa_accepts(&n2, &w);
            assert_eq!(oracle::nfa_accepts(&u, &w), expect, "{w:?}");
        }
    }

    #[test]
    fn concat_matches_the_oracle_up_to_length_8() {
        for (n1, n2) in [
            (contains_11(), odd_length()),
            (just_one(), contains_11()),
            (odd_length(), just_one()),
        ] {
            let (c, _) = nfa_concat(&n1, &n2, &Budget::default()).unwrap();
            assert_eq!(c.num_states(), n1.num_states() + n2.num_states());
            for w in oracle::all_strings(2, 8) {
                assert_eq!(
                    oracle::nfa_accepts(&c, &w),
                    in_concat(&n1, &n2, &w),
                    "{w:?}"
                );
            }
        }
    }

    #[test]
    fn star_matches_the_oracle_up_to_length_8() {
        for n in [contains_11(), odd_length(), just_one()] {
            let (s, _) = nfa_star(&n, &Budget::default()).unwrap();
            assert_eq!(s.num_states(), n.num_states() + 1);
            for w in oracle::all_strings(2, 8) {
                assert_eq!(oracle::nfa_accepts(&s, &w), in_star(&n, &w), "{w:?}");
            }
        }
    }

    #[test]
    fn star_adds_a_new_accepting_start_state() {
        // Theorem 1.49 adds a new start q0 that accepts; N₁'s old start q1 keeps its status.
        let (s, _) = nfa_star(&just_one(), &Budget::default()).unwrap();
        assert_eq!(s.name(s.start), "q0");
        assert!(s.accept.contains(s.start.index()));
        let old_start = StateId(1);
        assert_eq!(s.name(old_start), "q1");
        assert!(!s.accept.contains(old_start.index()));
        assert!(oracle::nfa_accepts(&s, &[]));
    }

    #[test]
    fn exercise_1_15_naive_star_is_wrong() {
        // The counterexample behind Exercise 1.15: N accepts strings ending in 1 and its start has
        // a loop. "Make the start accepting and add ε-arrows F₁ → q₁" also accepts "0".
        let n = nfa(
            &["0", "1"],
            &["q1", "q2"],
            &[(0, "0", 0), (0, "1", 1), (1, "", 0)],
            &[1],
        );
        let mut naive = n.clone();
        naive.accept.insert(0);
        let zero = [SymId(0)];
        assert!(oracle::nfa_accepts(&naive, &zero));
        assert!(!in_star(&n, &zero));
        let (s, _) = nfa_star(&n, &Budget::default()).unwrap();
        assert!(!oracle::nfa_accepts(&s, &zero));
    }

    #[test]
    fn different_alphabets_are_combined() {
        let a = nfa(&["a"], &["p", "q"], &[(0, "a", 1)], &[1]);
        let b = nfa(&["b"], &["p", "q"], &[(0, "b", 1)], &[1]);
        let (u, _) = nfa_union(&a, &b, &Budget::default()).unwrap();
        assert_eq!(u.sigma.len(), 2);
        let (sa, sb) = (u.sigma.get("a").unwrap(), u.sigma.get("b").unwrap());
        assert!(oracle::nfa_accepts(&u, &[sa]));
        assert!(oracle::nfa_accepts(&u, &[sb]));
        assert!(!oracle::nfa_accepts(&u, &[sa, sb]));
    }
}
