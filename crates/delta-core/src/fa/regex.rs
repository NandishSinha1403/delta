//! Regular expressions over symbol indexes, and three ways to build an NFA from one:
//! - `to_nfa/textbook`: one fragment per operator; union and star add a new start state,
//!   concatenation links accepting states to the next start with ε.
//! - `to_nfa/thompson`: fragments with exactly one start and one accepting state.
//! - `to_nfa/glushkov`: the position automaton, with no ε-edges.

use super::Fa;

pub const TEXTBOOK: &str = "to_nfa/textbook";
pub const THOMPSON: &str = "to_nfa/thompson";
pub const GLUSHKOV: &str = "to_nfa/glushkov";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Re {
    Empty,
    Eps,
    Sym(u32),
    Cat(Box<Re>, Box<Re>),
    Alt(Box<Re>, Box<Re>),
    Star(Box<Re>),
}

impl Re {
    /// Union with the identities ∅ ∪ r = r and r ∪ r = r.
    pub fn alt(a: Re, b: Re) -> Re {
        match (a, b) {
            (Re::Empty, r) | (r, Re::Empty) => r,
            (a, b) if a == b => a,
            (a, b) => Re::Alt(Box::new(a), Box::new(b)),
        }
    }
    /// Concatenation with ∅ · r = ∅ and ε · r = r.
    pub fn cat(a: Re, b: Re) -> Re {
        match (a, b) {
            (Re::Empty, _) | (_, Re::Empty) => Re::Empty,
            (Re::Eps, r) | (r, Re::Eps) => r,
            (a, b) => Re::Cat(Box::new(a), Box::new(b)),
        }
    }
    /// Star with ∅* = ε* = ε and (r*)* = r*.
    pub fn star(a: Re) -> Re {
        match a {
            Re::Empty | Re::Eps => Re::Eps,
            Re::Star(r) => Re::Star(r),
            r => Re::Star(Box::new(r)),
        }
    }

    /// `simplify/identities`: rebuilds with the identities above, bottom up.
    pub fn simplify(&self) -> Re {
        match self {
            Re::Cat(a, b) => Re::cat(a.simplify(), b.simplify()),
            Re::Alt(a, b) => Re::alt(a.simplify(), b.simplify()),
            Re::Star(a) => Re::star(a.simplify()),
            other => other.clone(),
        }
    }

    pub fn nullable(&self) -> bool {
        match self {
            Re::Empty | Re::Sym(_) => false,
            Re::Eps | Re::Star(_) => true,
            Re::Cat(a, b) => a.nullable() && b.nullable(),
            Re::Alt(a, b) => a.nullable() || b.nullable(),
        }
    }

    /// Text with the usual precedence (star, then concatenation, then union).
    pub fn text(&self, name: &dyn Fn(u32) -> String) -> String {
        fn go(r: &Re, name: &dyn Fn(u32) -> String, prec: u8) -> String {
            match r {
                Re::Empty => "∅".to_string(),
                Re::Eps => "ε".to_string(),
                Re::Sym(a) => name(*a),
                Re::Star(a) => format!("{}*", go(a, name, 3)),
                Re::Cat(a, b) => {
                    let s = format!("{} {}", go(a, name, 2), go(b, name, 2));
                    if prec > 2 { format!("({s})") } else { s }
                }
                Re::Alt(a, b) => {
                    let s = format!("{} ∪ {}", go(a, name, 1), go(b, name, 1));
                    if prec > 1 { format!("({s})") } else { s }
                }
            }
        }
        go(self, name, 0)
    }
}

/// A fragment of the textbook construction: start state and accepting states.
fn textbook_frag(f: &mut Fa, r: &Re) -> (usize, Vec<usize>) {
    match r {
        Re::Empty => (f.add_state(false), Vec::new()),
        Re::Eps => {
            let s = f.add_state(false);
            (s, vec![s])
        }
        Re::Sym(a) => {
            let s = f.add_state(false);
            let t = f.add_state(false);
            f.add_edge(s, Some(*a), t);
            (s, vec![t])
        }
        Re::Alt(a, b) => {
            let s = f.add_state(false);
            let (sa, fa) = textbook_frag(f, a);
            let (sb, fb) = textbook_frag(f, b);
            f.add_edge(s, None, sa);
            f.add_edge(s, None, sb);
            (s, fa.into_iter().chain(fb).collect())
        }
        Re::Cat(a, b) => {
            let (sa, fa) = textbook_frag(f, a);
            let (sb, fb) = textbook_frag(f, b);
            for q in fa {
                f.add_edge(q, None, sb);
            }
            (sa, fb)
        }
        Re::Star(a) => {
            let s = f.add_state(false);
            let (sa, fa) = textbook_frag(f, a);
            f.add_edge(s, None, sa);
            for q in &fa {
                f.add_edge(*q, None, sa);
            }
            let mut acc = fa;
            acc.push(s);
            (s, acc)
        }
    }
}

pub fn to_nfa_textbook(r: &Re, k: usize) -> Fa {
    let mut f = Fa::new(k, 0, 0);
    let (s, acc) = textbook_frag(&mut f, r);
    f.start = s;
    for q in acc {
        if let Some(a) = f.accept.get_mut(q) {
            *a = true;
        }
    }
    f
}

