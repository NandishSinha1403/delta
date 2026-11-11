//! Small constructions on finite automata, each with its id:
//! `complete/dead-state`, `trim/useful`, `reverse/edges`, `union/epsilon`, `concat/epsilon`,
//! `star/epsilon`, `remove-epsilon/closure`, `complement/swap`.

use super::Fa;

/// `complete/dead-state`: adds one non-accepting state that every missing (state, symbol) goes
/// to (and that loops on everything). Returns the automaton and the dead state, if one was added.
pub fn complete(fa: &Fa) -> (Fa, Option<usize>) {
    let missing = (0..fa.n()).any(|q| (0..fa.k as u32).any(|a| fa.next(q, a).is_none()));
    if !missing {
        return (fa.clone(), None);
    }
    let mut f = fa.clone();
    let dead = f.add_state(false);
    for q in 0..f.n() {
        for a in 0..f.k as u32 {
            if f.next(q, a).is_none() {
                f.add_edge(q, Some(a), dead);
            }
        }
    }
    (f, Some(dead))
}

fn reachable_from(fa: &Fa, from: &[usize], forward: bool) -> Vec<bool> {
    let n = fa.n();
    let mut rev: Vec<Vec<usize>> = vec![Vec::new(); n];
    if !forward {
        for (p, list) in fa.edges.iter().enumerate() {
            for (_, q) in list {
                if let Some(r) = rev.get_mut(*q) {
                    r.push(p);
                }
            }
        }
    }
    let mut seen = vec![false; n];
    let mut stack: Vec<usize> = from.to_vec();
    for q in from {
        if let Some(s) = seen.get_mut(*q) {
            *s = true;
        }
    }
    while let Some(q) = stack.pop() {
        let next: Vec<usize> = if forward {
            fa.edges
                .get(q)
                .into_iter()
                .flatten()
                .map(|(_, t)| *t)
                .collect()
        } else {
            rev.get(q).cloned().unwrap_or_default()
        };
        for t in next {
            if let Some(s) = seen.get_mut(t)
                && !*s
            {
                *s = true;
                stack.push(t);
            }
        }
    }
    seen
}

/// States reachable from the start.
pub fn reachable(fa: &Fa) -> Vec<bool> {
    reachable_from(fa, &[fa.start], true)
}

/// States from which an accepting state is reachable.
pub fn coreachable(fa: &Fa) -> Vec<bool> {
    let acc: Vec<usize> = (0..fa.n()).filter(|q| fa.is_accept(*q)).collect();
    reachable_from(fa, &acc, false)
}

/// Keeps the states with `keep[q]` (the start is always kept). Returns the automaton and, for
/// each new state, its old index.
pub fn restrict(fa: &Fa, keep: &[bool]) -> (Fa, Vec<usize>) {
    let old: Vec<usize> = (0..fa.n())
        .filter(|q| *q == fa.start || keep.get(*q).copied().unwrap_or(false))
        .collect();
    let mut new_of = vec![usize::MAX; fa.n()];
    for (i, q) in old.iter().enumerate() {
        new_of[*q] = i;
    }
    let mut f = Fa::new(fa.k, old.len(), new_of[fa.start]);
    for (i, q) in old.iter().enumerate() {
        f.accept[i] = fa.is_accept(*q);
        for (a, t) in fa.edges.get(*q).into_iter().flatten() {
            if let Some(&nt) = new_of.get(*t)
                && nt != usize::MAX
            {
                f.add_edge(i, *a, nt);
            }
        }
    }
    (f, old)
}

/// `trim/useful`: keeps only states that are reachable and can reach an accepting state.
pub fn trim(fa: &Fa) -> (Fa, Vec<usize>) {
    let r = reachable(fa);
    let c = coreachable(fa);
    let keep: Vec<bool> = r.iter().zip(c.iter()).map(|(a, b)| *a && *b).collect();
    restrict(fa, &keep)
}

/// `reverse/edges`: reverses every edge; a new start state has ε-edges to the old accepting
/// states, and the old start state becomes the only accepting one.
pub fn reverse(fa: &Fa) -> Fa {
    let n = fa.n();
    let mut f = Fa::new(fa.k, n + 1, n);
    for (p, list) in fa.edges.iter().enumerate() {
        for (a, q) in list {
            f.add_edge(*q, *a, p);
        }
    }
    for q in 0..n {
        if fa.is_accept(q) {
            f.add_edge(n, None, q);
        }
    }
    f.accept[fa.start] = true;
    f
}

