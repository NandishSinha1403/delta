//! The "Delta by Nandish" strip under a drawing in the files users get (SVG, PNG, PDF; id
//! `render/watermark`). Terminal previews, social cards and TikZ don't use it.

use crate::svg::FONT;

/// The line every SVG, PNG and PDF file carries under its drawing.
pub const WATERMARK: &str = "Delta by Nandish";

/// Height of the strip that holds the watermark, and its font size (SVG px).
const MARK_STRIP: f64 = 18.0;
const MARK_SIZE: f64 = 10.0;

/// A numeric attribute of the SVG's root element (`width="189.8"` → 189.8).
fn root_attr(svg: &str, name: &str) -> Option<f64> {
    let root = &svg[..svg.find('>')?];
    let start = root.find(&format!(" {name}=\""))? + name.len() + 3;
    let len = root[start..].find('"')?;
    root[start..start + len].parse().ok()
}

/// The drawing with a strip added below it holding [`WATERMARK`], right-aligned in a muted
/// colour on the drawing's own background (its first full-size `rect`, else white). The
/// original SVG is nested unchanged, so the drawing itself is not moved or scaled.
pub fn watermark(svg: &str) -> String {
    let (Some(w), Some(h)) = (root_attr(svg, "width"), root_attr(svg, "height")) else {
        return svg.to_string();
    };
    const BG: &str = r#"<rect width="100%" height="100%" fill=""#;
    let bg = svg
        .find(BG)
        .and_then(|i| {
            let rest = &svg[i + BG.len()..];
            rest.find('"').map(|n| rest[..n].to_string())
        })
        .unwrap_or_else(|| "#ffffff".to_string());
    // A dark background gets a light mark; anything else a mid grey.
    let dark = bg.len() == 7
        && u8::from_str_radix(&bg[1..3], 16).is_ok_and(|r| r < 0x60)
        && u8::from_str_radix(&bg[3..5], 16).is_ok_and(|g| g < 0x60);
    let ink = if dark { "#8c9199" } else { "#6b7884" };
    // The mark must fit: Plex Mono glyphs are 0.6 em wide, plus 8 px on each side.
    let mark_w = WATERMARK.chars().count() as f64 * MARK_SIZE * 0.6 + 16.0;
    let total_w = w.max(mark_w);
    let x = (total_w - w) / 2.0;
    let total_h = h + MARK_STRIP;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}" font-family="{FONT}"><rect width="100%" height="100%" fill="{bg}"/><svg x="{x}" y="0" width="{w}" height="{h}">{inner}</svg><text x="{tx}" y="{ty}" font-size="{MARK_SIZE}" text-anchor="end" fill="{ink}">{WATERMARK}</text></svg>"##,
        inner = svg,
        tx = total_w - 8.0,
        ty = h + MARK_STRIP - 6.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="30"><rect width="100%" height="100%" fill="#fff"/><text x="4" y="20">q0</text></svg>"##;

    #[test]
    fn the_mark_sits_in_a_strip_below_the_unchanged_drawing() {
        let marked = watermark(SVG);
        assert!(marked.contains(">Delta by Nandish</text>"));
        // The drawing is nested unchanged; a narrow one is widened to fit the mark.
        assert!(marked.contains(SVG));
        assert_eq!(root_attr(&marked, "height"), Some(30.0 + MARK_STRIP));
        assert!(root_attr(&marked, "width").is_some_and(|w| w > 60.0));
        // The strip takes the drawing's background.
        assert!(marked.contains(r##"fill="#fff"/><svg x="##));
        // A dark drawing gets a light mark; something that isn't an SVG root is left alone.
        let dark = watermark(&SVG.replace("#fff", "#0d0e10"));
        assert!(dark.contains("fill=\"#8c9199\">Delta by Nandish"));
        assert_eq!(watermark("<g/>"), "<g/>");
    }
}
