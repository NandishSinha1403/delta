//! C3 — regular expression → NFA (Lemma 1.55).
//!
//! Structural recursion over Definition 1.52 using *Sipser's* constructions (not Thompson's: they
//! differ, and the state counts must match the book's figures):
//! Faithful only: exactly the theorems' states and ε-arrows, with no ε-removal, merging,
//! simplification or minimization. States are named q0, q1, … left to right along the expression
//! (the start of `(0∪1)*0` is q0), and each theorem application is one trace step.
//!
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
use crate::nfa::{Label, Nfa, StateGroup};
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
    /// A new state, named q0, q1, … in the order states are created.
    fn state(&mut self, rule: &'static str) -> Result<StateId, TlError> {
        if self.nfa.num_states() as u32 >= self.budget.max_states {
            return Err(TlError::Budget {
                which: "max_states",
                limit: self.budget.max_states,
            });
        }
        let name = format!("q{}", self.nfa.num_states());
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

    /// Starts a layout group for sub-expression `id`; its states are filled in by `close`.
    fn open(&mut self, id: ReId, parent: Option<usize>) -> (usize, usize) {
        let label = self.text(id);
        self.nfa.groups.push(StateGroup {
            label,
            states: Vec::new(),
            parent,
        });
        (self.nfa.groups.len() - 1, self.nfa.num_states())
    }

    /// States are created in expression order, so a sub-expression's states are contiguous.
    fn close(&mut self, (group, first): (usize, usize)) {
        let states = (first..self.nfa.num_states())
            .map(StateId::from_index)
            .collect();
        if let Some(g) = self.nfa.groups.get_mut(group) {
            g.states = states;
        }
    }

    // States are numbered left to right along the expression: a union's or star's new start
    // state is created *before* the states of its operands (so (0∪1)* starts at q0), but it is
    // only added to the picture by the step that applies its theorem.
    fn build(&mut self, id: ReId, depth: usize, parent: Option<usize>) -> Result<Frag, TlError> {
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
                let sym = self.r.sigma.display(a);
                let msg = format!(
                    "R = {sym}: two states {} -{sym}→ {}, accepting {}.",
                    self.nfa.name(q1),
                    self.nfa.name(q2),
                    self.nfa.name(q2)
                );
                let step = Step::new("Lemma 1.55 (1)", msg)
                    .title(format!("Lemma 1.55 (1) · {sym}"))
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
                let step = Step::new("Lemma 1.55 (2)", msg)
                    .title("Lemma 1.55 (2) · ε")
                    .states(vec![q]);
                self.record(step, &frag);
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
                let step = Step::new("Lemma 1.55 (3)", msg)
                    .title("Lemma 1.55 (3) · ∅")
                    .states(vec![q]);
                self.record(step, &frag);
                Ok(frag)
            }
            Re::Union(kids) => {
                let group = self.open(id, parent);
                // R₁ ∪ R₂ ∪ R₃ folds left: ((R₁ ∪ R₂) ∪ R₃). The outermost union's start comes
                // first in the numbering, so the new starts are created outermost first.
                let mut starts = Vec::new();
                for _ in 1..kids.len() {
                    starts.push(self.state("Theorem 1.45")?);
                }
                let mut iter = kids.iter();
                let first = iter.next().copied().unwrap_or(id);
                let mut acc = self.build(first, depth + 1, Some(group.0))?;
                let mut acc_text = self.text(first);
                for (j, &k) in iter.enumerate() {
                    let right = self.build(k, depth + 1, Some(group.0))?;
                    let q0 = match starts.get(starts.len() - 1 - j) {
                        Some(&q) => q,
                        None => self.state("Theorem 1.45")?,
                    };
                    let (next, edges) = union_arrows(&mut self.nfa, q0, &acc, &right);
                    let joined = format!("{acc_text}∪{}", self.text(k));
                    let msg = format!(
                        "{} ∪ {}: new start {} with ε-arrows to {} and {}; F = F₁ ∪ F₂ = {}.",
                        acc_text,
                        self.text(k),
                        self.nfa.name(q0),
                        self.nfa.name(acc.start),
                        self.nfa.name(right.start),
                        self.names(&next.accept)
                    );
                    acc = next;
                    let step = Step::new("Theorem 1.45", msg)
                        .title(format!("Theorem 1.45 · {joined}"))
                        .states(vec![q0])
                        .edges(edges);
                    self.record(step, &acc);
                    acc_text = joined;
                }
                self.close(group);
                Ok(acc)
            }
            Re::Concat(kids) => {
                let mut iter = kids.iter();
                let first = iter.next().copied().unwrap_or(id);
                let mut acc = self.build(first, depth + 1, parent)?;
                let mut acc_text = self.concat_part(first);
                for &k in iter {
                    let right = self.build(k, depth + 1, parent)?;
                    let (next, edges) = concat_arrows(&mut self.nfa, &acc, &right);
                    for &f in &acc.accept {
                        self.live_accept.remove(f.index());
                    }
                    let joined = format!("{acc_text}{}", self.concat_part(k));
                    let msg = format!(
                        "{} ∘ {}: ε-arrows from F₁ = {} to {}; F = F₂ = {}.",
                        acc_text,
                        self.concat_part(k),
                        self.names(&acc.accept),
                        self.nfa.name(right.start),
                        self.names(&right.accept)
                    );
                    acc = next;
                    let focus = edges.iter().map(|e| e.0).chain([right.start]).collect();
                    let step = Step::new("Theorem 1.47", msg)
                        .title(format!("Theorem 1.47 · {joined}"))
                        .edges(edges)
                        .focus(focus);
                    self.record(step, &acc);
                    acc_text = joined;
                }
                Ok(acc)
            }
            Re::Star(inner) => {
                let group = self.open(id, parent);
                let q0 = self.state("Theorem 1.49")?;
                let body = self.build(inner, depth + 1, Some(group.0))?;
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
                let step = Step::new("Theorem 1.49", msg)
                    .title(format!("Theorem 1.49 · {}", self.text(id)))
                    .states(vec![q0])
                    .edges(edges);
                self.record(step, &frag);
                self.close(group);
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

/// Builds an NFA recognizing L(R), exactly as Lemma 1.55 does: no ε-removal, no merging, no
/// simplification. States are numbered q0, q1, … left to right along the expression.
pub fn regex_to_nfa(r: &RegexObj, budget: &Budget) -> Result<(Nfa, Trace), TlError> {
    let mut b = Builder {
        r,
        nfa: Nfa::empty(r.sigma.clone()),
        trace: Trace::new(budget),
        live_accept: BitSet::new(),
        budget,
    };
    let frag = b.build(r.root, 0, None)?;
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

    /// The book's construction for (0 ∪ 1)* 0 1 0*, state by state and arrow by arrow.
    #[test]
    fn golden_0_or_1_star_0_1_0_star() {
        let mut arena = ReArena::new();
        let (zero, one) = (arena.sym(SymId(0)), arena.sym(SymId(1)));
        let u = arena.union_raw(zero, one);
        let s = arena.star_raw(u);
        let z2 = arena.sym(SymId(0));
        let tail = arena.star_raw(z2);
        let a = arena.concat_raw(s, zero);
        let b = arena.concat_raw(a, one);
        let root = arena.concat_raw(b, tail);
        let sigma = Alphabet::from_names(&["0".into(), "1".into()]);
        let r = RegexObj { arena, root, sigma };
        let (n, trace) = regex_to_nfa(&r, &Budget::default()).unwrap();

        let names: Vec<String> = (0..n.num_states())
            .map(|i| n.name(StateId::from_index(i)).to_string())
            .collect();
        let expected_names: Vec<String> = (0..13).map(|i| format!("q{i}")).collect();
        assert_eq!(names, expected_names);
        assert_eq!(n.name(n.start), "q0");
        let accept: Vec<&str> = n
            .accept
            .to_vec()
            .into_iter()
            .map(|i| n.name(StateId::from_index(i)))
            .collect();
        assert_eq!(accept, ["q10", "q12"]);

        let mut arrows: Vec<String> = Vec::new();
        for q in 0..n.num_states() {
            let q = StateId::from_index(q);
            for &(label, to) in n.edges(q) {
                let l = match label {
                    Label::Eps => "ε".to_string(),
                    Label::Sym(a) => n.sigma.display(a),
                };
                arrows.push(format!("{}-{l}->{}", n.name(q), n.name(to)));
            }
        }
        arrows.sort();
        let mut expected: Vec<String> = [
            "q0-ε->q1",
            "q1-ε->q2",
            "q1-ε->q4",
            "q2-0->q3",
            "q4-1->q5",
            "q3-ε->q1",
            "q5-ε->q1",
            "q0-ε->q6",
            "q3-ε->q6",
            "q5-ε->q6",
            "q6-0->q7",
            "q7-ε->q8",
            "q8-1->q9",
            "q9-ε->q10",
            "q10-ε->q11",
            "q11-0->q12",
            "q12-ε->q11",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        expected.sort();
        assert_eq!(arrows, expected);

        // One step per theorem application: 0, 1, ∪, *, 0, ∘, 1, ∘, 0, *, ∘.
        let titles: Vec<String> = trace
            .steps
            .iter()
            .map(|s| s.title.clone().unwrap_or_default())
            .collect();
        assert_eq!(
            titles,
            [
                "Lemma 1.55 (1) · 0",
                "Lemma 1.55 (1) · 1",
                "Theorem 1.45 · 0∪1",
                "Theorem 1.49 · (0∪1)*",
                "Lemma 1.55 (1) · 0",
                "Theorem 1.47 · (0∪1)*0",
                "Lemma 1.55 (1) · 1",
                "Theorem 1.47 · (0∪1)*01",
                "Lemma 1.55 (1) · 0",
                "Theorem 1.49 · 0*",
                "Theorem 1.47 · (0∪1)*010*",
            ]
        );

        // Layout groups: (0∪1)* around q0…q5 containing 0∪1 (q1…q5), and 0* around q10…q12.
        let groups: Vec<(String, Vec<usize>, Option<usize>)> = n
            .groups
            .iter()
            .map(|g| {
                (
                    g.label.clone(),
                    g.states.iter().map(|q| q.index()).collect(),
                    g.parent,
                )
            })
            .collect();
        assert_eq!(
            groups,
            vec![
                ("(0∪1)*".to_string(), (0..6).collect(), None),
                ("0∪1".to_string(), (1..6).collect(), Some(0)),
                ("0*".to_string(), (10..13).collect(), None),
            ]
        );

        for w in oracle::all_strings(2, 8) {
            assert_eq!(
                oracle::nfa_accepts(&n, &w),
                oracle::regex_matches(&r.arena, root, &w),
                "{w:?}"
            );
        }
    }
}