fn thompson_frag(f: &mut Fa, r: &Re) -> (usize, usize) {
    match r {
        Re::Empty => (f.add_state(false), f.add_state(false)),
        Re::Eps | Re::Sym(_) => {
            let s = f.add_state(false);
            let t = f.add_state(false);
            let sym = if let Re::Sym(a) = r { Some(*a) } else { None };
            f.add_edge(s, sym, t);
            (s, t)
        }
        Re::Cat(a, b) => {
            let (sa, ta) = thompson_frag(f, a);
            let (sb, tb) = thompson_frag(f, b);
            f.add_edge(ta, None, sb);
            (sa, tb)
        }
        Re::Alt(a, b) => {
            let s = f.add_state(false);
            let (sa, ta) = thompson_frag(f, a);
            let (sb, tb) = thompson_frag(f, b);
            let t = f.add_state(false);
            f.add_edge(s, None, sa);
            f.add_edge(s, None, sb);
            f.add_edge(ta, None, t);
            f.add_edge(tb, None, t);
            (s, t)
        }
        Re::Star(a) => {
            let s = f.add_state(false);
            let (sa, ta) = thompson_frag(f, a);
            let t = f.add_state(false);
            f.add_edge(s, None, sa);
            f.add_edge(s, None, t);
            f.add_edge(ta, None, sa);
            f.add_edge(ta, None, t);
            (s, t)
        }
    }
}

pub fn to_nfa_thompson(r: &Re, k: usize) -> Fa {
    let mut f = Fa::new(k, 0, 0);
    let (s, t) = thompson_frag(&mut f, r);
    f.start = s;
    if let Some(a) = f.accept.get_mut(t) {
        *a = true;
    }
    f
}

/// Positions of a regex: (first, last, follow, symbol of each position, nullable).
struct Positions {
    syms: Vec<u32>,
    follow: Vec<Vec<usize>>,
}

fn glushkov_sets(r: &Re, p: &mut Positions) -> (Vec<usize>, Vec<usize>, bool) {
    match r {
        Re::Empty => (Vec::new(), Vec::new(), false),
        Re::Eps => (Vec::new(), Vec::new(), true),
        Re::Sym(a) => {
            let i = p.syms.len();
            p.syms.push(*a);
            p.follow.push(Vec::new());
            (vec![i], vec![i], false)
        }
        Re::Alt(a, b) => {
            let (fa, la, na) = glushkov_sets(a, p);
            let (fb, lb, nb) = glushkov_sets(b, p);
            (cat_sets(fa, fb), cat_sets(la, lb), na || nb)
        }
        Re::Cat(a, b) => {
            let (fa, la, na) = glushkov_sets(a, p);
            let (fb, lb, nb) = glushkov_sets(b, p);
            for &x in &la {
                for &y in &fb {
                    if let Some(f) = p.follow.get_mut(x)
                        && !f.contains(&y)
                    {
                        f.push(y);
                    }
                }
            }
            let first = if na { cat_sets(fa, fb.clone()) } else { fa };
            let last = if nb { cat_sets(la, lb) } else { lb };
            (first, last, na && nb)
        }
        Re::Star(a) => {
            let (fa, la, _) = glushkov_sets(a, p);
            for &x in &la {
                for &y in &fa {
                    if let Some(f) = p.follow.get_mut(x)
                        && !f.contains(&y)
                    {
                        f.push(y);
                    }
                }
            }
            (fa, la, true)
        }
    }
}

fn cat_sets(mut a: Vec<usize>, b: Vec<usize>) -> Vec<usize> {
    for x in b {
        if !a.contains(&x) {
            a.push(x);
        }
    }
    a
}

pub fn to_nfa_glushkov(r: &Re, k: usize) -> Fa {
    let mut p = Positions {
        syms: Vec::new(),
        follow: Vec::new(),
    };
    let (first, last, nullable) = glushkov_sets(r, &mut p);
    let n = p.syms.len() + 1;
    let mut f = Fa::new(k, n, 0);
    f.accept[0] = nullable;
    for &x in &last {
        f.accept[x + 1] = true;
    }
    for &y in &first {
        f.add_edge(0, p.syms.get(y).copied(), y + 1);
    }
    for (x, ys) in p.follow.iter().enumerate() {
        for &y in ys {
            f.add_edge(x + 1, p.syms.get(y).copied(), y + 1);
        }
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(a: u32) -> Re {
        Re::Sym(a)
    }

    #[test]
    fn three_constructions_agree() {
        // (0 ∪ 1)* 0 1 0*
        let r = Re::cat(
            Re::cat(Re::cat(Re::star(Re::alt(sym(0), sym(1))), sym(0)), sym(1)),
            Re::star(sym(0)),
        );
        let fs = [
            to_nfa_textbook(&r, 2),
            to_nfa_thompson(&r, 2),
            to_nfa_glushkov(&r, 2),
        ];
        let words: [&[u32]; 6] = [
            &[0, 1],
            &[1, 0, 1, 0, 0],
            &[0, 1, 0],
            &[],
            &[1, 0],
            &[0, 1, 1],
        ];
        let expect = [true, true, true, false, false, false];
        for f in &fs {
            for (w, e) in words.iter().zip(expect) {
                assert_eq!(f.accepts(w), e, "{w:?}");
            }
        }
        assert!(!fs[2].has_epsilon());
    }
}