/// Copies `src` into `dst` with indexes shifted; returns the offset.
fn embed(dst: &mut Fa, src: &Fa) -> usize {
    let off = dst.n();
    for q in 0..src.n() {
        dst.add_state(false);
        let _ = q;
    }
    for (p, list) in src.edges.iter().enumerate() {
        for (a, q) in list {
            dst.add_edge(p + off, *a, q + off);
        }
    }
    off
}

/// `union/epsilon`: a new start state with ε-edges to both start states.
pub fn union(a: &Fa, b: &Fa) -> Fa {
    let k = a.k.max(b.k);
    let mut f = Fa::new(k, 1, 0);
    let oa = embed(&mut f, a);
    let ob = embed(&mut f, b);
    f.add_edge(0, None, a.start + oa);
    f.add_edge(0, None, b.start + ob);
    for q in 0..a.n() {
        f.accept[q + oa] = a.is_accept(q);
    }
    for q in 0..b.n() {
        f.accept[q + ob] = b.is_accept(q);
    }
    f
}

/// `concat/epsilon`: ε-edges from the first automaton's accepting states to the second's start.
pub fn concat(a: &Fa, b: &Fa) -> Fa {
    let k = a.k.max(b.k);
    let mut f = Fa::new(k, 0, 0);
    let oa = embed(&mut f, a);
    let ob = embed(&mut f, b);
    f.start = a.start + oa;
    for q in 0..a.n() {
        if a.is_accept(q) {
            f.add_edge(q + oa, None, b.start + ob);
        }
    }
    for q in 0..b.n() {
        f.accept[q + ob] = b.is_accept(q);
    }
    f
}

/// `star/epsilon`: a new accepting start state with an ε-edge to the old start, and ε-edges
/// from the accepting states back to the old start.
pub fn star(a: &Fa) -> Fa {
    let mut f = Fa::new(a.k, 1, 0);
    let oa = embed(&mut f, a);
    f.accept[0] = true;
    f.add_edge(0, None, a.start + oa);
    for q in 0..a.n() {
        if a.is_accept(q) {
            f.accept[q + oa] = true;
            f.add_edge(q + oa, None, a.start + oa);
        }
    }
    f
}

/// `remove-epsilon/closure`: q -a-> r whenever r is reachable from q by ε* a ε*; q accepts when
/// its ε-closure contains an accepting state. The states stay the same.
pub fn remove_epsilon(fa: &Fa) -> Fa {
    let n = fa.n();
    let mut f = Fa::new(fa.k, n, fa.start);
    for q in 0..n {
        let cl = fa.eclose(&[q]);
        f.accept[q] = cl.iter().any(|p| fa.is_accept(*p));
        for a in 0..fa.k as u32 {
            for r in fa.eclose(&fa.step(&cl, a)) {
                f.add_edge(q, Some(a), r);
            }
        }
    }
    f
}

/// `complement/swap`: for a DFA (deterministic and total), swaps accepting and non-accepting.
pub fn complement_dfa(dfa: &Fa) -> Fa {
    let mut f = dfa.clone();
    for a in f.accept.iter_mut() {
        *a = !*a;
    }
    f
}

#[cfg(test)]
mod tests {
    use super::super::fa_from;
    use super::*;

    #[test]
    fn constructions_agree_with_their_definitions() {
        let a = fa_from(2, 2, 0, &[1], &[(0, Some(0), 1)]); // "a"
        let b = fa_from(2, 2, 0, &[1], &[(0, Some(1), 1)]); // "b"
        assert!(union(&a, &b).accepts(&[1]));
        assert!(concat(&a, &b).accepts(&[0, 1]));
        assert!(!concat(&a, &b).accepts(&[0]));
        let s = star(&concat(&a, &b));
        assert!(s.accepts(&[]) && s.accepts(&[0, 1, 0, 1]) && !s.accepts(&[0, 1, 0]));
        let r = reverse(&concat(&a, &b));
        assert!(r.accepts(&[1, 0]) && !r.accepts(&[0, 1]));
        let e = remove_epsilon(&s);
        assert!(!e.has_epsilon() && e.accepts(&[0, 1]) && e.accepts(&[]));
        let (c, dead) = complete(&a);
        assert!(dead.is_some() && c.is_dfa());
        assert!(complement_dfa(&c).accepts(&[0, 0]));
        let (t, _) = trim(&c);
        assert_eq!(t.n(), 2);
    }
}
