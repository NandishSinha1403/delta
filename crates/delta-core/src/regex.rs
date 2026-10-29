//! Regular expressions: Definition 1.52.
//!
//! > R is a regular expression if R is a for some a in Σ, ε, ∅, (R₁ ∪ R₂), (R₁ ∘ R₂), or (R₁*).
//!
//! Expressions live in a hash-consed arena: each distinct sub-expression is stored once and named
//! by a `ReId`, so equality is O(1) and state elimination (which copies sub-expressions many
//! times) stays small in memory. Union and concatenation are n-ary (flattened).
//!
//! Two sets of constructors:
//! - `*_raw`: faithful, builds exactly what it is given (only flattening nested ∪ / ∘).
//! - plain: applies only the identities from p. 66 (R ∪ ∅ = R, Rε = R, R∅ = ∅, ∅* = ε, ε* = ε)
//!   plus idempotence R ∪ R = R and (R*)* = R*. Never R ∪ ε = R, which is false.

use crate::alphabet::Alphabet;
use crate::ids::SymId;
use rustc_hash::FxHashMap;

/// An expression's id inside its `ReArena`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReId(pub u32);

/// One node of a regular expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Re {
    /// ∅
    Empty,
    /// ε
    Eps,
    /// a ∈ Σ
    Sym(SymId),
    /// R₁ ∪ R₂ ∪ … (two or more children, none of them a Union)
    Union(Vec<ReId>),
    /// R₁ ∘ R₂ ∘ … (two or more children, none of them a Concat)
    Concat(Vec<ReId>),
    /// R*
    Star(ReId),
}

const EMPTY: ReId = ReId(0);
const EPS: ReId = ReId(1);

/// Storage for expressions; equal expressions get the same id.
#[derive(Debug, Clone)]
pub struct ReArena {
    nodes: Vec<Re>,
    dedup: FxHashMap<Re, ReId>,
}

impl Default for ReArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ReArena {
    /// A fresh arena (∅ and ε are pre-interned).
    pub fn new() -> Self {
        let mut arena = ReArena {
            nodes: Vec::new(),
            dedup: FxHashMap::default(),
        };
        arena.intern(Re::Empty);
        arena.intern(Re::Eps);
        arena
    }

    /// Number of distinct nodes stored (a size measure for budgets).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Always false: ∅ and ε are always present.
    pub fn is_empty(&self) -> bool {
        false
    }

    fn intern(&mut self, re: Re) -> ReId {
        if let Some(&id) = self.dedup.get(&re) {
            return id;
        }
        let id = ReId(self.nodes.len() as u32);
        self.nodes.push(re.clone());
        self.dedup.insert(re, id);
        id
    }

    /// The node behind an id.
    pub fn get(&self, id: ReId) -> &Re {
        self.nodes.get(id.0 as usize).unwrap_or(&Re::Empty)
    }

    /// ∅
    pub fn empty(&self) -> ReId {
        EMPTY
    }

    /// ε
    pub fn eps(&self) -> ReId {
        EPS
    }

    /// a
    pub fn sym(&mut self, a: SymId) -> ReId {
        self.intern(Re::Sym(a))
    }

    // ---------- faithful constructors ----------

    /// R₁ ∪ R₂, built literally.
    pub fn union_raw(&mut self, a: ReId, b: ReId) -> ReId {
        let mut kids = Vec::new();
        self.flatten_into(&mut kids, a, true);
        self.flatten_into(&mut kids, b, true);
        self.intern(Re::Union(kids))
    }

    /// R₁ ∘ R₂, built literally.
    pub fn concat_raw(&mut self, a: ReId, b: ReId) -> ReId {
        let mut kids = Vec::new();
        self.flatten_into(&mut kids, a, false);
        self.flatten_into(&mut kids, b, false);
        self.intern(Re::Concat(kids))
    }

    /// R*, built literally.
    pub fn star_raw(&mut self, a: ReId) -> ReId {
        self.intern(Re::Star(a))
    }

    // ---------- simplifying constructors (identities from p. 66) ----------

    /// R₁ ∪ R₂ with R ∪ ∅ = R and R ∪ R = R.
    pub fn union(&mut self, a: ReId, b: ReId) -> ReId {
        self.union_all(&[a, b])
    }

    /// The union of several expressions, simplified.
    pub fn union_all(&mut self, items: &[ReId]) -> ReId {
        let mut flat = Vec::new();
        for &x in items {
            self.flatten_into(&mut flat, x, true);
        }
        let mut kids: Vec<ReId> = Vec::new();
        for x in flat {
            if x != EMPTY && !kids.contains(&x) {
                kids.push(x);
            }
        }
        // `match kids.len() { 0 => …, 1 => …, _ => … }` picks a branch by the number of children.
        match kids.len() {
            0 => EMPTY,
            1 => kids.first().copied().unwrap_or(EMPTY),
            _ => self.intern(Re::Union(kids)),
        }
    }

    /// R₁ ∘ R₂ with Rε = R and R∅ = ∅.
    pub fn concat(&mut self, a: ReId, b: ReId) -> ReId {
        self.concat_all(&[a, b])
    }

    /// The concatenation of several expressions, simplified.
    pub fn concat_all(&mut self, items: &[ReId]) -> ReId {
        let mut flat = Vec::new();
        for &x in items {
            self.flatten_into(&mut flat, x, false);
        }
        if flat.contains(&EMPTY) {
            return EMPTY;
        }
        // `.retain(|x| cond)` keeps only the elements for which cond is true.
        flat.retain(|&x| x != EPS);
        match flat.len() {
            0 => EPS,
            1 => flat.first().copied().unwrap_or(EPS),
            _ => self.intern(Re::Concat(flat)),
        }
    }

