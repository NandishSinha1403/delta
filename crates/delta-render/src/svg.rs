//! The picture as SVG, in the Pad's look: circles, double circles for accept states, a start
//! arrow, labels on rounded pills, dashed ε-arrows.

use crate::{
    Curve, Graph, LABEL_LINE, LABEL_SIZE, Layout, NAME_SIZE, Point, SUB_SCALE, dist, num,
    split_subscript,
};
// `std::fmt::Write` lets `write!` append formatted text to a String.
use std::fmt::Write;

/// Colours of a drawing. The light and dark themes match the Pad's tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// `None` = transparent background.
    pub background: Option<&'static str>,
    pub node: &'static str,
    pub node_fill: &'static str,
    pub accept: &'static str,
    pub edge: &'static str,
    pub pill: &'static str,
    pub label: &'static str,
    pub text: &'static str,
}

// `impl Theme { … }` adds functions that belong to the type.
impl Theme {
    pub fn light() -> Theme {
        Theme {
            background: Some("#ffffff"),
            node: "#1d3557",
            node_fill: "#ffffff",
            accept: "#1d3557",
            edge: "#6b7884",
            pill: "#e5ecee",
            label: "#2f4058",
            text: "#1d3557",
        }
    }

    pub fn dark() -> Theme {
        Theme {
            background: Some("#0d0e10"),
            node: "#c6c5c0",
            node_fill: "#111215",
            accept: "#f1faee",
            edge: "#7a7f87",
            pill: "#1b1d21",
            label: "#c6c5c0",
            text: "#f1faee",
        }
    }
}

/// The font stack: IBM Plex Mono, then the fallbacks for what it lacks. `delta-export` embeds all
/// three; elsewhere `monospace` is the last resort.
pub const FONT: &str = "IBM Plex Mono, Noto Sans Math, Noto Sans Mono, monospace";

/// State names: STIX Two Text italic, as maths is set; the code fonts cover anything it lacks.
pub const NAME_FONT: &str = "STIX Two Text, IBM Plex Mono, Noto Sans Math, Noto Sans Mono, serif";

const ARROW: f64 = 9.0;
const ARROW_HALF: f64 = 4.2;

/// Escapes text for XML.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// SVG path data for a curve, ending `trim` px early (where the arrowhead starts).
fn path_data(c: &Curve, trim: f64) -> String {
    let mut d = format!("M{},{}", num(c.start.x), num(c.start.y));
    let last = c.segments.len().saturating_sub(1);
    for (i, &(c1, c2, end)) in c.segments.iter().enumerate() {
        let (c2, end) = if i == last && trim > 0.0 {
            let back = c.before_end();
            let len = dist(back, end);
            if len > 0.0 {
                let k = (trim / len).min(1.0);
                let moved = Point {
                    x: end.x + (back.x - end.x) * k,
                    y: end.y + (back.y - end.y) * k,
                };
                (
                    Point {
                        x: c2.x + moved.x - end.x,
                        y: c2.y + moved.y - end.y,
                    },
                    moved,
                )
            } else {
                (c2, end)
            }
        } else {
            (c2, end)
        };
        let _ = write!(
            d,
            " C{},{} {},{} {},{}",
            num(c1.x),
            num(c1.y),
            num(c2.x),
            num(c2.y),
            num(end.x),
            num(end.y)
        );
    }
    d
}

/// SVG path data for an arrow's curve, stopping where its arrowhead begins (the Pad draws the
/// head itself from [`Curve::end`] and [`Curve::before_end`]).
pub fn arrow_path(c: &Curve) -> String {
    path_data(c, ARROW * 0.8)
}

fn arrowhead(c: &Curve) -> String {
    let tip = c.end();
    let base = c.before_end();
    let len = dist(tip, base).max(1e-9);
    let (ux, uy) = ((tip.x - base.x) / len, (tip.y - base.y) / len);
    let (bx, by) = (tip.x - ux * ARROW, tip.y - uy * ARROW);
    format!(
        "{},{} {},{} {},{}",
        num(tip.x),
        num(tip.y),
        num(bx - uy * ARROW_HALF),
        num(by + ux * ARROW_HALF),
        num(bx + uy * ARROW_HALF),
        num(by - ux * ARROW_HALF)
    )
}

/// A circle, or an ellipse if the radii differ.
fn shape(at: Point, rx: f64, ry: f64, attrs: &str) -> String {
    if (rx - ry).abs() < 0.01 {
        format!(
            r#"<circle cx="{}" cy="{}" r="{}" {attrs}/>"#,
            num(at.x),
            num(at.y),
            num(rx)
        )
    } else {
        format!(
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" {attrs}/>"#,
            num(at.x),
            num(at.y),
            num(rx),
            num(ry)
        )
    }
}

