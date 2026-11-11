//! Decision procedures for finite automata. They explore pairs of subsets breadth first with
//! symbols in order, so every witness is the shortlex-least word with the property.
//! Ids: `decide/equivalence`, `decide/inclusion`, `decide/emptiness`, `decide/finiteness`,
//! `decide/universality`, `decide/shortest`, `decide/words`, `decide/count`.

use super::{Fa, determinize, ops};
use std::collections::{HashMap, VecDeque};

/// The search ran out of its state budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exhausted;

type Pair = (Vec<usize>, Vec<usize>);

// manual:begin equivalence-search
/// Searches the product of the two subset automata for a pair where `bad(a accepts, b accepts)`;
/// returns the shortlex-least word reaching one, if any.
fn search(
    a: &Fa,
    b: &Fa,
    bad: &dyn Fn(bool, bool) -> bool,
    max: usize,
) -> Result<Option<Vec<u32>>, Exhausted> {
    let k = a.k.max(b.k) as u32;
    let start: Pair = (a.eclose(&[a.start]), b.eclose(&[b.start]));
    let mut parent: HashMap<Pair, Option<(Pair, u32)>> = HashMap::new();
    parent.insert(start.clone(), None);
    let mut queue = VecDeque::from([start]);
    while let Some(cur) = queue.pop_front() {
        let acc_a = cur.0.iter().any(|q| a.is_accept(*q));
        let acc_b = cur.1.iter().any(|q| b.is_accept(*q));
        if bad(acc_a, acc_b) {
            let mut word = Vec::new();
            let mut at = cur;
            while let Some(Some((prev, s))) = parent.get(&at).cloned() {
                word.push(s);
                at = prev;
            }
            word.reverse();
            return Ok(Some(word));
        }
        for s in 0..k {
            let next: Pair = (a.eclose(&a.step(&cur.0, s)), b.eclose(&b.step(&cur.1, s)));
            if !parent.contains_key(&next) {
                if parent.len() >= max {
                    return Err(Exhausted);
                }
                parent.insert(next.clone(), Some((cur.clone(), s)));
                queue.push_back(next);
            }
        }
    }
    Ok(None)
}
// manual:end equivalence-search

/// `None` if equivalent, else the shortest word in exactly one of them.
pub fn equivalence(a: &Fa, b: &Fa, max: usize) -> Result<Option<Vec<u32>>, Exhausted> {
    search(a, b, &|x, y| x != y, max)
}

/// `None` if L(a) ⊆ L(b), else the shortest word of L(a) ∖ L(b).
pub fn inclusion(a: &Fa, b: &Fa, max: usize) -> Result<Option<Vec<u32>>, Exhausted> {
    search(a, b, &|x, y| x && !y, max)
}

fn nothing(k: usize) -> Fa {
    Fa::new(k, 1, 0)
}

/// The shortest accepted word, if any.
pub fn shortest(a: &Fa, max: usize) -> Result<Option<Vec<u32>>, Exhausted> {
    search(a, &nothing(a.k), &|x, _| x, max)
}

pub fn is_empty(a: &Fa) -> bool {
    let c = ops::coreachable(a);
    !c.get(a.start).copied().unwrap_or(false)
}

/// `None` if every word is accepted, else the shortest rejected word.
pub fn universality(a: &Fa, max: usize) -> Result<Option<Vec<u32>>, Exhausted> {
    search(a, &nothing(a.k), &|x, _| !x, max)
}

/// Finite iff the useful part of the subset DFA (dropping the empty set) has no cycle.
pub fn is_finite(a: &Fa, max: usize) -> Result<bool, Exhausted> {
    let (d, _, _) = determinize::subset(a, max).map_err(|_| Exhausted)?;
    let (t, _) = ops::trim(&d);
    if is_empty(&t) {
        return Ok(true);
    }
    // Cycle detection by depth-first search with colors (0 new, 1 on the path, 2 done).
    let n = t.n();
    let mut color = vec![0u8; n];
    let mut stack: Vec<(usize, usize)> = vec![(t.start, 0)];
    color[t.start] = 1;
    while let Some(&mut (q, ref mut i)) = stack.last_mut() {
        let edges = t.edges.get(q).cloned().unwrap_or_default();
        if let Some(&(_, r)) = edges.get(*i) {
            *i += 1;
            match color.get(r).copied() {
                Some(1) => return Ok(false),
                Some(0) => {
                    color[r] = 1;
                    stack.push((r, 0));
                }
                _ => {}
            }
        } else {
            color[q] = 2;
            stack.pop();
        }
    }
    Ok(true)
}

