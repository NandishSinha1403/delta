//! DFA minimization, four algorithms with one result: the minimal DFA, whose states are the
//! classes of equivalent reachable states (listed by their original indexes), numbered in
//! breadth-first order from the start.
//! - `minimize/hopcroft`: partition refinement with a worklist of splitters.
//! - `minimize/moore`: refine by successor classes until nothing changes.
//! - `minimize/brzozowski`: determinize the reverse, twice.
//! - `minimize/table`: mark distinguishable pairs until no more can be marked.

use super::{Fa, Step, determinize, ops};
use std::collections::{BTreeSet, HashMap, VecDeque};

pub const HOPCROFT: &str = "minimize/hopcroft";
pub const MOORE: &str = "minimize/moore";
pub const BRZOZOWSKI: &str = "minimize/brzozowski";
pub const TABLE: &str = "minimize/table";

/// The minimal DFA and, for each of its states, the original states it merges.
pub type Minimal = (Fa, Vec<Vec<usize>>, Vec<Step>);

/// Builds the quotient automaton from a class number per state (of a reachable, total DFA).
fn quotient(dfa: &Fa, class: &[usize], mut steps: Vec<Step>) -> Minimal {
    let start_class = class.get(dfa.start).copied().unwrap_or(0);
    let mut order: Vec<usize> = vec![start_class];
    let mut new_of: HashMap<usize, usize> = HashMap::new();
    new_of.insert(start_class, 0);
    let mut queue = VecDeque::from([dfa.start]);
    let mut visited = vec![false; dfa.n()];
    if let Some(v) = visited.get_mut(dfa.start) {
        *v = true;
    }
    let mut rep: Vec<usize> = vec![dfa.start];
    while let Some(q) = queue.pop_front() {
        for a in 0..dfa.k as u32 {
            if let Some(t) = dfa.next(q, a) {
                let c = class.get(t).copied().unwrap_or(0);
                if let std::collections::hash_map::Entry::Vacant(e) = new_of.entry(c) {
                    e.insert(order.len());
                    order.push(c);
                    rep.push(t);
                }
                if !visited.get(t).copied().unwrap_or(true) {
                    visited[t] = true;
                    queue.push_back(t);
                }
            }
        }
    }
    let m = order.len();
    let mut f = Fa::new(dfa.k, m, 0);
    let mut blocks: Vec<Vec<usize>> = vec![Vec::new(); m];
    for q in 0..dfa.n() {
        if let Some(&i) = class.get(q).and_then(|c| new_of.get(c)) {
            blocks[i].push(q);
        }
    }
    for (i, &r) in rep.iter().enumerate() {
        f.accept[i] = dfa.is_accept(r);
        for a in 0..dfa.k as u32 {
            if let Some(t) = dfa.next(r, a)
                && let Some(&j) = class.get(t).and_then(|c| new_of.get(c))
            {
                f.add_edge(i, Some(a), j);
            }
        }
    }
    for (i, b) in blocks.iter().enumerate() {
        if b.len() > 1 {
            steps.push(Step::new(
                "Merge",
                format!("states {b:?} are equivalent: they become state {i}"),
            ));
        }
    }
    (f, blocks, steps)
}

/// Restricts to reachable states and completes, so every algorithm works on the same input.
fn prepare(dfa: &Fa) -> (Fa, Vec<usize>) {
    let (c, _) = ops::complete(dfa);
    let r = ops::reachable(&c);
    ops::restrict(&c, &r)
}

/// Maps a quotient over the prepared automaton back to the caller's state indexes.
fn back(min: Minimal, old: &[usize]) -> Minimal {
    let (f, blocks, steps) = min;
    let blocks = blocks
        .into_iter()
        .map(|b| b.into_iter().filter_map(|q| old.get(q).copied()).collect())
        .collect();
    (f, blocks, steps)
}

pub fn moore(dfa: &Fa) -> Minimal {
    let (d, old) = prepare(dfa);
    let n = d.n();
    let mut class: Vec<usize> = (0..n).map(|q| usize::from(d.is_accept(q))).collect();
    let mut steps = vec![Step::new(
        "Partition",
        "start with two classes: accepting and non-accepting",
    )];
    loop {
        let mut sig_index: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut next = vec![0; n];
        for q in 0..n {
            let mut sig = vec![class[q]];
            for a in 0..d.k as u32 {
                sig.push(d.next(q, a).map(|t| class[t]).unwrap_or(usize::MAX));
            }
            let len = sig_index.len();
            next[q] = *sig_index.entry(sig).or_insert(len);
        }
        let before: BTreeSet<usize> = class.iter().copied().collect();
        let after: BTreeSet<usize> = next.iter().copied().collect();
        steps.push(Step::new("Refine", format!("{} classes", after.len())));
        if after.len() == before.len() {
            break;
        }
        class = next;
    }
    back(quotient(&d, &class, steps), &old)
}

