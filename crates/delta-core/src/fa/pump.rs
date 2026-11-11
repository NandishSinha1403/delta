//! `regular/pump`: splits a word w into x y z along the run of a DFA, at the first state the run
//! visits twice. With p states, that repeat happens within the first p symbols, so |xy| ≤ p and
//! |y| ≥ 1, and repeating or dropping y keeps the run ending in the same state.

use super::Fa;

pub const ID: &str = "regular/pump";

/// Where the run repeats: x = w[..i], y = w[i..j], z = w[j..]; `state` is visited at i and j.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Split {
    pub i: usize,
    pub j: usize,
    pub state: usize,
}

/// The split of `w` in the deterministic automaton `dfa`, or `None` if the run visits no state
/// twice (then |w| < number of states) or gets stuck on a missing transition.
// manual:begin pump-split
pub fn split(dfa: &Fa, w: &[u32]) -> Option<Split> {
    // `first[q]` is the position at which the run first reached q.
    let mut first: Vec<Option<usize>> = vec![None; dfa.n()];
    let mut q = dfa.start;
    for pos in 0..=w.len() {
        match first.get(q).copied().flatten() {
            Some(i) => {
                return Some(Split {
                    i,
                    j: pos,
                    state: q,
                });
            }
            None => *first.get_mut(q)? = Some(pos),
        }
        let a = *w.get(pos)?;
        q = dfa.next(q, a)?;
    }
    None
}
// manual:end pump-split

#[cfg(test)]
mod tests {
    use super::*;

    /// Words over {0, 1} ending in 1: q0 (start) and q1 (accepting).
    fn ends_in_1() -> Fa {
        let mut f = Fa::new(2, 2, 0);
        f.accept[1] = true;
        for q in 0..2 {
            f.add_edge(q, Some(0), 0);
            f.add_edge(q, Some(1), 1);
        }
        f
    }

    #[test]
    fn splits_at_the_first_repeated_state() {
        let f = ends_in_1();
        // Run 0 →(1) 1 →(1) 1: state 1 repeats at positions 1 and 2.
        assert_eq!(
            split(&f, &[1, 1]),
            Some(Split {
                i: 1,
                j: 2,
                state: 1
            })
        );
        // Run 0 →(0) 0: the start repeats at once.
        assert_eq!(
            split(&f, &[0, 1]),
            Some(Split {
                i: 0,
                j: 1,
                state: 0
            })
        );
    }

    #[test]
    fn short_words_have_no_repeat() {
        assert_eq!(split(&ends_in_1(), &[1]), None);
    }

    #[test]
    fn every_pumped_word_keeps_its_verdict() {
        let f = ends_in_1();
        let w = [0, 1, 0, 1, 1];
        let s = split(&f, &w).unwrap_or(Split {
            i: 0,
            j: 0,
            state: 0,
        });
        assert!(s.j - s.i >= 1 && s.j <= f.n());
        for k in 0..4 {
            let mut pumped: Vec<u32> = w[..s.i].to_vec();
            for _ in 0..k {
                pumped.extend_from_slice(&w[s.i..s.j]);
            }
            pumped.extend_from_slice(&w[s.j..]);
            assert!(f.accepts(&pumped), "k = {k}");
        }
    }
}