/// The diagram as a standalone SVG document.
pub fn svg(g: &Graph, l: &Layout, t: &Theme) -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{FONT}">"#,
        w = num(l.width),
        h = num(l.height)
    );
    let _ = write!(s, "<title>{}</title>", esc(&describe(g)));
    if let Some(bg) = t.background {
        let _ = write!(s, r#"<rect width="100%" height="100%" fill="{bg}"/>"#);
    }
    if let Some(start) = &l.start {
        let _ = write!(
            s,
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="1.4"/><polygon points="{}" fill="{}"/>"#,
            path_data(start, ARROW * 0.8),
            t.edge,
            arrowhead(start),
            t.edge
        );
    }
    for (e, pe) in g.edges.iter().zip(&l.edges) {
        let dash = if e.label == "ε" {
            r#" stroke-dasharray="5 4""#
        } else {
            ""
        };
        let _ = write!(
            s,
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="1.4"{dash}/><polygon points="{}" fill="{}"/>"#,
            path_data(&pe.curve, ARROW * 0.8),
            t.edge,
            arrowhead(&pe.curve),
            t.edge
        );
    }
    // Labels after all arrows, so no arrow crosses a label.
    for (e, pe) in g.edges.iter().zip(&l.edges) {
        let _ = write!(
            s,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="5" fill="{}"/>"#,
            num(pe.label_at.x - pe.label_w / 2.0),
            num(pe.label_at.y - pe.label_h / 2.0),
            num(pe.label_w),
            num(pe.label_h),
            t.pill
        );
        let lines: Vec<&str> = e.label.split('\n').collect();
        let first = pe.label_at.y - (lines.len() as f64 - 1.0) * LABEL_LINE / 2.0;
        for (i, line) in lines.iter().enumerate() {
            let _ = write!(
                s,
                r#"<text x="{}" y="{}" font-size="{}" text-anchor="middle" fill="{}">{}</text>"#,
                num(pe.label_at.x),
                num(first + i as f64 * LABEL_LINE + LABEL_SIZE * 0.35),
                num(LABEL_SIZE),
                t.label,
                esc(line)
            );
        }
    }
    for (node, pn) in g.nodes.iter().zip(&l.nodes) {
        let stroke = if node.accept { t.accept } else { t.node };
        let _ = write!(
            s,
            "{}",
            shape(
                pn.at,
                pn.rx,
                pn.ry,
                &format!(
                    r#"fill="{}" stroke="{stroke}" stroke-width="1.6""#,
                    t.node_fill
                )
            )
        );
        if node.accept {
            let _ = write!(
                s,
                "{}",
                shape(
                    pn.at,
                    pn.rx - 4.5,
                    pn.ry - 4.5,
                    &format!(r#"fill="none" stroke="{stroke}" stroke-width="1.4""#)
                )
            );
        }
        let base_y = pn.at.y + NAME_SIZE * 0.3;
        // Centred as one chunk (name and subscript together) by `text-anchor`.
        let _ = write!(
            s,
            r#"<text x="{}" y="{}" font-family="{NAME_FONT}" font-style="italic" font-size="{}" text-anchor="middle" fill="{}">"#,
            num(pn.at.x),
            num(base_y),
            num(NAME_SIZE),
            t.text
        );
        match split_subscript(&node.name) {
            Some((base, sub)) => {
                let _ = write!(
                    s,
                    r#"{}<tspan font-size="{}" dy="4">{}</tspan>"#,
                    esc(base),
                    num(NAME_SIZE * SUB_SCALE),
                    esc(sub)
                );
            }
            None => s.push_str(&esc(&node.name)),
        }
        s.push_str("</text>");
    }
    s.push_str("</svg>\n");
    s
}

/// A sentence that says what the picture shows (the SVG title, for screen readers).
pub fn describe(g: &Graph) -> String {
    let start = g
        .nodes
        .iter()
        .find(|n| n.start)
        .map_or("none", |n| n.name.as_str());
    let accept: Vec<&str> = g
        .nodes
        .iter()
        .filter(|n| n.accept)
        .map(|n| n.name.as_str())
        .collect();
    let n = g.nodes.len();
    format!(
        "{} with {n} state{}, start {start}, accept {}",
        if g.title.is_empty() {
            "machine"
        } else {
            &g.title
        },
        if n == 1 { "" } else { "s" },
        if accept.is_empty() {
            "none".to_string()
        } else {
            accept.join(", ")
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node, layout};

    #[test]
    fn draws_states_arrows_and_escapes_text() {
        let g = Graph {
            title: "pda".into(),
            nodes: vec![
                Node {
                    name: "q0".into(),
                    start: true,
                    accept: false,
                    reject: false,
                },
                Node {
                    name: "<a&b>".into(),
                    start: false,
                    accept: true,
                    reject: false,
                },
            ],
            edges: vec![
                Edge {
                    from: 0,
                    to: 1,
                    label: "a, X → ε".into(),
                },
                Edge {
                    from: 1,
                    to: 1,
                    label: "ε".into(),
                },
            ],
        };
        let out = svg(&g, &layout(&g), &Theme::light());
        assert!(out.starts_with("<svg") && out.ends_with("</svg>\n"));
        assert_eq!(
            out.matches("<circle").count() + out.matches("<ellipse").count(),
            3,
            "two states, one double ring"
        );
        assert!(out.contains("&lt;a&amp;b&gt;"));
        assert!(out.contains("stroke-dasharray"));
        assert!(out.contains(r#"<tspan font-size="11.7" dy="4">0</tspan>"#));
        assert!(out.contains(r#"font-style="italic""#));
        assert!(!out.contains("NaN"));
    }
}
