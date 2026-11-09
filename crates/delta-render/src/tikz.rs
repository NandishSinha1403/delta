//! The same drawing as a TikZ picture: the layout's coordinates, so the paper matches the Pad.
//! Needs only core TikZ (`\usepackage{tikz}`); compiles with pdfLaTeX, XeLaTeX and LuaLaTeX.

use crate::{Curve, Graph, Layout, num, split_subscript};
use std::fmt::Write;

/// Text in math mode, with the symbols Delta labels use spelled as TeX commands.
fn tex(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let t = match c {
            'ε' => r"\varepsilon{}",
            'λ' => r"\lambda{}",
            'δ' => r"\delta{}",
            'Σ' => r"\Sigma{}",
            'Γ' => r"\Gamma{}",
            '→' => r"\to{}",
            '←' => r"\leftarrow{}",
            '⊔' => r"\sqcup{}",
            '□' => r"\square{}",
            '∅' => r"\emptyset{}",
            '·' => r"\cdot{}",
            '$' | '&' | '#' | '%' | '_' | '{' | '}' => {
                out.push('\\');
                out.push(c);
                continue;
            }
            '\\' => r"\backslash{}",
            '^' => r"\hat{}",
            '~' => r"\sim{}",
            ' ' => r"\ ",
            _ => {
                out.push(c);
                continue;
            }
        };
        out.push_str(t);
    }
    out
}

fn pt(x: f64, y: f64) -> String {
    format!("({},{})", num(x), num(y))
}

fn curve(c: &Curve) -> String {
    let mut s = pt(c.start.x, c.start.y);
    for (c1, c2, end) in &c.segments {
        let _ = write!(
            s,
            " .. controls {} and {} .. {}",
            pt(c1.x, c1.y),
            pt(c2.x, c2.y),
            pt(end.x, end.y)
        );
    }
    s
}

/// A `tikzpicture` (1 px = 0.75 pt, y down as in the SVG).
pub fn tikz(g: &Graph, l: &Layout) -> String {
    let mut s = String::from(
        "% Drawn by Delta (render/layered). Needs \\usepackage{tikz}.\n\\begin{tikzpicture}[x=0.75pt, y=-0.75pt, >=stealth, every node/.style={inner sep=0pt}]\n",
    );
    if let Some(start) = &l.start {
        let _ = writeln!(s, "  \\draw[->, semithick] {};", curve(start));
    }
    for (e, pe) in g.edges.iter().zip(&l.edges) {
        let dash = if e.label == "ε" { ", dashed" } else { "" };
        let _ = writeln!(s, "  \\draw[->, semithick{dash}] {};", curve(&pe.curve));
    }
    for (e, pe) in g.edges.iter().zip(&l.edges) {
        let lines: Vec<String> = e
            .label
            .split('\n')
            .map(|x| format!("${}$", tex(x)))
            .collect();
        let _ = writeln!(
            s,
            "  \\node[fill=white, rounded corners=2pt, inner sep=2pt, align=center, font=\\small] at {} {{{}}};",
            pt(pe.label_at.x, pe.label_at.y),
            lines.join(r"\\")
        );
    }
    for (i, (node, pn)) in g.nodes.iter().zip(&l.nodes).enumerate() {
        let name = match split_subscript(&node.name) {
            Some((base, sub)) => format!("${}_{{{}}}$", tex(base), sub),
            None => format!("${}$", tex(&node.name)),
        };
        let (x, y) = (pn.at.x, pn.at.y);
        let _ = writeln!(
            s,
            "  \\draw[fill=white, semithick] {} ellipse [x radius={}, y radius={}];",
            pt(x, y),
            num(pn.rx),
            num(pn.ry)
        );
        if node.accept {
            let _ = writeln!(
                s,
                "  \\draw[semithick] {} ellipse [x radius={}, y radius={}];",
                pt(x, y),
                num(pn.rx - 4.5),
                num(pn.ry - 4.5)
            );
        }
        let _ = writeln!(s, "  \\node (s{i}) at {} {{{name}}};", pt(x, y));
    }
    s.push_str("\\end{tikzpicture}\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node, layout};

    #[test]
    fn writes_nodes_curves_and_escaped_labels() {
        let g = Graph {
            title: "tm".into(),
            nodes: vec![
                Node {
                    name: "q1".into(),
                    start: true,
                    accept: false,
                },
                Node {
                    name: "acc_1".into(),
                    start: false,
                    accept: true,
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                label: "⊔ → R".into(),
            }],
        };
        let out = tikz(&g, &layout(&g));
        assert!(out.contains("$q_{1}$"));
        assert!(out.contains(r"$acc\_1$"));
        assert!(out.contains(r"\sqcup{}\ \to{}\ R"));
        assert_eq!(
            out.matches("ellipse").count(),
            3,
            "two states, one inner ring"
        );
        assert!(out.contains(".. controls"));
        assert!(out.trim_end().ends_with(r"\end{tikzpicture}"));
    }
}
