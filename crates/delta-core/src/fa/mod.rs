//! Finite automata for Delta 2: index-based states `0..n`, symbols `0..k`, `None` = ε.
//! The language layer maps its state values and symbols to these indexes and back. Every
//! algorithm has a stable id (`determinize/subset`, …) and records neutral trace steps.

// Matrix-style algorithms read best with index loops.
#![allow(clippy::needless_range_loop)]

pub mod decide;
pub mod determinize;
pub mod minimize;
pub mod myhill_nerode;
pub mod ops;
pub mod product;
pub mod pump;
pub mod regex;
pub mod to_regex;

/// A finite automaton with one start state. `edges[q]` lists `(symbol or ε, target)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fa {
    /// Alphabet size: symbols are `0..k`.
    pub k: usize,
    pub start: usize,
    pub accept: Vec<bool>,
    pub edges: Vec<Vec<(Option<u32>, usize)>>,
}

/// One step of an algorithm's trace, with a neutral message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// `AddState`, `AddEdge`, `Closure`, `Merge`, `MarkPair`, `Rip`, …
    pub kind: &'static str,
    pub text: String,
}

impl Step {
    pub fn new(kind: &'static str, text: impl Into<String>) -> Step {
        Step {
            kind,
            text: text.into(),
        }
    }
}

impl Fa {
    /// An automaton with `n` states and no edges.
    pub fn new(k: usize, n: usize, start: usize) -> Fa {
        Fa {
            k,
            start,
            accept: vec![false; n],
            edges: vec![Vec::new(); n],
        }
    }

    pub fn n(&self) -> usize {
        self.edges.len()
    }

    pub fn add_state(&mut self, accepting: bool) -> usize {
        self.edges.push(Vec::new());
        self.accept.push(accepting);
        self.edges.len() - 1
    }

    pub fn add_edge(&mut self, from: usize, sym: Option<u32>, to: usize) {
        if let Some(list) = self.edges.get_mut(from)
            && !list.contains(&(sym, to))
        {
            list.push((sym, to));
        }
    }

    pub fn is_accept(&self, q: usize) -> bool {
        self.accept.get(q).copied().unwrap_or(false)
    }

    pub fn has_epsilon(&self) -> bool {
        self.edges.iter().flatten().any(|(s, _)| s.is_none())
    }

    /// Deterministic and total: exactly one edge per (state, symbol), no ε.
    pub fn is_dfa(&self) -> bool {
        self.edges.iter().all(|list| {
            let mut seen = vec![false; self.k];
            for (s, _) in list {
                match s {
                    None => return false,
                    Some(a) => match seen.get_mut(*a as usize) {
                        Some(slot) if !*slot => *slot = true,
                        _ => return false,
                    },
                }
            }
            seen.iter().all(|b| *b)
        })
    }

    /// The DFA successor of `q` on `a` (first matching edge).
    pub fn next(&self, q: usize, a: u32) -> Option<usize> {
        self.edges
            .get(q)?
            .iter()
            .find(|(s, _)| *s == Some(a))
            .map(|(_, t)| *t)
    }

    /// ε-closure of a set of states, as a sorted list (algorithm id `fa/eclose`).
    pub fn eclose(&self, set: &[usize]) -> Vec<usize> {
        let mut seen = vec![false; self.n()];
        let mut stack: Vec<usize> = Vec::new();
        for &q in set {
            if let Some(s) = seen.get_mut(q)
                && !*s
            {
                *s = true;
                stack.push(q);
            }
        }
        while let Some(q) = stack.pop() {
            for (sym, t) in self.edges.get(q).into_iter().flatten() {
                if sym.is_none()
                    && let Some(s) = seen.get_mut(*t)
                    && !*s
                {
                    *s = true;
                    stack.push(*t);
                }
            }
        }
        (0..self.n())
            .filter(|q| seen.get(*q).copied().unwrap_or(false))
            .collect()
    }

    /// The states reachable from `set` on symbol `a` (before ε-closure).
    pub fn step(&self, set: &[usize], a: u32) -> Vec<usize> {
        let mut out: Vec<usize> = set
            .iter()
            .flat_map(|q| self.edges.get(*q).into_iter().flatten())
            .filter(|(s, _)| *s == Some(a))
            .map(|(_, t)| *t)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn accepts(&self, w: &[u32]) -> bool {
        let mut cur = self.eclose(&[self.start]);
        for &a in w {
            cur = self.eclose(&self.step(&cur, a));
            if cur.is_empty() {
                return false;
            }
        }
        cur.iter().any(|q| self.is_accept(*q))
    }

    /// The same automaton over a larger alphabet `k2 ≥ k` (symbols keep their indexes).
    pub fn widen(&self, k2: usize) -> Fa {
        let mut f = self.clone();
        f.k = k2.max(self.k);
        f
    }
}

#[cfg(test)]
pub(crate) fn fa_from(
    k: usize,
    n: usize,
    start: usize,
    accept: &[usize],
    edges: &[(usize, Option<u32>, usize)],
) -> Fa {
    let mut f = Fa::new(k, n, start);
    for &q in accept {
        f.accept[q] = true;
    }
    for &(p, a, q) in edges {
        f.add_edge(p, a, q);
    }
    f
}
