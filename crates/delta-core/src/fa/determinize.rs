//! `determinize/subset`: the subset construction. Each new state is the ε-closed set of states
//! the automaton can be in; only sets reachable from the start set are built, breadth first,
//! symbols in order. The empty set appears when it is reachable (the result is total).

use super::{Fa, Step};

pub const ID: &str = "determinize/subset";

/// The DFA, the source-state set of each new state, and the trace.
pub type Subset = (Fa, Vec<Vec<usize>>, Vec<Step>);

/// The DFA, the source-state set of each new state, and the trace.
pub fn subset(fa: &Fa, max_states: usize) -> Result<Subset, usize> {
    subset_from(fa, &[fa.start], max_states)
}

// manual:begin subset
/// The subset construction from a set of start states.
pub fn subset_from(fa: &Fa, starts: &[usize], max_states: usize) -> Result<Subset, usize> {
    let mut sets: Vec<Vec<usize>> = Vec::new();
    // `HashMap` from a set (as a sorted Vec) to its new index.
    let mut index: std::collections::HashMap<Vec<usize>, usize> = std::collections::HashMap::new();
    let mut steps = Vec::new();
    let start = fa.eclose(starts);
    index.insert(start.clone(), 0);
    sets.push(start.clone());
    steps.push(Step::new(
        "AddState",
        format!("start: the ε-closure of the start state, {}", show(&start)),
    ));
    let mut dfa = Fa::new(fa.k, 1, 0);
    dfa.accept[0] = start.iter().any(|q| fa.is_accept(*q));
    let mut i = 0;
    while i < sets.len() {
        let cur = sets.get(i).cloned().unwrap_or_default();
        for a in 0..fa.k as u32 {
            let next = fa.eclose(&fa.step(&cur, a));
            let j = match index.get(&next) {
                Some(&j) => j,
                None => {
                    if sets.len() >= max_states {
                        return Err(max_states);
                    }
                    let j = dfa.add_state(next.iter().any(|q| fa.is_accept(*q)));
                    steps.push(Step::new("AddState", format!("new state {}", show(&next))));
                    index.insert(next.clone(), j);
                    sets.push(next);
                    j
                }
            };
            dfa.add_edge(i, Some(a), j);
            steps.push(Step::new(
                "AddEdge",
                format!(
                    "{} on symbol #{a} goes to {}",
                    show(&cur),
                    show(sets.get(j).map(|v| v.as_slice()).unwrap_or(&[]))
                ),
            ));
        }
        i += 1;
    }
    Ok((dfa, sets, steps))
}
// manual:end subset

fn show(set: &[usize]) -> String {
    let parts: Vec<String> = set.iter().map(|q| q.to_string()).collect();
    format!("{{{}}}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::super::fa_from;
    use super::*;

    #[test]
    fn ends_with_ab() {
        // NFA for (a|b)*ab: 0 -a,b-> 0, 0 -a-> 1, 1 -b-> 2
        let n = fa_from(
            2,
            3,
            0,
            &[2],
            &[
                (0, Some(0), 0),
                (0, Some(1), 0),
                (0, Some(0), 1),
                (1, Some(1), 2),
            ],
        );
        let (d, sets, _) = subset(&n, 100).unwrap_or_else(|_| (Fa::new(2, 1, 0), vec![], vec![]));
        assert!(d.is_dfa());
        assert_eq!(sets.len(), 3);
        for w in [vec![0, 1], vec![1, 0, 1], vec![0, 0, 1]] {
            assert!(d.accepts(&w));
        }
        assert!(!d.accepts(&[1, 0]));
    }
}
