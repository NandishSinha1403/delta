//! `cfg/ambiguity-bounded` (SEMANTICS §11.9): a bounded search for a word with two parse trees.
//! Ambiguity is undecidable, so this only ever looks at words up to a given length.
//!
//! For one word w, c(A, i, j) counts the parse trees of w[i..j] from A, capped at 2, as the least
//! fixed point of "sum over rules and splits of the product of the children's counts". It is
//! computed in rounds, each from the previous round's values, so a cycle A ⇒⁺ A (infinitely many
//! trees) also reaches 2. Remembering the round in which each count first reached 1 and 2 lets
//! us read off finite trees: a tree for a count reached in round r only uses children reached
//! before r.

use crate::grammar::{GSym, Grammar};
use crate::parse_tree::Tree;
use crate::value::Sym;

pub const ID: &str = "cfg/ambiguity-bounded";

/// The step budget ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfFuel;

/// The shortest ambiguous word (shortlex) with two of its parse trees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub word: Vec<Sym>,
    pub trees: (Tree, Tree),
}

/// The counts for one word: value, and the rounds in which it first reached 1 and 2.
struct Chart<'g> {
    g: &'g Grammar,
    w: &'g [Sym],
    n: usize,
    count: Vec<u8>,
    rank1: Vec<u32>,
    rank2: Vec<u32>,
}

const NEVER: u32 = u32::MAX;

/// One way to derive a span by a rule: the rule and the split points p₀ = i ≤ … ≤ pₖ = j.
type Combo = (usize, Vec<usize>);

impl<'g> Chart<'g> {
    fn ix(&self, a: usize, i: usize, j: usize) -> usize {
        (a * (self.n + 1) + i) * (self.n + 1) + j
    }

    /// Count of a symbol over w[i..j] in the current values (terminals: 1 on their own cell).
    fn sym_count(&self, s: &GSym, i: usize, j: usize) -> u8 {
        match s {
            GSym::T(t) => u8::from(j == i + 1 && self.w.get(i) == Some(t)),
            GSym::V(v) => self.count.get(self.ix(*v, i, j)).copied().unwrap_or(0),
        }
    }

    /// The round in which a symbol's count over w[i..j] first reached `level` (terminals: 0).
    fn sym_rank(&self, s: &GSym, i: usize, j: usize, level: u8) -> u32 {
        match s {
            GSym::T(_) if level == 1 && self.sym_count(s, i, j) == 1 => 0,
            GSym::T(_) => NEVER,
            GSym::V(v) => {
                let r = if level == 1 { &self.rank1 } else { &self.rank2 };
                r.get(self.ix(*v, i, j)).copied().unwrap_or(NEVER)
            }
        }
    }

    /// Number of ways (capped at 2) that `rhs` derives w[i..j] from the current values.
    fn ways(&self, rhs: &[GSym], i: usize, j: usize) -> u8 {
        // dp[p − i] = ways the symbols so far derive w[i..p].
        let mut dp = vec![0u8; j - i + 1];
        if let Some(d) = dp.first_mut() {
            *d = 1;
        }
        for s in rhs {
            let mut next = vec![0u8; j - i + 1];
            for (dp_off, &d) in dp.iter().enumerate() {
                if d == 0 {
                    continue;
                }
                let p = i + dp_off;
                for (q_off, slot) in next.iter_mut().enumerate().skip(dp_off) {
                    let c = self.sym_count(s, p, i + q_off);
                    if c > 0 {
                        *slot = (*slot + (d * c).min(2)).min(2);
                    }
                }
            }
            dp = next;
        }
        dp.last().copied().unwrap_or(0)
    }

    /// Up to `limit` combos for A over w[i..j] whose children all reached count 1 before round
    /// `before`.
    fn combos(&self, a: usize, i: usize, j: usize, before: u32, limit: usize) -> Vec<Combo> {
        /// The fixed parts of one search: the right side, the span end and the round bound.
        struct Walk<'a, 'g> {
            ch: &'a Chart<'g>,
            rhs: &'a [GSym],
            j: usize,
            before: u32,
            limit: usize,
        }
        // Depth-first over the cut points: symbol k starts at p.
        fn go(wk: &Walk, k: usize, p: usize, cuts: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
            if out.len() >= wk.limit {
                return;
            }
            let Some(s) = wk.rhs.get(k) else {
                if p == wk.j {
                    out.push(cuts.clone());
                }
                return;
            };
            for q in p..=wk.j {
                if wk.ch.sym_rank(s, p, q, 1) < wk.before {
                    cuts.push(q);
                    go(wk, k + 1, q, cuts, out);
                    cuts.pop();
                }
            }
        }
        let mut found = Vec::new();
        for (r, (l, rhs)) in self.g.rules.iter().enumerate() {
            if *l != a || found.len() >= limit {
                continue;
            }
            let mut out = Vec::new();
            let wk = Walk {
                ch: self,
                rhs,
                j,
                before,
                limit: limit - found.len(),
            };
            go(&wk, 0, i, &mut vec![i], &mut out);
            found.extend(out.into_iter().map(|cuts| (r, cuts)));
        }
        found
    }

