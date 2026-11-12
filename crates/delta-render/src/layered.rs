//! `render/layered`: a layered drawing, left to right.
//!
//! 1. **Cycles.** A depth-first search from the start state marks the arrows that close a cycle;
//!    those are laid out as if reversed, then drawn the right way round.
//! 2. **Columns.** Each state goes in column 2·(longest path to it). Every arrow gets a label
//!    box in an odd column between its ends, and plain bend points in the other columns it
//!    crosses, so labels take part in the layout and never overlap.
//! 3. **Order.** Barycentre sweeps reduce crossings; the order with the fewest crossings wins.
//! 4. **Heights.** Each item moves towards the mean height of its neighbours, keeping the order
//!    and the minimum gaps (pool-adjacent-violators), sweeping both ways a few times.
//! 5. **Curves.** Each arrow is a smooth curve through its bend points; self-loops sit on top.

use crate::{
    Curve, Graph, LOOP_RISE, Layout, PlacedEdge, PlacedNode, Point, label_size, loop_height,
    node_radii,
};

const MARGIN: f64 = 14.0;
const START_ARROW: f64 = 30.0;
const COLUMN_GAP: f64 = 16.0;
const NODE_GAP: f64 = 22.0;
const SMALL_GAP: f64 = 8.0;
const SWEEPS: usize = 24;
const PLACE_PASSES: usize = 10;

/// What a layout item stands for.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    /// A state, by index.
    State(usize),
    /// A bend point of an arrow.
    Bend,
    /// The label box of an arrow, by edge index.
    Label(usize),
}

#[derive(Clone, Debug)]
struct Item {
    kind: Kind,
    column: usize,
    /// Width along x.
    w: f64,
    /// Extent above and below the centre.
    above: f64,
    below: f64,
    y: f64,
}

/// One arrow as a chain of items, from its lower column to its higher one.
struct Chain {
    edge: usize,
    items: Vec<usize>,
    reversed: bool,
}

