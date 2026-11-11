//! Draws a state diagram the same way everywhere: `deltac`, the Pad's image export and the
//! site's social cards all go through this crate, so they agree to the pixel.
//!
//! - [`layout`]: a layered (left-to-right) layout, algorithm id `render/layered`.
//! - [`svg`]: the picture as SVG text, in a [`Theme`].
//! - [`tikz`]: the same geometry as a TikZ picture.
//! - [`text`]: a plain-text listing for terminals that can't show images.
//!
//! Labels are measured as IBM Plex Mono (every character 0.6 em wide) and state names as STIX Two
//! Text italic (from its width table), the fonts `delta-export` embeds, so shapes fit their text
//! exactly when the SVG is rasterized.

mod layered;
mod stix_widths;
mod svg;
mod text;
mod tikz;
mod watermark;

pub use layered::layout;
pub use svg::{FONT, NAME_FONT, Theme, arrow_path, describe, svg};
pub use text::text;
pub use tikz::tikz;
pub use watermark::{WATERMARK, watermark};

/// A diagram to draw: states, and arrows between them by index.
// `#[derive(...)]` asks the compiler to write common trait impls (copying, printing, comparing).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    /// What the drawing is of, e.g. `dfa`; used in the text form and as the SVG title.
    pub title: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// One state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub name: String,
    pub start: bool,
    pub accept: bool,
}

/// One arrow. Parallel arrows should already be merged into one label; a label may have
/// several lines (`\n`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: String,
}

/// A point in pixels; y grows downwards, as in SVG.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A curve: `start`, then cubic segments `(control 1, control 2, end)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    pub start: Point,
    pub segments: Vec<(Point, Point, Point)>,
}

impl Curve {
    /// The last point: where the arrowhead's tip goes.
    pub fn end(&self) -> Point {
        self.segments.last().map_or(self.start, |s| s.2)
    }

    /// The point the arrowhead points away from (the curve's direction at its end).
    pub fn before_end(&self) -> Point {
        match self.segments.last() {
            Some((_, c2, end)) if dist(*c2, *end) > 0.01 => *c2,
            Some((c1, _, _)) => *c1,
            None => self.start,
        }
    }
}

/// A placed state: a circle, or an ellipse when its name is long.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlacedNode {
    pub at: Point,
    /// Outer radii in px (equal for a circle).
    pub rx: f64,
    pub ry: f64,
}

impl PlacedNode {
    /// Where the line from the centre towards `toward` meets the outline.
    pub fn boundary(&self, toward: Point) -> Point {
        let (dx, dy) = (toward.x - self.at.x, toward.y - self.at.y);
        let k = ((dx / self.rx).powi(2) + (dy / self.ry).powi(2)).sqrt();
        if k > 0.0 {
            Point {
                x: self.at.x + dx / k,
                y: self.at.y + dy / k,
            }
        } else {
            self.at
        }
    }
}

/// The radii of a state with this name: a circle up to `MAX_ROUND` px tall, then an ellipse.
pub fn node_radii(name: &str) -> (f64, f64) {
    let w = name_width(name);
    let r = (w / 2.0 + 8.0).max(MIN_ROUND);
    if r <= MAX_ROUND {
        (r, r)
    } else {
        // Wider than half the text so the ends of the name stay inside the curve.
        (w / 2.0 * 1.18 + 8.0, MAX_ROUND)
    }
}

/// The smallest circle radius, and the largest before a state becomes an ellipse.
pub const MIN_ROUND: f64 = 20.0;
pub const MAX_ROUND: f64 = 24.0;

/// How far a self-loop rises above its state's centre (its highest point).
pub fn loop_height(ry: f64) -> f64 {
    0.25 * ry + 0.75 * (ry + LOOP_RISE)
}

/// How far the loop's control points rise above the top of the state.
pub const LOOP_RISE: f64 = 30.0;

/// A placed arrow, in the same order as [`Graph::edges`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacedEdge {
    pub curve: Curve,
    /// The centre of the label's box.
    pub label_at: Point,
    pub label_w: f64,
    pub label_h: f64,
}

/// Where everything goes. Sizes in px.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub width: f64,
    pub height: f64,
    pub nodes: Vec<PlacedNode>,
    pub edges: Vec<PlacedEdge>,
    /// The start arrow, if the graph has a start state.
    pub start: Option<Curve>,
}

/// Font sizes, and the width of one label character (IBM Plex Mono advances are 600/1000 em).
/// State names are STIX Two Text italic at `NAME_SIZE`, subscripts at `SUB_SCALE` of it.
pub const NAME_SIZE: f64 = 18.0;
pub const SUB_SCALE: f64 = 0.65;
pub const LABEL_SIZE: f64 = 13.0;
pub const LABEL_LINE: f64 = 16.0;
pub const CHAR_EM: f64 = 0.6;

/// How a state name is set: `q12` becomes `q` with a subscript `12`.
pub fn split_subscript(name: &str) -> Option<(&str, &str)> {
    // `char_indices` yields each character with its byte offset.
    let mut chars = name.char_indices();
    let (_, first) = chars.next()?;
    let (i, _) = chars.next()?;
    let rest = &name[i..];
    if first.is_ascii_alphabetic() && rest.chars().all(|c| c.is_ascii_digit()) && rest.len() <= 4 {
        Some((&name[..i], rest))
    } else {
        None
    }
}

/// Width of text in STIX Two Text italic at `size` px (characters outside the subset count as
/// 0.55 em, about a fallback font's average).
fn stix_width(text: &str, size: f64) -> f64 {
    text.chars()
        .map(|c| {
            let em = stix_widths::STIX_ITALIC
                .binary_search_by_key(&(c as u32), |(cp, _)| *cp)
                .ok()
                .and_then(|i| stix_widths::STIX_ITALIC.get(i))
                .map_or(550.0, |(_, w)| f64::from(*w));
            em / 1000.0 * size
        })
        .sum()
}

/// Width of a state name as drawn.
pub fn name_width(name: &str) -> f64 {
    match split_subscript(name) {
        Some((base, sub)) => stix_width(base, NAME_SIZE) + stix_width(sub, NAME_SIZE * SUB_SCALE),
        None => stix_width(name, NAME_SIZE),
    }
}

/// Width and height of an arrow label's box.
pub fn label_size(label: &str) -> (f64, f64) {
    let lines: Vec<&str> = label.split('\n').collect();
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    (
        longest as f64 * LABEL_SIZE * CHAR_EM + 12.0,
        lines.len() as f64 * LABEL_LINE + 4.0,
    )
}

pub(crate) fn dist(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// Formats a coordinate with at most two decimals, without trailing zeros.
pub(crate) fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}