    /// The node for rule `r` with the given cuts, children built by `child`.
    fn node(
        &self,
        (r, cuts): &Combo,
        child: &mut dyn FnMut(usize, &GSym, usize, usize) -> Tree,
    ) -> Tree {
        let (l, rhs) = self.g.rules.get(*r).cloned().unwrap_or((0, Vec::new()));
        let mut children: Vec<Tree> = rhs
            .iter()
            .enumerate()
            .map(|(k, s)| {
                let p = cuts.get(k).copied().unwrap_or(0);
                let q = cuts.get(k + 1).copied().unwrap_or(p);
                child(k, s, p, q)
            })
            .collect();
        if children.is_empty() {
            children.push(Tree {
                label: String::new(),
                var: false,
                children: Vec::new(),
            });
        }
        Tree {
            label: self.g.vars.get(l).cloned().unwrap_or_default(),
            var: true,
            children,
        }
    }

    /// One tree of a symbol over w[i..j] (its count must be ≥ 1).
    fn one(&self, s: &GSym, i: usize, j: usize) -> Tree {
        let a = match s {
            GSym::T(t) => {
                return Tree {
                    label: t.to_string(),
                    var: false,
                    children: Vec::new(),
                };
            }
            GSym::V(a) => *a,
        };
        let r1 = self.sym_rank(s, i, j, 1);
        match self.combos(a, i, j, r1, 1).first() {
            Some(c) => self.node(c, &mut |_, s, p, q| self.one(s, p, q)),
            None => Tree {
                label: self.g.vars.get(a).cloned().unwrap_or_default(),
                var: true,
                children: Vec::new(),
            },
        }
    }

    /// Two different trees of A over w[i..j] (its count must be 2).
    fn two(&self, a: usize, i: usize, j: usize) -> (Tree, Tree) {
        let r2 = self.sym_rank(&GSym::V(a), i, j, 2);
        let combos = self.combos(a, i, j, r2, 2);
        if let [c1, c2] = combos.as_slice() {
            let t1 = self.node(c1, &mut |_, s, p, q| self.one(s, p, q));
            let t2 = self.node(c2, &mut |_, s, p, q| self.one(s, p, q));
            return (t1, t2);
        }
        // One combo: a child of it already had two trees before round r2.
        if let Some(c) = combos.first() {
            let rhs = self
                .g
                .rules
                .get(c.0)
                .map(|(_, r)| r.clone())
                .unwrap_or_default();
            let twice = rhs.iter().enumerate().find_map(|(k, s)| {
                let p = c.1.get(k).copied().unwrap_or(0);
                let q = c.1.get(k + 1).copied().unwrap_or(p);
                match s {
                    GSym::V(v) if self.sym_rank(s, p, q, 2) < r2 => Some((k, *v, p, q)),
                    _ => None,
                }
            });
            if let Some((k2, v, p2, q2)) = twice {
                let (u1, u2) = self.two(v, p2, q2);
                let pick = |first: bool| {
                    let (u1, u2) = (u1.clone(), u2.clone());
                    self.node(c, &mut move |k, s, p, q| {
                        if k == k2 && p == p2 && q == q2 {
                            if first { u1.clone() } else { u2.clone() }
                        } else {
                            self.one(s, p, q)
                        }
                    })
                };
                return (pick(true), pick(false));
            }
        }
        let t = self.one(&GSym::V(a), i, j);
        (t.clone(), t)
    }
}