/// Lays out a graph. Never fails: an empty graph gives an empty (margin-sized) picture.
pub fn layout(g: &Graph) -> Layout {
    let n = g.nodes.len();
    let radii: Vec<(f64, f64)> = g.nodes.iter().map(|s| node_radii(&s.name)).collect();
    let rx_of = |i: usize| radii.get(i).map_or(18.0, |r| r.0);
    let ry_of = |i: usize| radii.get(i).map_or(18.0, |r| r.1);

    // Self-loops are drawn on top of their state; they need room above it.
    let mut loop_room = vec![0.0f64; n];
    for e in &g.edges {
        if e.from == e.to && e.from < n {
            let (_, h) = label_size(&e.label);
            if let Some(room) = loop_room.get_mut(e.from) {
                *room = room.max(loop_height(ry_of(e.from)) - ry_of(e.from) + h + 4.0);
            }
        }
    }

    // 1. Arrows that close a cycle.
    let reversed = back_edges(g);

    // 2. Columns: longest path over the arrows as oriented.
    let mut rank = vec![0usize; n];
    let oriented: Vec<(usize, usize)> = g
        .edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.from != e.to && e.from < n && e.to < n)
        .map(|(i, e)| {
            if reversed.get(i).copied().unwrap_or(false) {
                (e.to, e.from)
            } else {
                (e.from, e.to)
            }
        })
        .collect();
    longest_path(n, &oriented, &mut rank);

    let mut items: Vec<Item> = (0..n)
        .map(|i| Item {
            kind: Kind::State(i),
            column: rank.get(i).copied().unwrap_or(0) * 2,
            w: 2.0 * rx_of(i),
            above: ry_of(i) + loop_room.get(i).copied().unwrap_or(0.0),
            below: ry_of(i),
            y: 0.0,
        })
        .collect();
    let mut chains: Vec<Chain> = Vec::new();
    for (i, e) in g.edges.iter().enumerate() {
        if e.from == e.to || e.from >= n || e.to >= n {
            continue;
        }
        let rev = reversed.get(i).copied().unwrap_or(false);
        let (a, b) = if rev { (e.to, e.from) } else { (e.from, e.to) };
        let (ca, cb) = (
            items.get(a).map_or(0, |x| x.column),
            items.get(b).map_or(0, |x| x.column),
        );
        let mut chain = vec![a];
        // The label sits in the odd column nearest the middle.
        let mut label_col = ca + (cb - ca) / 2;
        if label_col % 2 == 0 {
            label_col -= 1;
        }
        let (lw, lh) = label_size(&e.label);
        for col in ca + 1..cb {
            let item = if col == label_col {
                Item {
                    kind: Kind::Label(i),
                    column: col,
                    w: lw,
                    above: lh / 2.0,
                    below: lh / 2.0,
                    y: 0.0,
                }
            } else {
                Item {
                    kind: Kind::Bend,
                    column: col,
                    w: 0.0,
                    above: 2.0,
                    below: 2.0,
                    y: 0.0,
                }
            };
            chain.push(items.len());
            items.push(item);
        }
        chain.push(b);
        chains.push(Chain {
            edge: i,
            items: chain,
            reversed: rev,
        });
    }

    let columns = items.iter().map(|x| x.column + 1).max().unwrap_or(0);
    // Neighbours of each item in the column before and after.
    let mut prev: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    let mut next: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    for c in &chains {
        for pair in c.items.windows(2) {
            if let [a, b] = pair {
                if let Some(v) = next.get_mut(*a) {
                    v.push(*b);
                }
                if let Some(v) = prev.get_mut(*b) {
                    v.push(*a);
                }
            }
        }
    }

    // 3. Order within each column. Start from depth-first order of the states, chains after.
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); columns];
    for id in first_order(g, &chains, items.len()) {
        if let Some(col) = items.get(id).map(|x| x.column)
            && let Some(layer) = layers.get_mut(col)
        {
            layer.push(id);
        }
    }
    order_layers(&mut layers, &prev, &next, items.len());

    // 4. Heights.
    place(&mut items, &layers, &prev, &next);

    // x of each column: columns are as wide as their widest item.
    let mut col_w = vec![0.0f64; columns];
    for it in &items {
        if let Some(w) = col_w.get_mut(it.column) {
            *w = w.max(it.w);
        }
    }
    let has_start = g.nodes.iter().any(|s| s.start);
    let mut col_x = vec![0.0f64; columns];
    let mut x = MARGIN + if has_start { START_ARROW } else { 0.0 };
    for (c, w) in col_w.iter().enumerate() {
        if let Some(cx) = col_x.get_mut(c) {
            *cx = x + w / 2.0;
        }
        x += w + COLUMN_GAP;
    }
    let width = (x - COLUMN_GAP + MARGIN).max(2.0 * MARGIN);
    let top = items
        .iter()
        .map(|it| it.y - it.above)
        .fold(f64::INFINITY, f64::min);
    let bottom = items
        .iter()
        .map(|it| it.y + it.below)
        .fold(f64::NEG_INFINITY, f64::max);
    let (shift, height) = if top.is_finite() {
        (MARGIN - top, bottom - top + 2.0 * MARGIN)
    } else {
        (0.0, 2.0 * MARGIN)
    };
    let at = |id: usize| -> Point {
        items.get(id).map_or(Point::default(), |it| Point {
            x: col_x.get(it.column).copied().unwrap_or(0.0),
            y: it.y + shift,
        })
    };

    let nodes: Vec<PlacedNode> = (0..n)
        .map(|i| PlacedNode {
            at: at(i),
            rx: rx_of(i),
            ry: ry_of(i),
        })
        .collect();

    // 5. Curves.
    let mut edges: Vec<PlacedEdge> = g
        .edges
        .iter()
        .map(|e| {
            let (w, h) = label_size(&e.label);
            PlacedEdge {
                label_w: w,
                label_h: h,
                ..PlacedEdge::default()
            }
        })
        .collect();
    for c in &chains {
        let mut pts: Vec<Point> = c.items.iter().map(|&id| at(id)).collect();
        let label = c
            .items
            .iter()
            .find(|&&id| {
                items
                    .get(id)
                    .is_some_and(|it| it.kind == Kind::Label(c.edge))
            })
            .map(|&id| at(id));
        if c.reversed {
            pts.reverse();
        }
        let (from, to) = if c.reversed {
            (
                c.items.last().copied().unwrap_or(0),
                c.items.first().copied().unwrap_or(0),
            )
        } else {
            (
                c.items.first().copied().unwrap_or(0),
                c.items.last().copied().unwrap_or(0),
            )
        };
        clip_to_node(&mut pts, true, nodes.get(from));
        clip_to_node(&mut pts, false, nodes.get(to));
        if let Some(pe) = edges.get_mut(c.edge) {
            pe.curve = smooth(&pts);
            pe.label_at = label.unwrap_or_else(|| mid(&pe.curve));
        }
    }
    for (i, e) in g.edges.iter().enumerate() {
        if e.from != e.to {
            continue;
        }
        let Some(node) = nodes.get(e.from) else {
            continue;
        };
        if let Some(pe) = edges.get_mut(i) {
            pe.curve = self_loop(*node);
            pe.label_at = Point {
                x: node.at.x,
                y: node.at.y - loop_height(node.ry) - pe.label_h / 2.0 - 2.0,
            };
        }
    }
    let start = g
        .nodes
        .iter()
        .position(|s| s.start)
        .and_then(|i| nodes.get(i))
        .map(|node| {
            let tip = Point {
                x: node.at.x - node.rx,
                y: node.at.y,
            };
            let base = Point {
                x: tip.x - START_ARROW + 4.0,
                y: tip.y,
            };
            Curve {
                start: base,
                segments: vec![(lerp(base, tip, 1.0 / 3.0), lerp(base, tip, 2.0 / 3.0), tip)],
            }
        });
    Layout {
        width,
        height,
        nodes,
        edges,
        start,
    }
}

