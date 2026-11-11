//! `regular/myhill-nerode`: the classes of the right congruence of a regular language
//! (u ≡ v when uz and vz are both in L or both out of L, for every z). They are the states of the
//! minimal complete DFA. For each class: its shortlex-least word and whether it is accepting; for
//! each pair of classes: the shortlex-least suffix that tells them apart.

use super::Fa;
use std::collections::VecDeque;

pub const ID: &str = "regular/myhill-nerode";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classes {
    /// Per state of the minimal DFA: (shortlex-least word reaching it, accepting).
    pub classes: Vec<(Vec<u32>, bool)>,
    /// (p, q, z) for p < q: the shortlex-least z with exactly one of δ(p, z), δ(q, z) accepting.
    pub separators: Vec<(usize, usize, Vec<u32>)>,
}

/// Shortlex-least words to every reachable state: breadth-first, symbols in order.
fn least_words(dfa: &Fa) -> Vec<Option<Vec<u32>>> {
    let mut word: Vec<Option<Vec<u32>>> = vec![None; dfa.n()];
    if let Some(w) = word.get_mut(dfa.start) {
        *w = Some(Vec::new());
    }
    let mut queue = VecDeque::from([dfa.start]);
    while let Some(q) = queue.pop_front() {
        let here = word.get(q).cloned().flatten().unwrap_or_default();
        for a in 0..dfa.k as u32 {
            if let Some(t) = dfa.next(q, a)
                && word.get(t).is_some_and(|w| w.is_none())
            {
                let mut w = here.clone();
                w.push(a);
                word[t] = Some(w);
                queue.push_back(t);
            }
        }
    }
    word
}

/// The shortlex-least suffix separating p and q, by breadth-first search over pairs.
fn separator(dfa: &Fa, p: usize, q: usize) -> Option<Vec<u32>> {
    let n = dfa.n();
    let mut seen = vec![false; n * n];
    let mut queue = VecDeque::from([(p, q, Vec::new())]);
    seen[p * n + q] = true;
    while let Some((s, t, z)) = queue.pop_front() {
        if dfa.is_accept(s) != dfa.is_accept(t) {
            return Some(z);
        }
        for a in 0..dfa.k as u32 {
            if let (Some(s2), Some(t2)) = (dfa.next(s, a), dfa.next(t, a))
                && !seen[s2 * n + t2]
            {
                seen[s2 * n + t2] = true;
                let mut z2 = z.clone();
                z2.push(a);
                queue.push_back((s2, t2, z2));
            }
        }
    }
    None
}

/// The classes of a minimal, complete DFA (every state reachable, no two equivalent), or
/// `Err(max)` if it has more than `max` states.
pub fn classes(min_dfa: &Fa, max: usize) -> Result<Classes, usize> {
    let n = min_dfa.n();
    if n > max {
        return Err(max);
    }
    let words = least_words(min_dfa);
    let classes = (0..n)
        .map(|q| {
            (
                words.get(q).cloned().flatten().unwrap_or_default(),
                min_dfa.is_accept(q),
            )
        })
        .collect();
    let mut separators = Vec::new();
    for p in 0..n {
        for q in p + 1..n {
            if let Some(z) = separator(min_dfa, p, q) {
                separators.push((p, q, z));
            }
        }
    }
    Ok(Classes {
        classes,
        separators,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fa::minimize;

    /// Words over {a, b} ending in "ab", with a redundant copy of the start state (state 3).
    fn ends_ab() -> Fa {
        let mut f = Fa::new(2, 4, 0);
        f.accept[2] = true;
        for (p, a, q) in [
            (0, 0, 1),
            (0, 1, 3),
            (1, 0, 1),
            (1, 1, 2),
            (2, 0, 1),
            (2, 1, 0),
        ] {
            f.add_edge(p, Some(a), q);
        }
        f.add_edge(3, Some(0), 1);
        f.add_edge(3, Some(1), 3);
        f
    }

    #[test]
    fn three_classes_with_their_words_and_separators() {
        let (min, _, _) = minimize::hopcroft(&ends_ab());
        let c = classes(&min, 100).unwrap_or(Classes {
            classes: Vec::new(),
            separators: Vec::new(),
        });
        let mut words: Vec<(Vec<u32>, bool)> = c.classes.clone();
        words.sort_by(|x, y| (x.0.len(), &x.0).cmp(&(y.0.len(), &y.0)));
        assert_eq!(
            words,
            vec![(vec![], false), (vec![0], false), (vec![0, 1], true)]
        );
        // Every pair is separated, and the separator really separates.
        assert_eq!(c.separators.len(), 3);
        for (p, q, z) in &c.separators {
            let mut u = c.classes[*p].0.clone();
            let mut v = c.classes[*q].0.clone();
            u.extend(z);
            v.extend(z);
            assert_ne!(min.accepts(&u), min.accepts(&v));
        }
    }

    #[test]
    fn too_many_states_is_an_error() {
        let (min, _, _) = minimize::hopcroft(&ends_ab());
        assert!(classes(&min, 2).is_err());
    }
}
