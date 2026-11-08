//! `product/pairs`: the product of two DFAs over the same alphabet. A state is a pair; it
//! accepts when `keep(first accepts, second accepts)`. Only reachable pairs are built.

use super::Fa;
use std::collections::HashMap;

pub const ID: &str = "product/pairs";

pub fn product(
    a: &Fa,
    b: &Fa,
    keep: &dyn Fn(bool, bool) -> bool,
    max_states: usize,
) -> Result<(Fa, Vec<(usize, usize)>), usize> {
    let k = a.k.max(b.k);
    let mut pairs = vec![(a.start, b.start)];
    let mut index: HashMap<(usize, usize), usize> = HashMap::new();
    index.insert((a.start, b.start), 0);
    let mut f = Fa::new(k, 1, 0);
    f.accept[0] = keep(a.is_accept(a.start), b.is_accept(b.start));
    let mut i = 0;
    while let Some(&(p, q)) = pairs.get(i) {
        for s in 0..k as u32 {
            let (Some(p2), Some(q2)) = (a.next(p, s), b.next(q, s)) else {
                continue;
            };
            let j = match index.get(&(p2, q2)) {
                Some(&j) => j,
                None => {
                    if pairs.len() >= max_states {
                        return Err(max_states);
                    }
                    let j = f.add_state(keep(a.is_accept(p2), b.is_accept(q2)));
                    index.insert((p2, q2), j);
                    pairs.push((p2, q2));
                    j
                }
            };
            f.add_edge(i, Some(s), j);
        }
        i += 1;
    }
    Ok((f, pairs))
}
