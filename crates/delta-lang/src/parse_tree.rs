//! Parse trees and derivations (SEMANTICS §11.3–11.4):
//! - `parse/earley`: the tree is read off the Earley chart, trying each variable's rules in
//!   source order and split points from left to right.
//! - `parse/cyk`: the CYK table over a grammar in Chomsky normal form, with back-pointers.
//! - `derive`: the leftmost derivation of a tree.

use crate::grammar::{GSym, Grammar};
use crate::value::{Object, Sym, sym_text};
use delta_core::fa::Step;
// `HashSet` is an unordered set with fast lookups; `BTreeMap` is a map kept sorted by key.
use std::collections::{BTreeMap, HashSet};
// `Rc` is a reference-counted pointer: several owners share one value, freed when the last goes.
use std::rc::Rc;

/// The type tag of parse trees (after machines, languages, models and traces).
pub const TAG_TREE: u8 = 18;

// `#[derive(...)]` asks the compiler to write the listed traits (copying, printing, comparing).
/// A node: a variable with children, or a leaf (a terminal, or ε when `label` is empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    pub label: String,
    pub var: bool,
    pub children: Vec<Tree>,
}

// `impl Tree { … }` defines functions that belong to the type (methods).
impl Tree {
    fn leaf(label: String) -> Tree {
        Tree {
            label,
            var: false,
            children: Vec::new(),
        }
    }

    /// The word of the leaves, in order.
    pub fn yield_word(&self) -> Vec<Sym> {
        if !self.var {
            return if self.label.is_empty() {
                Vec::new()
            } else {
                vec![Sym::from(self.label.as_str())]
            };
        }
        self.children.iter().flat_map(Tree::yield_word).collect()
    }

    fn name(&self) -> String {
        if self.label.is_empty() {
            crate::notation::eps()
        } else if self.var && self.label.chars().count() > 1 {
            format!("<{}>", self.label)
        } else {
            self.label.clone()
        }
    }

    /// `S(a S(ε) b)`.
    pub fn bracketed(&self) -> String {
        if self.children.is_empty() {
            return self.name();
        }
        let kids: Vec<String> = self.children.iter().map(Tree::bracketed).collect();
        format!("{}({})", self.name(), kids.join(" "))
    }

    /// An indented drawing, one node per line.
    pub fn indented(&self) -> String {
        fn go(t: &Tree, prefix: &str, last: bool, top: bool, out: &mut Vec<String>) {
            let branch = if top {
                ""
            } else if last {
                "└─ "
            } else {
                "├─ "
            };
            out.push(format!("{prefix}{branch}{}", t.name()));
            let next = if top {
                String::new()
            } else {
                format!("{prefix}{}", if last { "   " } else { "│  " })
            };
            for (i, c) in t.children.iter().enumerate() {
                go(c, &next, i + 1 == t.children.len(), false, out);
            }
        }
        let mut out = Vec::new();
        go(self, "", true, true, &mut out);
        out.join("\n")
    }
}

/// A parse tree as a Delta value, with the steps that built it.
#[derive(Debug)]
pub struct ParseTree {
    pub tree: Tree,
    pub algorithm: &'static str,
    pub steps: Vec<Step>,
}

