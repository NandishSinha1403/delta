//! C3 — regular expression → NFA (Lemma 1.55).
//!
//! Structural recursion over Definition 1.52 using *Sipser's* constructions (not Thompson's: they
//! differ, and the state counts must match the book's figures):
//! 1. R = a: two states q₁ -a→ q₂, F = {q₂}.
//! 2. R = ε: one accepting state.
//! 3. R = ∅: one state, no accept states.
//! 4. R₁ ∪ R₂ (Theorem 1.45): new start with ε-arrows to both starts; F = F₁ ∪ F₂.
//! 5. R₁ ∘ R₂ (Theorem 1.47): ε-arrows from F₁ to N₂'s start; F = F₂.
//! 6. R₁* (Theorem 1.49): new start that is also accepting, ε-arrow to q₁; ε-arrows from F₁ back to
//!    q₁; F = {q₀} ∪ F₁. (Not "just make the start accepting": Exercise 1.15 shows that is wrong.)
//!
//! Cases 4–6 call the same arrow-wiring helpers as the `union`, `concat` and `star` commands (C7).
//! n-ary unions and concatenations fold left, matching the book's figures. O(|R|).

use super::c07_nfa_closure::{Frag, concat_arrows, star_arrows, union_arrows};
use super::ids;
use crate::bitset::BitSet;
use crate::budget::{Budget, TlError};
use crate::ids::StateId;
use crate::nfa::{Label, Nfa};
use crate::object::RegexObj;
use crate::regex::{Re, ReId};
use crate::state::StateInfo;
use crate::trace::{Step, Trace};

/// Deepest expression accepted (deeper input would risk overflowing the call stack).
pub const MAX_DEPTH: usize = 500;

struct Builder<'a> {
    // `'a` is a lifetime: it says this Builder borrows `r` and must not outlive it. (Private
    // helper types may use lifetimes; public types stay lifetime-free.)
    r: &'a RegexObj,
    nfa: Nfa,
    trace: Trace,
    /// Accept states of the fragments built so far that are still accepting.
    live_accept: BitSet,
    budget: &'a Budget,
}

impl Builder<'_> {
    fn state(&mut self, rule: &'static str) -> Result<StateId, TlError> {
        if self.nfa.num_states() as u32 >= self.budget.max_states {
            return Err(TlError::Budget {
                which: "max_states",
                limit: self.budget.max_states,
            });
        }
        let name = format!("q{}", self.nfa.num_states() + 1);
        Ok(self.nfa.add_state(StateInfo::rule(name, rule)))
    }

    fn text(&self, id: ReId) -> String {
        self.r.arena.print(id, &self.r.sigma)
    }

    fn names(&self, v: &[StateId]) -> String {
        let n: Vec<&str> = v.iter().map(|&q| self.nfa.name(q)).collect();
        format!("{{{}}}", n.join(", "))
    }

    fn record(&mut self, step: Step, frag: &Frag) {
        let accept = ids(&self.live_accept);
        self.trace.push(step.status(frag.start, accept));
    }

    fn build(&mut self, id: ReId, depth: usize) -> Result<Frag, TlError> {
        if depth > MAX_DEPTH {
            return Err(TlError::Invalid(format!(
                "the expression is nested more than {MAX_DEPTH} levels deep"
            )));
        }
        // `.clone()` here copies the node so we stop borrowing the arena while we build.
        let node = self.r.arena.get(id).clone();
        match node {
            Re::Sym(a) => {
                let q1 = self.state("Lemma 1.55 (1)")?;
                let q2 = self.state("Lemma 1.55 (1)")?;
                self.nfa.add_edge(q1, Label::Sym(a), q2);
                self.live_accept.insert(q2.index());
                let frag = Frag {
                    start: q1,
                    accept: vec![q2],
                };
                let msg = format!(
                    "R = {}: two states {} -{}→ {}, accepting {}.",
                    self.text(id),
                    self.nfa.name(q1),
                    self.r.sigma.display(a),
                    self.nfa.name(q2),
                    self.nfa.name(q2)
                );
                let step = Step::new("Lemma 1.55 (1)", msg)
                    .states(vec![q1, q2])
                    .edges(vec![(q1, q2)]);
                self.record(step, &frag);
                Ok(frag)
            }
            Re::Eps => {
                let q = self.state("Lemma 1.55 (2)")?;
                self.live_accept.insert(q.index());
                let frag = Frag {
                    start: q,
                    accept: vec![q],
                };
                let msg = format!("R = ε: one state {} that is accepting.", self.nfa.name(q));
                self.record(Step::new("Lemma 1.55 (2)", msg).states(vec![q]), &frag);
                Ok(frag)
            }
            Re::Empty => {
                let q = self.state("Lemma 1.55 (3)")?;
                let frag = Frag {
                    start: q,
                    accept: vec![],
                };
                let msg = format!(
                    "R = ∅: one state {} and no accept states.",
                    self.nfa.name(q)
                );
                self.record(Step::new("Lemma 1.55 (3)", msg).states(vec![q]), &frag);
                Ok(frag)
            }
            Re::Union(kids) => {
                let mut iter = kids.iter();
                let first = iter.next().copied().unwrap_or(id);
                let mut acc = self.build(first, depth + 1)?;
                let mut acc_text = self.text(first);
                for &k in iter {
                    let right = self.build(k, depth + 1)?;
                    let q0 = self.state("Theorem 1.45")?;
                    let (next, edges) = union_arrows(&mut self.nfa, q0, &acc, &right);
                    let accept = next.accept.clone();
                    let msg = format!(
                        "{} ∪ {}: new start {} with ε-arrows to {} and {}; F = F₁ ∪ F₂ = {}.",
                        acc_text,
                        self.text(k),
                        self.nfa.name(q0),
                        self.nfa.name(acc.start),
                        self.nfa.name(right.start),
                        self.names(&accept)
                    );
                    acc = next;
                    acc_text = format!("{acc_text}∪{}", self.text(k));
                    self.record(
                        Step::new("Theorem 1.45", msg).states(vec![q0]).edges(edges),
                        &acc,
                    );
                }
                Ok(acc)
            }
            Re::Concat(kids) => {
                let mut iter = kids.iter();
                let first = iter.next().copied().unwrap_or(id);
                let mut acc = self.build(first, depth + 1)?;
                let mut acc_text = self.concat_part(first);
                for &k in iter {
                    let right = self.build(k, depth + 1)?;
                    let (next, edges) = concat_arrows(&mut self.nfa, &acc, &right);
                    for &f in &acc.accept {
                        self.live_accept.remove(f.index());
                    }
                    let msg = format!(
                        "{} ∘ {}: ε-arrows from F₁ = {} to {}; F = F₂ = {}.",
                        acc_text,
                        self.concat_part(k),
                        self.names(&acc.accept),
                        self.nfa.name(right.start),
                        self.names(&right.accept)
                    );
                    acc = next;
                    acc_text.push_str(&self.concat_part(k));
                    let focus = edges.iter().map(|e| e.0).chain([acc.start]).collect();
                    self.record(
                        Step::new("Theorem 1.47", msg).edges(edges).focus(focus),
                        &acc,
                    );
                }
                Ok(acc)
            }
            Re::Star(inner) => {
                let body = self.build(inner, depth + 1)?;
                let q0 = self.state("Theorem 1.49")?;
                let (frag, edges) = star_arrows(&mut self.nfa, q0, &body);
                self.live_accept.insert(q0.index());
                let msg = format!(
                    "({})*: new start {} that is also accepting, with an ε-arrow to {}; ε-arrows from F₁ = {} back to {}.",
                    self.text(inner),
                    self.nfa.name(q0),
                    self.nfa.name(body.start),
                    self.names(&body.accept),
                    self.nfa.name(body.start)
                );
                self.record(
                    Step::new("Theorem 1.49", msg).states(vec![q0]).edges(edges),
                    &frag,
                );
                Ok(frag)
            }
        }
    }

    /// A child's text as it appears inside a concatenation (unions need parentheses).
    fn concat_part(&self, id: ReId) -> String {
        match self.r.arena.get(id) {
            Re::Union(_) => format!("({})", self.text(id)),
            _ => self.text(id),
        }
    }
}

