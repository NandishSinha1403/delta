//! A plain-text drawing for terminals without images: one line per arrow, grouped by state.
//!
//! ```text
//! dfa: 2 states (→ start, * accept)   (a Turing machine adds ", ✗ reject")
//! → * q0 ──0──▶ q1
//!        ──1──▶ q0
//!     q1 ──0,1──▶ q1
//! ```

use crate::Graph;
use std::fmt::Write;

pub fn text(g: &Graph) -> String {
    let n = g.nodes.len();
    // `.any(…)`: true if some element matches, like `Array.prototype.some` in TypeScript.
    let rejects = g.nodes.iter().any(|x| x.reject);
    let mut s = format!(
        "{}: {n} state{} (→ start, * accept{})\n",
        if g.title.is_empty() {
            "machine"
        } else {
            &g.title
        },
        if n == 1 { "" } else { "s" },
        if rejects { ", ✗ reject" } else { "" }
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
            if node.accept {
                "*"
            } else if node.reject {
                "✗"
            } else {
                " "
            }
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
                    reject: false,
                },
                Node {
                    name: "q1".into(),
                    start: false,
                    accept: false,
                    reject: false,
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

    #[test]
    fn marks_a_reject_state() {
        let node = |name: &str, start, accept, reject| Node {
            name: name.into(),
            start,
            accept,
            reject,
        };
        let g = Graph {
            title: "tm".into(),
            nodes: vec![
                node("q0", true, false, false),
                node("qa", false, true, false),
                node("qr", false, false, true),
            ],
            edges: vec![],
        };
        assert_eq!(
            text(&g),
            "tm: 3 states (→ start, * accept, ✗ reject)\n→   q0\n  * qa\n  ✗ qr\n"
        );
    }
}