/// Marks the arrows that close a cycle in a depth-first search (start state first).
fn back_edges(g: &Graph) -> Vec<bool> {
    let n = g.nodes.len();
    let mut out: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for (i, e) in g.edges.iter().enumerate() {
        if e.from != e.to
            && e.to < n
            && let Some(v) = out.get_mut(e.from)
        {
            v.push((e.to, i));
        }
    }
    // 0 = not seen, 1 = on the current path, 2 = done.
    let mut state = vec![0u8; n];
    let mut back = vec![false; g.edges.len()];
    let starts = g.nodes.iter().position(|s| s.start).into_iter().chain(0..n);
    for root in starts {
        if state.get(root).copied() != Some(0) {
            continue;
        }
        // An explicit stack of (state, next arrow to look at), so deep graphs can't overflow.
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        if let Some(s) = state.get_mut(root) {
            *s = 1;
        }
        while let Some(top) = stack.last_mut() {
            let (u, k) = *top;
            top.1 += 1;
            match out.get(u).and_then(|v| v.get(k)).copied() {
                Some((v, i)) => match state.get(v).copied() {
                    Some(0) => {
                        if let Some(s) = state.get_mut(v) {
                            *s = 1;
                        }
                        stack.push((v, 0));
                    }
                    Some(1) => {
                        if let Some(b) = back.get_mut(i) {
                            *b = true;
                        }
                    }
                    _ => {}
                },
                None => {
                    if let Some(s) = state.get_mut(u) {
                        *s = 2;
                    }
                    stack.pop();
                }
            }
        }
    }
    back
}