/// Counts the trees of every span of `w`, capped at 2. `fuel` is decreased per cell evaluated.
fn chart<'g>(g: &'g Grammar, w: &'g [Sym], fuel: &mut u64) -> Result<Chart<'g>, OutOfFuel> {
    let n = w.len();
    let cells = g.vars.len() * (n + 1) * (n + 1);
    let mut ch = Chart {
        g,
        w,
        n,
        count: vec![0; cells],
        rank1: vec![NEVER; cells],
        rank2: vec![NEVER; cells],
    };
    let mut round: u32 = 0;
    loop {
        round += 1;
        // manual:begin ambiguity-rounds
        // Each cell is the capped sum over its rules, computed from the previous round's values.
        let mut fresh = vec![0u8; cells];
        for (l, rhs) in &g.rules {
            for i in 0..=n {
                for j in i..=n {
                    *fuel = fuel.checked_sub(1).ok_or(OutOfFuel)?;
                    let ix = ch.ix(*l, i, j);
                    let w = ch.ways(rhs, i, j);
                    if let Some(c) = fresh.get_mut(ix) {
                        *c = (*c + w).min(2);
                    }
                }
            }
        }
        let mut changed = false;
        for (ix, (&old, &new)) in ch.count.iter().zip(&fresh).enumerate() {
            if new != old {
                changed = true;
                if new >= 1 && old < 1 {
                    ch.rank1[ix] = round;
                }
                if new >= 2 && old < 2 {
                    ch.rank2[ix] = round;
                }
            }
        }
        // manual:end ambiguity-rounds
        ch.count = fresh;
        if !changed {
            return Ok(ch);
        }
    }
}

/// The shortlex-first word of length ≤ `upto` with two parse trees in `g`, or `None`.
pub fn search(g: &Grammar, upto: usize, fuel: u64) -> Result<Option<Witness>, OutOfFuel> {
    let mut fuel = fuel;
    let terms: Vec<Sym> = g.terminals.iter().cloned().collect();
    let mut layer: Vec<Vec<Sym>> = vec![Vec::new()];
    for len in 0..=upto {
        for w in &layer {
            let ch = chart(g, w, &mut fuel)?;
            if ch.sym_count(&GSym::V(g.start), 0, len) >= 2 {
                return Ok(Some(Witness {
                    word: w.clone(),
                    trees: ch.two(g.start, 0, len),
                }));
            }
        }
        if len == upto || terms.is_empty() {
            break;
        }
        let mut next = Vec::new();
        for w in &layer {
            for t in &terms {
                fuel = fuel.checked_sub(1).ok_or(OutOfFuel)?;
                let mut x = w.clone();
                x.push(t.clone());
                next.push(x);
            }
        }
        layer = next;
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse;

    fn grammar(text: &str) -> Grammar {
        parse(None, text).unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None))
    }

    fn text(w: &[Sym]) -> String {
        w.concat()
    }

    #[test]
    fn finds_the_shortest_ambiguous_word_with_two_different_trees() {
        let g = grammar("E -> E + E | a");
        let found = search(&g, 6, 1_000_000).ok().flatten();
        assert!(found.is_some(), "no witness");
        let Some(Witness { word, trees }) = found else {
            return;
        };
        assert_eq!(text(&word), "a+a+a");
        assert_ne!(trees.0, trees.1);
        assert_eq!(trees.0.yield_word(), word);
        assert_eq!(trees.1.yield_word(), word);
    }

    #[test]
    fn unambiguous_grammars_have_no_witness_up_to_the_bound() {
        for t in ["S -> a S b | ε", "E -> E + T | T\nT -> a", "S -> a S | b"] {
            assert_eq!(search(&grammar(t), 7, 10_000_000), Ok(None), "{t}");
        }
    }

    #[test]
    fn cycles_and_epsilon_rules_count_as_ambiguity() {
        // A ⇒ A gives infinitely many trees of "a"; S → A A with A → ε gives two of "a".
        for (t, w) in [("S -> S | a", "a"), ("S -> A A\nA -> a | ε", "a")] {
            let found = search(&grammar(t), 3, 1_000_000).ok().flatten();
            assert!(found.is_some(), "{t}: no witness");
            let Some(Witness { word, trees }) = found else {
                return;
            };
            assert_eq!(text(&word), w, "{t}");
            assert_ne!(trees.0, trees.1, "{t}");
            assert_eq!(trees.0.yield_word(), word);
            assert_eq!(trees.1.yield_word(), word);
        }
    }

    #[test]
    fn the_budget_stops_the_search() {
        assert_eq!(search(&grammar("S -> a S b | ε"), 12, 50), Err(OutOfFuel));
    }
}