/// Builds an NFA recognizing L(R).
pub fn regex_to_nfa(r: &RegexObj, budget: &Budget) -> Result<(Nfa, Trace), TlError> {
    let mut b = Builder {
        r,
        nfa: Nfa::empty(r.sigma.clone()),
        trace: Trace::new(budget),
        live_accept: BitSet::new(),
        budget,
    };
    let frag = b.build(r.root, 0)?;
    b.nfa.start = frag.start;
    b.nfa.accept = BitSet::of(frag.accept.iter().map(|q| q.index()));
    Ok((b.nfa, b.trace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use crate::ids::SymId;
    use crate::oracle;
    use crate::regex::ReArena;

    fn sigma_ab() -> Alphabet {
        Alphabet::from_names(&["a".into(), "b".into()])
    }

    #[test]
    fn example_1_56_has_eight_states() {
        // (ab ∪ a)*
        let mut arena = ReArena::new();
        let (a, b) = (arena.sym(SymId(0)), arena.sym(SymId(1)));
        let ab = arena.concat_raw(a, b);
        let u = arena.union_raw(ab, a);
        let root = arena.star_raw(u);
        let r = RegexObj {
            arena,
            root,
            sigma: sigma_ab(),
        };
        let (n, trace) = regex_to_nfa(&r, &Budget::default()).unwrap();
        assert_eq!(n.num_states(), 8);
        for w in oracle::all_strings(2, 6) {
            assert_eq!(
                oracle::nfa_accepts(&n, &w),
                oracle::regex_matches(&r.arena, root, &w),
                "{w:?}"
            );
        }
        // a, b, ab, a, ∪, *  → six construction steps
        assert_eq!(trace.steps.len(), 6);
    }

    #[test]
    fn example_1_58_has_twelve_states() {
        // (a ∪ b)*aba
        let mut arena = ReArena::new();
        let (a, b) = (arena.sym(SymId(0)), arena.sym(SymId(1)));
        let u = arena.union_raw(a, b);
        let s = arena.star_raw(u);
        let root = arena.concat_all(&[s, a, b, a]);
        let r = RegexObj {
            arena,
            root,
            sigma: sigma_ab(),
        };
        let (n, _) = regex_to_nfa(&r, &Budget::default()).unwrap();
        assert_eq!(n.num_states(), 12);
        for w in oracle::all_strings(2, 6) {
            assert_eq!(
                oracle::nfa_accepts(&n, &w),
                oracle::regex_matches(&r.arena, root, &w),
                "{w:?}"
            );
        }
    }

    #[test]
    fn empty_set_and_epsilon() {
        let arena = ReArena::new();
        let root = arena.empty();
        let r = RegexObj {
            arena,
            root,
            sigma: sigma_ab(),
        };
        let (n, _) = regex_to_nfa(&r, &Budget::default()).unwrap();
        assert_eq!(n.num_states(), 1);
        assert!(n.accept.is_empty());
    }
}