/// rank[v] = length of the longest path to v over acyclic `arcs` (Kahn's order).
fn longest_path(n: usize, arcs: &[(usize, usize)], rank: &mut [usize]) {
    let mut indeg = vec![0usize; n];
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in arcs {
        if let (Some(d), Some(o)) = (indeg.get_mut(b), out.get_mut(a)) {
            *d += 1;
            o.push(b);
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&v| indeg.get(v) == Some(&0)).collect();
    let mut head = 0;
    while let Some(&u) = queue.get(head) {
        head += 1;
        let ru = rank.get(u).copied().unwrap_or(0);
        for &v in out.get(u).map(Vec::as_slice).unwrap_or(&[]) {
            if let Some(rv) = rank.get_mut(v) {
                *rv = (*rv).max(ru + 1);
            }
            if let Some(d) = indeg.get_mut(v) {
                *d -= 1;
                if *d == 0 {
                    queue.push(v);
                }
            }
        }
    }
}

/// States in depth-first order from the start, then each chain's inner items in arrow order.
fn first_order(g: &Graph, chains: &[Chain], total: usize) -> Vec<usize> {
    let n = g.nodes.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for e in &g.edges {
        if e.to < n
            && let Some(v) = adj.get_mut(e.from)
        {
            v.push(e.to);
        }
    }
    let mut seen = vec![false; n];
    let mut order = Vec::with_capacity(total);
    let roots = g.nodes.iter().position(|s| s.start).into_iter().chain(0..n);
    for root in roots {
        if seen.get(root).copied().unwrap_or(true) {
            continue;
        }
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            if seen.get(u).copied().unwrap_or(true) {
                continue;
            }
            if let Some(s) = seen.get_mut(u) {
                *s = true;
            }
            order.push(u);
            for &v in adj.get(u).map(Vec::as_slice).unwrap_or(&[]).iter().rev() {
                stack.push(v);
            }
        }
    }
    for c in chains {
        let inner = c.items.len().saturating_sub(1);
        order.extend(c.items.iter().take(inner).skip(1));
    }
    order
}

/// Barycentre sweeps; keeps the order with the fewest crossings.
fn order_layers(layers: &mut [Vec<usize>], prev: &[Vec<usize>], next: &[Vec<usize>], total: usize) {
    let mut pos = vec![0.0f64; total];
    let set_pos = |layers: &[Vec<usize>], pos: &mut Vec<f64>| {
        for layer in layers {
            for (i, &id) in layer.iter().enumerate() {
                if let Some(p) = pos.get_mut(id) {
                    *p = i as f64;
                }
            }
        }
    };
    set_pos(layers, &mut pos);
    let mut best = layers.to_vec();
    let mut best_cross = crossings(layers, next, &pos);
    // Big graphs get fewer sweeps; they are slow to count and rarely improve late.
    let sweeps = if total > 3000 { 4 } else { SWEEPS };
    for sweep in 0..sweeps {
        let down = sweep % 2 == 0;
        let cols: Vec<usize> = if down {
            (1..layers.len()).collect()
        } else {
            (0..layers.len().saturating_sub(1)).rev().collect()
        };
        for c in cols {
            let Some(layer) = layers.get_mut(c) else {
                continue;
            };
            let nbrs = if down { prev } else { next };
            let mut keyed: Vec<(f64, usize)> = layer
                .iter()
                .map(|&id| {
                    let ns = nbrs.get(id).map(Vec::as_slice).unwrap_or(&[]);
                    let own = pos.get(id).copied().unwrap_or(0.0);
                    let key = if ns.is_empty() {
                        own
                    } else {
                        ns.iter()
                            .map(|&m| pos.get(m).copied().unwrap_or(0.0))
                            .sum::<f64>()
                            / ns.len() as f64
                    };
                    (key, id)
                })
                .collect();
            // A stable sort keeps ties in their current order.
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
            *layer = keyed.into_iter().map(|(_, id)| id).collect();
            for (i, &id) in layer.iter().enumerate() {
                if let Some(p) = pos.get_mut(id) {
                    *p = i as f64;
                }
            }
        }
        let cross = crossings(layers, next, &pos);
        if cross < best_cross {
            best_cross = cross;
            best = layers.to_vec();
        }
        if best_cross == 0 {
            break;
        }
    }
    layers.clone_from_slice(&best);
}

