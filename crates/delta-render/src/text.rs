//! A plain-text drawing for terminals without images: one line per arrow, grouped by state.
//!
//! ```text
//! dfa: 2 states (→ start, * accept)
//! → * q0 ──0──▶ q1
//!        ──1──▶ q0
//!     q1 ──0,1──▶ q1
//! ```

use crate::Graph;
use std::fmt::Write;

pub fn text(g: &Graph) -> String {
    let n = g.nodes.len();
    let mut s = format!(
        "{}: {n} state{} (→ start, * accept)\n",
        if g.title.is_empty() {
            "machine"
        } else {
            &g.title
        },
        if n == 1 { "" } else { "s" }
    );
    let width = g
        .nodes
        .iter()
        .map(|x| x.name.chars().count())
        .max()
        .unwrap_or(0);
    for (i, node) in g.nodes.iter().enumerate() {
        let mark = format!(
            "{} {}",
            if node.start { "→" } else { " " },
            if node.accept { "*" } else { " " }
        );
        let head = format!("{mark} {:>width$}", node.name);
        let blank = " ".repeat(head.chars().count());
        let out: Vec<_> = g.edges.iter().filter(|e| e.from == i).collect();
        if out.is_empty() {
            let _ = writeln!(s, "{}", head.trim_end());
        }
        for (k, e) in out.iter().enumerate() {
            let to = g.nodes.get(e.to).map_or("?", |x| x.name.as_str());
            let label = e.label.replace('\n', "; ");
            let _ = writeln!(
                s,
                "{} ──{label}──▶ {to}",
                if k == 0 { &head } else { &blank }
            );
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node};

    #[test]
    fn lists_arrows_by_state() {
        let g = Graph {
            title: "dfa".into(),
            nodes: vec![
                Node {
                    name: "q0".into(),
                    start: true,
                    accept: true,
                },
                Node {
                    name: "q1".into(),
                    start: false,
                    accept: false,
                },
            ],
            edges: vec![
                Edge {
                    from: 0,
                    to: 1,
                    label: "0".into(),
                },
                Edge {
                    from: 0,
                    to: 0,
                    label: "1".into(),
                },
                Edge {
                    from: 1,
                    to: 1,
                    label: "0,1".into(),
                },
            ],
        };
        assert_eq!(
            text(&g),
            "dfa: 2 states (→ start, * accept)\n→ * q0 ──0──▶ q1\n       ──1──▶ q0\n    q1 ──0,1──▶ q1\n"
        );
    }
}