    /// R* with ∅* = ε, ε* = ε and (R*)* = R*.
    pub fn star(&mut self, a: ReId) -> ReId {
        match self.get(a) {
            Re::Empty | Re::Eps => EPS,
            Re::Star(_) => a,
            _ => self.intern(Re::Star(a)),
        }
    }

    /// Appends `x` to `out`, splicing in its children if it is the same n-ary operator.
    fn flatten_into(&self, out: &mut Vec<ReId>, x: ReId, union: bool) {
        match (self.get(x), union) {
            (Re::Union(kids), true) | (Re::Concat(kids), false) => out.extend(kids.iter().copied()),
            _ => out.push(x),
        }
    }

    // ---------- printing ----------

    /// Prints with minimal parentheses by precedence (star > concatenation > union, p. 64):
    /// `a*b(a∪b)*`, never `a*(b((a∪b)*))`.
    pub fn print(&self, id: ReId, sigma: &Alphabet) -> String {
        let mut out = String::new();
        self.print_into(&mut out, id, sigma, 0);
        out
    }

    fn print_into(&self, out: &mut String, id: ReId, sigma: &Alphabet, ctx: u8) {
        // Precedence levels: 0 = union, 1 = concatenation, 2 = star / atom.
        let (prec, needs) = match self.get(id) {
            Re::Union(_) => (0, ctx > 0),
            Re::Concat(_) => (1, ctx > 1),
            _ => (2, false),
        };
        if needs {
            out.push('(');
        }
        match self.get(id) {
            Re::Empty => out.push('∅'),
            Re::Eps => out.push('ε'),
            Re::Sym(a) => out.push_str(&sigma.display(*a)),
            Re::Union(kids) => {
                for (i, &k) in kids.iter().enumerate() {
                    if i > 0 {
                        out.push('∪');
                    }
                    self.print_into(out, k, sigma, prec + 1);
                }
            }
            Re::Concat(kids) => {
                for &k in kids {
                    self.print_into(out, k, sigma, prec + 1);
                }
            }
            Re::Star(inner) => {
                // A starred star is written (R*)* like the book, not R**.
                let ctx = if matches!(self.get(*inner), Re::Star(_)) {
                    3
                } else {
                    2
                };
                if ctx == 3 {
                    out.push('(');
                    self.print_into(out, *inner, sigma, 2);
                    out.push(')');
                } else {
                    self.print_into(out, *inner, sigma, ctx);
                }
                out.push('*');
            }
        }
        if needs {
            out.push(')');
        }
    }

    /// Size of the expression tree (counting shared nodes once per use).
    pub fn size(&self, id: ReId) -> usize {
        match self.get(id) {
            Re::Empty | Re::Eps | Re::Sym(_) => 1,
            Re::Union(k) | Re::Concat(k) => 1 + k.iter().map(|&c| self.size(c)).sum::<usize>(),
            Re::Star(c) => 1 + self.size(*c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ab() -> (ReArena, Alphabet, ReId, ReId) {
        let sigma = Alphabet::from_names(&["a".to_string(), "b".to_string()]);
        let mut r = ReArena::new();
        let a = r.sym(SymId(0));
        let b = r.sym(SymId(1));
        (r, sigma, a, b)
    }

    #[test]
    fn prints_with_minimal_parentheses() {
        let (mut r, sigma, a, b) = ab();
        let a_star = r.star(a);
        let ab_union = r.union(a, b);
        let ab_star = r.star(ab_union);
        let e = r.concat_all(&[a_star, b, ab_star]);
        assert_eq!(r.print(e, &sigma), "a*b(a∪b)*");
        let ab_cat = r.concat(a, b);
        let u = r.union(ab_cat, a);
        let s = r.star(u);
        assert_eq!(r.print(s, &sigma), "(ab∪a)*");
    }

    #[test]
    fn simplifier_uses_only_safe_identities() {
        let (mut r, sigma, a, _) = ab();
        let eps = r.eps();
        let empty = r.empty();
        assert_eq!(r.union(a, empty), a);
        assert_eq!(r.concat(a, eps), a);
        assert_eq!(r.concat(a, empty), empty);
        assert_eq!(r.star(empty), eps);
        // R ∪ ε is NOT R.
        let u = r.union(a, eps);
        assert_eq!(r.print(u, &sigma), "a∪ε");
        let s = r.star(a);
        assert_eq!(r.star(s), s);
    }

    #[test]
    fn raw_constructors_keep_everything() {
        let (mut r, sigma, a, b) = ab();
        let eps = r.eps();
        let empty = r.empty();
        let ab_union = r.union_raw(a, b);
        let st = r.star_raw(ab_union);
        let cat = r.concat_raw(b, st);
        let cat = r.concat_raw(cat, eps);
        let full = r.union_raw(cat, empty);
        assert_eq!(r.print(full, &sigma), "b(a∪b)*ε∪∅");
        let ss = r.star_raw(st);
        assert_eq!(r.print(ss, &sigma), "((a∪b)*)*");
    }

    #[test]
    fn equal_expressions_share_an_id() {
        let (mut r, _, a, b) = ab();
        let x = r.union(a, b);
        let y = r.union(a, b);
        assert_eq!(x, y);
    }
}