/// Number of crossing pairs of arrows between neighbouring columns.
fn crossings(layers: &[Vec<usize>], next: &[Vec<usize>], pos: &[f64]) -> usize {
    let mut total = 0;
    for layer in layers {
        // Every arrow leaving this column as (from position, to position), sorted.
        let mut arcs: Vec<(usize, usize)> = Vec::new();
        for &id in layer {
            for &m in next.get(id).map(Vec::as_slice).unwrap_or(&[]) {
                let a = pos.get(id).copied().unwrap_or(0.0) as usize;
                let b = pos.get(m).copied().unwrap_or(0.0) as usize;
                arcs.push((a, b));
            }
        }
        arcs.sort_unstable();
        // Count inversions of the targets with a Fenwick tree.
        let size = arcs.iter().map(|a| a.1 + 1).max().unwrap_or(0);
        let mut tree = vec![0usize; size + 1];
        for (seen, &(_, b)) in arcs.iter().enumerate() {
            let mut not_greater = 0;
            let mut i = b + 1;
            while i > 0 {
                not_greater += tree.get(i).copied().unwrap_or(0);
                i &= i - 1;
            }
            total += seen - not_greater;
            let mut i = b + 1;
            while i <= size {
                if let Some(t) = tree.get_mut(i) {
                    *t += 1;
                }
                i += i.isolate_lowest_one();
            }
        }
    }
    total
}

/// Gives every item a height: towards its neighbours, in order, with minimum gaps.
fn place(items: &mut [Item], layers: &[Vec<usize>], prev: &[Vec<usize>], next: &[Vec<usize>]) {
    let gap = |items: &[Item], a: usize, b: usize| -> f64 {
        let (Some(x), Some(y)) = (items.get(a), items.get(b)) else {
            return 0.0;
        };
        let both_states = matches!(x.kind, Kind::State(_)) && matches!(y.kind, Kind::State(_));
        x.below + y.above + if both_states { NODE_GAP } else { SMALL_GAP }
    };
    // Start stacked from 0 in each column.
    for layer in layers {
        let mut y = 0.0;
        for (i, &id) in layer.iter().enumerate() {
            if i > 0
                && let Some(&before) = layer.get(i - 1)
            {
                y += gap(items, before, id);
            }
            if let Some(it) = items.get_mut(id) {
                it.y = y;
            }
        }
    }
    let weight = |items: &[Item], a: usize, b: usize| -> f64 {
        let straight = |k: Kind| !matches!(k, Kind::State(_));
        match (items.get(a).map(|x| x.kind), items.get(b).map(|x| x.kind)) {
            (Some(x), Some(y)) if straight(x) && straight(y) => 8.0,
            (Some(x), Some(y)) if straight(x) || straight(y) => 2.0,
            _ => 1.0,
        }
    };
    for pass in 0..PLACE_PASSES {
        let down = pass % 2 == 0;
        let last = pass + 1 == PLACE_PASSES;
        let cols: Vec<usize> = if down {
            (0..layers.len()).collect()
        } else {
            (0..layers.len()).rev().collect()
        };
        for c in cols {
            let Some(layer) = layers.get(c) else { continue };
            let wanted: Vec<f64> = layer
                .iter()
                .map(|&id| {
                    let mut ns: Vec<usize> = Vec::new();
                    if down || last {
                        ns.extend(prev.get(id).map(Vec::as_slice).unwrap_or(&[]));
                    }
                    if !down || last {
                        ns.extend(next.get(id).map(Vec::as_slice).unwrap_or(&[]));
                    }
                    let own = items.get(id).map_or(0.0, |x| x.y);
                    let (mut sum, mut wsum) = (0.0, 0.0);
                    for m in ns {
                        let w = weight(items, id, m);
                        sum += w * items.get(m).map_or(0.0, |x| x.y);
                        wsum += w;
                    }
                    if wsum > 0.0 { sum / wsum } else { own }
                })
                .collect();
            let gaps: Vec<f64> = layer
                .windows(2)
                .map(|p| match p {
                    [a, b] => gap(items, *a, *b),
                    _ => 0.0,
                })
                .collect();
            let ys = fit_in_order(&wanted, &gaps);
            for (&id, y) in layer.iter().zip(ys) {
                if let Some(it) = items.get_mut(id) {
                    it.y = y;
                }
            }
        }
    }
}