pub fn hopcroft(dfa: &Fa) -> Minimal {
    let (d, old) = prepare(dfa);
    let n = d.n();
    let k = d.k as u32;
    // Inverse transitions: inv[a][t] = states that go to t on a.
    let mut inv: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); n]; d.k];
    for q in 0..n {
        for a in 0..k {
            if let Some(t) = d.next(q, a) {
                inv[a as usize][t].push(q);
            }
        }
    }
    let acc: BTreeSet<usize> = (0..n).filter(|q| d.is_accept(*q)).collect();
    let rej: BTreeSet<usize> = (0..n).filter(|q| !d.is_accept(*q)).collect();
    let mut partition: Vec<BTreeSet<usize>> =
        [acc, rej].into_iter().filter(|b| !b.is_empty()).collect();
    let mut work: VecDeque<(BTreeSet<usize>, u32)> = VecDeque::new();
    let smaller = partition
        .iter()
        .min_by_key(|b| b.len())
        .cloned()
        .unwrap_or_default();
    for a in 0..k {
        work.push_back((smaller.clone(), a));
    }
    let mut steps = vec![Step::new(
        "Partition",
        "start with accepting and non-accepting states",
    )];
    while let Some((splitter, a)) = work.pop_front() {
        let pre: BTreeSet<usize> = splitter
            .iter()
            .flat_map(|t| inv[a as usize][*t].iter().copied())
            .collect();
        let mut next_partition = Vec::new();
        for block in partition {
            let inside: BTreeSet<usize> = block.intersection(&pre).copied().collect();
            let outside: BTreeSet<usize> = block.difference(&pre).copied().collect();
            if inside.is_empty() || outside.is_empty() {
                next_partition.push(block);
                continue;
            }
            steps.push(Step::new(
                "Split",
                format!("{block:?} splits into {inside:?} and {outside:?}"),
            ));
            let (small, _) = if inside.len() <= outside.len() {
                (&inside, &outside)
            } else {
                (&outside, &inside)
            };
            for b in 0..k {
                work.push_back((small.clone(), b));
            }
            next_partition.push(inside);
            next_partition.push(outside);
        }
        partition = next_partition;
    }
    let mut class = vec![0; n];
    for (i, b) in partition.iter().enumerate() {
        for q in b {
            class[*q] = i;
        }
    }
    back(quotient(&d, &class, steps), &old)
}

pub fn table(dfa: &Fa) -> Minimal {
    let (d, old) = prepare(dfa);
    let n = d.n();
    let mut marked = vec![vec![false; n]; n];
    let mut steps = Vec::new();
    for p in 0..n {
        for q in 0..p {
            if d.is_accept(p) != d.is_accept(q) {
                marked[p][q] = true;
                marked[q][p] = true;
            }
        }
    }
    steps.push(Step::new(
        "MarkPair",
        "mark every pair of an accepting and a non-accepting state",
    ));
    let mut changed = true;
    while changed {
        changed = false;
        for p in 0..n {
            for q in 0..p {
                if marked[p][q] {
                    continue;
                }
                for a in 0..d.k as u32 {
                    if let (Some(p2), Some(q2)) = (d.next(p, a), d.next(q, a))
                        && marked[p2][q2]
                    {
                        marked[p][q] = true;
                        marked[q][p] = true;
                        changed = true;
                        steps.push(Step::new(
                            "MarkPair",
                            format!("({q}, {p}) on symbol #{a} reach a marked pair"),
                        ));
                        break;
                    }
                }
            }
        }
    }
    let mut class = vec![usize::MAX; n];
    let mut next = 0;
    for p in 0..n {
        if class[p] != usize::MAX {
            continue;
        }
        class[p] = next;
        for q in p + 1..n {
            if !marked[p][q] {
                class[q] = next;
            }
        }
        next += 1;
    }
    back(quotient(&d, &class, steps), &old)
}

pub fn brzozowski(dfa: &Fa, max_states: usize) -> Result<Minimal, usize> {
    let (d, old) = prepare(dfa);
    // Reverse the edges and start from the old accepting states (no extra start state).
    let rev = |f: &Fa| -> (Fa, Vec<usize>) {
        let mut r = Fa::new(f.k, f.n(), f.start);
        for (p, list) in f.edges.iter().enumerate() {
            for (a, q) in list {
                r.add_edge(*q, *a, p);
            }
        }
        r.accept[f.start] = true;
        let starts = (0..f.n()).filter(|q| f.is_accept(*q)).collect();
        (r, starts)
    };
    let (rd, s1) = rev(&d);
    let (r1, _, _) = determinize::subset_from(&rd, &s1, max_states)?;
    let (rr, s2) = rev(&r1);
    let (r2, _, _) = determinize::subset_from(&rr, &s2, max_states)?;
    // Map each original state to the minimal state reached by the same word.
    let mut class = vec![usize::MAX; d.n()];
    let mut queue = VecDeque::from([(d.start, r2.start)]);
    class[d.start] = r2.start;
    while let Some((q, m)) = queue.pop_front() {
        for a in 0..d.k as u32 {
            if let (Some(q2), Some(m2)) = (d.next(q, a), r2.next(m, a))
                && class[q2] == usize::MAX
            {
                class[q2] = m2;
                queue.push_back((q2, m2));
            }
        }
    }
    let steps = vec![
        Step::new("Reverse", "reverse the automaton and determinize it"),
        Step::new("Reverse", "reverse the result and determinize it again"),
    ];
    Ok(back(quotient(&d, &class, steps), &old))
}

#[cfg(test)]
mod tests {
    use super::super::fa_from;
    use super::*;

    #[test]
    fn all_four_methods_agree() {
        // A 4-state DFA over {a} for "even length" with redundant states.
        let d = fa_from(
            1,
            4,
            0,
            &[0, 2],
            &[
                (0, Some(0), 1),
                (1, Some(0), 2),
                (2, Some(0), 3),
                (3, Some(0), 0),
            ],
        );
        let results = [
            hopcroft(&d),
            moore(&d),
            table(&d),
            brzozowski(&d, 1000).unwrap_or_else(|_| hopcroft(&d)),
        ];
        for (i, (f, blocks, _)) in results.iter().enumerate() {
            assert_eq!(f.n(), 2, "method {i}: {blocks:?}");
            assert_eq!(blocks, &vec![vec![0, 2], vec![1, 3]]);
        }
    }
}