// `impl Object for ParseTree` makes parse trees Delta values: the trait lists what a value must do.
impl Object for ParseTree {
    fn tag(&self) -> u8 {
        TAG_TREE
    }
    fn key(&self) -> String {
        self.tree.bracketed()
    }
    fn type_name(&self) -> &'static str {
        "tree"
    }
    fn display(&self) -> String {
        self.tree.bracketed()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

/// The step budget ran out while parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfFuel;

/// The Earley tree of `w`, or Ok(None) if w ∉ L(G). Err if the fuel ran out.
pub fn earley(g: &Grammar, w: &[Sym], fuel: u64) -> Result<Option<Tree>, OutOfFuel> {
    if w.iter().any(|s| !g.terminals.contains(s)) {
        return Ok(None);
    }
    // `ok_or` turns an Option into a Result; `?` returns early with the error if there is one.
    let chart = g.earley_chart(w, fuel).ok_or(OutOfFuel)?;
    // Completed (variable, from, to) spans.
    let mut done: HashSet<(usize, usize, usize)> = HashSet::new();
    for (end, items) in chart.iter().enumerate() {
        for (r, dot, origin) in items {
            if let Some((l, rhs)) = g.rules.get(*r)
                && *dot == rhs.len()
            {
                done.insert((*l, *origin, end));
            }
        }
    }
    if !done.contains(&(g.start, 0, w.len())) {
        return Ok(None);
    }
    let mut budget = fuel;
    let mut path = HashSet::new();
    Ok(build(
        g,
        w,
        &done,
        g.start,
        0,
        w.len(),
        &mut path,
        &mut budget,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build(
    g: &Grammar,
    w: &[Sym],
    done: &HashSet<(usize, usize, usize)>,
    a: usize,
    i: usize,
    j: usize,
    path: &mut HashSet<(usize, usize, usize)>,
    budget: &mut u64,
) -> Option<Tree> {
    if *budget == 0 || !path.insert((a, i, j)) {
        return None;
    }
    *budget -= 1;
    let mut found = None;
    for (l, rhs) in &g.rules {
        if *l != a {
            continue;
        }
        if let Some(kids) = fit(g, w, done, rhs, 0, i, j, path, budget) {
            let children = if rhs.is_empty() {
                vec![Tree::leaf(String::new())]
            } else {
                kids
            };
            found = Some(Tree {
                label: g.vars.get(a).cloned().unwrap_or_default(),
                var: true,
                children,
            });
            break;
        }
    }
    path.remove(&(a, i, j));
    found
}

#[allow(clippy::too_many_arguments)]
fn fit(
    g: &Grammar,
    w: &[Sym],
    done: &HashSet<(usize, usize, usize)>,
    rhs: &[GSym],
    k: usize,
    pos: usize,
    j: usize,
    path: &mut HashSet<(usize, usize, usize)>,
    budget: &mut u64,
) -> Option<Vec<Tree>> {
    let Some(sym) = rhs.get(k) else {
        return (pos == j).then(Vec::new);
    };
    match sym {
        GSym::T(t) => {
            if w.get(pos) != Some(t) || pos >= j {
                return None;
            }
            let mut rest = fit(g, w, done, rhs, k + 1, pos + 1, j, path, budget)?;
            rest.insert(0, Tree::leaf(sym_text(t)));
            Some(rest)
        }
        GSym::V(b) => {
            for end in pos..=j {
                if !done.contains(&(*b, pos, end)) {
                    continue;
                }
                let Some(child) = build(g, w, done, *b, pos, end, path, budget) else {
                    continue;
                };
                if let Some(mut rest) = fit(g, w, done, rhs, k + 1, end, j, path, budget) {
                    rest.insert(0, child);
                    return Some(rest);
                }
            }
            None
        }
    }
}

/// How a CYK cell got a variable: a terminal rule, or a split at `k` into two variables.
#[derive(Clone, Copy, Debug)]
enum Back {
    Term,
    Split(usize, usize, usize),
}

/// CYK over `g`, which must be in Chomsky normal form. Returns the tree (or None) and one step
/// per filled cell.
pub fn cyk(g: &Grammar, w: &[Sym]) -> (Option<Tree>, Vec<Step>) {
    let n = w.len();
    let mut steps = Vec::new();
    let name = |v: usize| {
        let s = g.vars.get(v).cloned().unwrap_or_default();
        if s.chars().count() > 1 {
            format!("<{s}>")
        } else {
            s
        }
    };
    let var_tree = |v: usize, children: Vec<Tree>| Tree {
        label: g.vars.get(v).cloned().unwrap_or_default(),
        var: true,
        children,
    };
    if n == 0 {
        let ok = g.rules.iter().any(|(l, r)| *l == g.start && r.is_empty());
        steps.push(Step::new(
            "CykCell",
            format!(
                "the empty word: {}",
                if ok {
                    "S0 → ε is a rule"
                } else {
                    "no rule S0 → ε"
                }
            ),
        ));
        return (
            ok.then(|| var_tree(g.start, vec![Tree::leaf(String::new())])),
            steps,
        );
    }
    // table[i][len-1]: variables deriving w[i..i+len], each with its first back-pointer.
    let mut table: Vec<Vec<BTreeMap<usize, Back>>> = vec![vec![BTreeMap::new(); n]; n];
    for (i, row) in table.iter_mut().enumerate() {
        for (l, r) in &g.rules {
            if let [GSym::T(t)] = r.as_slice()
                && w.get(i) == Some(t)
                && let Some(cell) = row.first_mut()
            {
                cell.entry(*l).or_insert(Back::Term);
            }
        }
    }
    // manual:begin cyk-fill
    for len in 2..=n {
        for i in 0..=n - len {
            for k in 1..len {
                for (l, r) in &g.rules {
                    if let [GSym::V(b), GSym::V(c)] = r.as_slice()
                        && table[i][k - 1].contains_key(b)
                        && table[i + k][len - k - 1].contains_key(c)
                    {
                        table[i][len - 1]
                            .entry(*l)
                            .or_insert(Back::Split(k, *b, *c));
                    }
                }
            }
        }
    }
    // manual:end cyk-fill
    for len in 1..=n {
        for i in 0..=n - len {
            let cell = &table[i][len - 1];
            if !cell.is_empty() {
                let word: String = w[i..i + len].iter().map(|s| sym_text(s)).collect();
                let vars: Vec<String> = cell.keys().map(|v| name(*v)).collect();
                steps.push(Step::new(
                    "CykCell",
                    format!(
                        "\"{word}\" (positions {}–{}): {{{}}}",
                        i + 1,
                        i + len,
                        vars.join(", ")
                    ),
                ));
            }
        }
    }
    fn read(
        g: &Grammar,
        w: &[Sym],
        table: &[Vec<BTreeMap<usize, Back>>],
        v: usize,
        i: usize,
        len: usize,
    ) -> Tree {
        let label = g.vars.get(v).cloned().unwrap_or_default();
        match table
            .get(i)
            .and_then(|row| row.get(len - 1))
            .and_then(|c| c.get(&v))
        {
            Some(Back::Split(k, b, c)) => Tree {
                label,
                var: true,
                children: vec![
                    read(g, w, table, *b, i, *k),
                    read(g, w, table, *c, i + k, len - k),
                ],
            },
            _ => Tree {
                label,
                var: true,
                children: vec![Tree::leaf(
                    w.get(i).map(|s| sym_text(s)).unwrap_or_default(),
                )],
            },
        }
    }
    let tree = table[0][n - 1]
        .contains_key(&g.start)
        .then(|| read(g, w, &table, g.start, 0, n));
    (tree, steps)
}

/// The leftmost derivation of a tree: the sentential forms from the root to the yield.
pub fn derive(t: &Tree) -> Vec<Vec<Sym>> {
    // A sentential form is a list of nodes; expand the leftmost variable each time.
    let mut form: Vec<&Tree> = vec![t];
    let word = |form: &[&Tree]| -> Vec<Sym> {
        form.iter()
            .filter(|n| !n.label.is_empty())
            .map(|n| Sym::from(n.label.as_str()))
            .collect()
    };
    let mut out = vec![word(&form)];
    while let Some(i) = form.iter().position(|n| n.var) {
        let node = form.remove(i);
        for (k, c) in node.children.iter().enumerate() {
            form.insert(i + k, c);
        }
        out.push(word(&form));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse;

    fn w(s: &str) -> Vec<Sym> {
        s.chars()
            .map(|c| Sym::from(c.to_string().as_str()))
            .collect()
    }

    #[test]
    fn earley_and_cyk_trees_yield_the_word() {
        let g = parse(None, "S -> a S b | ε")
            .unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
        let t = earley(&g, &w("aabb"), 100_000).ok().flatten();
        assert_eq!(
            t.as_ref().map(Tree::bracketed).as_deref(),
            Some("S(a S(a S(ε) b) b)")
        );
        assert_eq!(t.as_ref().map(Tree::yield_word), Some(w("aabb")));
        assert_eq!(earley(&g, &w("aab"), 100_000), Ok(None));
        let c = crate::cnf::cnf(&g, 100_000)
            .unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
        let (k, steps) = cyk(&c, &w("aabb"));
        assert_eq!(k.map(|t| t.yield_word()), Some(w("aabb")));
        assert!(!steps.is_empty());
        assert!(cyk(&c, &w("abb")).0.is_none());
        assert!(cyk(&c, &[]).0.is_some());
    }

    #[test]
    fn leftmost_derivation() {
        let g = parse(None, "S -> a S b | ε")
            .unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
        let t = earley(&g, &w("ab"), 100_000).ok().flatten();
        let forms: Vec<String> = t
            .map(|t| derive(&t))
            .unwrap_or_default()
            .iter()
            .map(|f| f.iter().map(|s| sym_text(s)).collect())
            .collect();
        assert_eq!(forms, ["S", "aSb", "ab"]);
    }
}