/// Every accepted word of length ≤ `upto`, in shortlex order (at most `max` words).
pub fn words(a: &Fa, upto: usize, max: usize) -> Result<Vec<Vec<u32>>, Exhausted> {
    let mut out = Vec::new();
    let mut layer: Vec<(Vec<u32>, Vec<usize>)> = vec![(Vec::new(), a.eclose(&[a.start]))];
    let co = ops::coreachable(a);
    for len in 0..=upto {
        for (w, set) in &layer {
            if set.iter().any(|q| a.is_accept(*q)) {
                out.push(w.clone());
                if out.len() > max {
                    return Err(Exhausted);
                }
            }
        }
        if len == upto {
            break;
        }
        let mut next = Vec::new();
        for (w, set) in &layer {
            for s in 0..a.k as u32 {
                let t = a.eclose(&a.step(set, s));
                if t.iter().any(|q| co.get(*q).copied().unwrap_or(false)) {
                    let mut w2 = w.clone();
                    w2.push(s);
                    next.push((w2, t));
                    if next.len() > max {
                        return Err(Exhausted);
                    }
                }
            }
        }
        layer = next;
    }
    Ok(out)
}

/// The number of accepted words of length exactly `len` (saturating).
pub fn count(a: &Fa, len: usize, max: usize) -> Result<u128, Exhausted> {
    let (d, _, _) = determinize::subset(a, max).map_err(|_| Exhausted)?;
    let mut ways = vec![0u128; d.n()];
    ways[d.start] = 1;
    for _ in 0..len {
        let mut next = vec![0u128; d.n()];
        for (q, w) in ways.iter().enumerate() {
            if *w == 0 {
                continue;
            }
            for (_, t) in d.edges.get(q).into_iter().flatten() {
                next[*t] = next[*t].saturating_add(*w);
            }
        }
        ways = next;
    }
    Ok(ways
        .iter()
        .enumerate()
        .filter(|(q, _)| d.is_accept(*q))
        .fold(0u128, |acc, (_, w)| acc.saturating_add(*w)))
}

#[cfg(test)]
mod tests {
    use super::super::fa_from;
    use super::*;

    #[test]
    fn witnesses_are_shortlex_least() {
        let even_a = fa_from(
            2,
            2,
            0,
            &[0],
            &[
                (0, Some(0), 1),
                (1, Some(0), 0),
                (0, Some(1), 0),
                (1, Some(1), 1),
            ],
        );
        let all = fa_from(2, 1, 0, &[0], &[(0, Some(0), 0), (0, Some(1), 0)]);
        assert_eq!(equivalence(&even_a, &all, 100), Ok(Some(vec![0])));
        assert_eq!(equivalence(&even_a, &even_a, 100), Ok(None));
        assert_eq!(inclusion(&even_a, &all, 100), Ok(None));
        assert_eq!(universality(&even_a, 100), Ok(Some(vec![0])));
        assert_eq!(shortest(&even_a, 100), Ok(Some(vec![])));
        assert_eq!(is_finite(&even_a, 100), Ok(false));
        assert_eq!(count(&even_a, 2, 100), Ok(2)); // aa, bb
        assert_eq!(words(&even_a, 1, 100).map(|w| w.len()), Ok(2)); // "", "b"
        let one = fa_from(2, 2, 0, &[1], &[(0, Some(0), 1)]);
        assert_eq!(is_finite(&one, 100), Ok(true));
        assert!(!is_empty(&one));
    }
}