/// Least-squares heights closest to `wanted` with y[i+1] - y[i] >= gaps[i]
/// (pool-adjacent-violators on the gap-shifted values).
fn fit_in_order(wanted: &[f64], gaps: &[f64]) -> Vec<f64> {
    let mut offset = Vec::with_capacity(wanted.len());
    let mut acc = 0.0;
    for i in 0..wanted.len() {
        offset.push(acc);
        acc += gaps.get(i).copied().unwrap_or(0.0);
    }
    // Blocks of (mean, count) that must be non-decreasing.
    let mut blocks: Vec<(f64, usize)> = Vec::new();
    for (w, o) in wanted.iter().zip(&offset) {
        blocks.push((w - o, 1));
        while blocks.len() >= 2 {
            let (Some(&(m2, c2)), Some(&(m1, c1))) = (blocks.last(), blocks.get(blocks.len() - 2))
            else {
                break;
            };
            if m1 <= m2 {
                break;
            }
            blocks.pop();
            blocks.pop();
            let c = c1 + c2;
            blocks.push(((m1 * c1 as f64 + m2 * c2 as f64) / c as f64, c));
        }
    }
    let mut out = Vec::with_capacity(wanted.len());
    for (m, c) in blocks {
        for _ in 0..c {
            let o = offset.get(out.len()).copied().unwrap_or(0.0);
            out.push(m + o);
        }
    }
    out
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

/// Moves the first (or last) point from a state's centre to its outline.
fn clip_to_node(pts: &mut [Point], first: bool, node: Option<&PlacedNode>) {
    let len = pts.len();
    let Some(node) = node else { return };
    if len < 2 {
        return;
    }
    let (c, toward) = if first { (0, 1) } else { (len - 1, len - 2) };
    let Some(&other) = pts.get(toward) else {
        return;
    };
    if let Some(p) = pts.get_mut(c) {
        *p = node.boundary(other);
    }
}

/// A Catmull-Rom curve through the points, as cubic segments.
fn smooth(pts: &[Point]) -> Curve {
    let Some(&start) = pts.first() else {
        return Curve::default();
    };
    let mut segments = Vec::new();
    for i in 0..pts.len().saturating_sub(1) {
        let p1 = pts.get(i).copied().unwrap_or(start);
        let p2 = pts.get(i + 1).copied().unwrap_or(p1);
        let p0 = if i == 0 {
            p1
        } else {
            pts.get(i - 1).copied().unwrap_or(p1)
        };
        let p3 = pts.get(i + 2).copied().unwrap_or(p2);
        let c1 = Point {
            x: p1.x + (p2.x - p0.x) / 6.0,
            y: p1.y + (p2.y - p0.y) / 6.0,
        };
        let c2 = Point {
            x: p2.x - (p3.x - p1.x) / 6.0,
            y: p2.y - (p3.y - p1.y) / 6.0,
        };
        segments.push((c1, c2, p2));
    }
    Curve { start, segments }
}

/// The point halfway along the curve's middle segment.
fn mid(c: &Curve) -> Point {
    let k = c.segments.len() / 2;
    let p0 = if k == 0 {
        c.start
    } else {
        c.segments.get(k - 1).map_or(c.start, |s| s.2)
    };
    match c.segments.get(k) {
        Some(&(c1, c2, p3)) => Point {
            x: 0.125 * p0.x + 0.375 * c1.x + 0.375 * c2.x + 0.125 * p3.x,
            y: 0.125 * p0.y + 0.375 * c1.y + 0.375 * c2.y + 0.125 * p3.y,
        },
        None => p0,
    }
}

/// A loop of the same size on top of every state: it leaves and returns `k` px either side
/// of the top, and its controls rise `LOOP_RISE` px above the outline.
fn self_loop(node: PlacedNode) -> Curve {
    let k = (node.rx * 0.45).min(10.0);
    let h0 = node.ry * (1.0 - (k / node.rx).powi(2)).max(0.0).sqrt();
    let (x, y) = (node.at.x, node.at.y);
    let top = y - node.ry - LOOP_RISE;
    Curve {
        start: Point {
            x: x - k,
            y: y - h0,
        },
        segments: vec![(
            Point {
                x: x - k - 16.0,
                y: top,
            },
            Point {
                x: x + k + 16.0,
                y: top,
            },
            Point {
                x: x + k,
                y: y - h0,
            },
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node, dist};

    fn graph(n: usize, edges: &[(usize, usize, &str)]) -> Graph {
        Graph {
            title: "dfa".into(),
            nodes: (0..n)
                .map(|i| Node {
                    name: format!("q{i}"),
                    start: i == 0,
                    accept: i + 1 == n,
                    reject: false,
                })
                .collect(),
            edges: edges
                .iter()
                .map(|&(from, to, l)| Edge {
                    from,
                    to,
                    label: l.into(),
                })
                .collect(),
        }
    }

    #[test]
    fn a_chain_runs_left_to_right() {
        let g = graph(3, &[(0, 1, "a"), (1, 2, "b")]);
        let l = layout(&g);
        let xs: Vec<f64> = l.nodes.iter().map(|n| n.at.x).collect();
        assert!(xs[0] < xs[1] && xs[1] < xs[2], "{xs:?}");
        // On one line.
        assert!((l.nodes[0].at.y - l.nodes[2].at.y).abs() < 0.5);
        assert!(l.start.is_some());
    }

    #[test]
    fn a_back_arrow_keeps_its_direction() {
        let g = graph(2, &[(0, 1, "a"), (1, 0, "b")]);
        let l = layout(&g);
        let back = &l.edges[1].curve;
        // Starts near q1, ends near q0.
        assert!(dist(back.start, l.nodes[1].at) < dist(back.start, l.nodes[0].at));
        assert!(dist(back.end(), l.nodes[0].at) <= l.nodes[0].rx + 0.01);
        // The two labels don't overlap.
        let (a, b) = (&l.edges[0], &l.edges[1]);
        assert!((a.label_at.y - b.label_at.y).abs() >= (a.label_h + b.label_h) / 2.0);
    }

    #[test]
    fn states_in_a_column_do_not_overlap() {
        let g = graph(4, &[(0, 1, "a"), (0, 2, "b"), (0, 3, "c"), (1, 1, "x")]);
        let l = layout(&g);
        let mut ys: Vec<(f64, f64)> = l.nodes[1..].iter().map(|n| (n.at.y, n.ry)).collect();
        ys.sort_by(|a, b| a.0.total_cmp(&b.0));
        for w in ys.windows(2) {
            assert!(w[1].0 - w[0].0 >= w[0].1 + w[1].1, "{ys:?}");
        }
        assert!(l.nodes.iter().all(|n| n.at.y - n.ry >= 0.0));
        // The loop's label sits above the loop.
        let lp = &l.edges[3];
        assert!(lp.label_at.y + lp.label_h / 2.0 <= l.nodes[1].at.y - loop_height(l.nodes[1].ry));
    }

    #[test]
    fn gaps_are_kept_and_order_too() {
        let ys = fit_in_order(&[5.0, 0.0, 0.0], &[10.0, 10.0]);
        assert!(
            ys[1] - ys[0] >= 10.0 - 1e-9 && ys[2] - ys[1] >= 10.0 - 1e-9,
            "{ys:?}"
        );
    }

    #[test]
    fn crossings_are_counted() {
        // 0→3, 1→2 in columns [0,1] and [2,3] cross once.
        let layers = vec![vec![0, 1], vec![2, 3]];
        let next = vec![vec![3], vec![2], vec![], vec![]];
        let pos = vec![0.0, 1.0, 0.0, 1.0];
        assert_eq!(crossings(&layers, &next, &pos), 1);
    }

    #[test]
    fn empty_and_big_graphs_are_fine() {
        let l = layout(&Graph::default());
        assert!(l.width > 0.0 && l.nodes.is_empty());
        let edges: Vec<(usize, usize, &str)> = (0..300)
            .flat_map(|i| [(i, (i + 1) % 300, "a"), (i, (i * 7) % 300, "b")])
            .collect();
        let l = layout(&graph(300, &edges));
        assert!(l.width.is_finite() && l.height.is_finite());
    }
}
