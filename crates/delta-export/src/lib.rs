//! SVG → PNG and PDF, with the code font built in: IBM Plex Mono, plus Noto Sans Math (Greek,
//! ∪ ∩ ∈ ⊔ ⊢ ▸ …) and Noto Sans Mono (⁺ ⁿ ᵏ …) subsets for exactly the characters Plex lacks, and
//! a STIX Two Text italic subset for state names.
//!
//! `deltac` and the Pad's export module (`delta-export-wasm`) both call these two functions on
//! the SVG from `delta-render`, so a file saved in the terminal and one downloaded from the Pad
//! are the same bytes. No system fonts are read.

// `Arc` is a shared, reference-counted pointer that can cross threads; usvg keeps fonts in one.
use std::sync::Arc;

/// The embedded fonts (OFL; licences in `fonts/`), cut by `scripts/subset-fonts.py`.
const PLEX_MONO: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.subset.ttf");
const NOTO_MATH: &[u8] = include_bytes!("../fonts/NotoSansMath-Regular.subset.ttf");
const NOTO_MONO: &[u8] = include_bytes!("../fonts/NotoSansMono-Regular.subset.ttf");
const STIX_ITALIC: &[u8] = include_bytes!("../fonts/STIXTwoText-Italic.subset.ttf");

/// The font stack for SVG text: the code font first, then its two fallbacks.
pub const FONT_FAMILY: &str = "IBM Plex Mono, Noto Sans Math, Noto Sans Mono, monospace";

/// The largest PNG side we draw, in pixels (keeps a huge machine from using gigabytes).
pub const MAX_SIDE: u32 = 8192;

fn options() -> resvg::usvg::Options<'static> {
    let mut db = resvg::usvg::fontdb::Database::new();
    db.load_font_data(PLEX_MONO.to_vec());
    db.load_font_data(NOTO_MATH.to_vec());
    db.load_font_data(NOTO_MONO.to_vec());
    db.load_font_data(STIX_ITALIC.to_vec());
    db.set_monospace_family("IBM Plex Mono");
    db.set_sans_serif_family("IBM Plex Mono");
    resvg::usvg::Options {
        font_family: "IBM Plex Mono".to_string(),
        fontdb: Arc::new(db),
        ..resvg::usvg::Options::default()
    }
}

fn tree(svg: &str) -> Result<resvg::usvg::Tree, String> {
    resvg::usvg::Tree::from_str(svg, &options()).map_err(|e| format!("bad SVG: {e}"))
}

fn raster(svg: &str, scale: f32) -> Result<resvg::tiny_skia::Pixmap, String> {
    let tree = tree(svg)?;
    let size = tree.size();
    let scale = scale.clamp(0.1, 8.0);
    let side = |v: f32| (v * scale).ceil().clamp(1.0, MAX_SIDE as f32) as u32;
    let (w, h) = (side(size.width()), side(size.height()));
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("image too large")?;
    let sx = w as f32 / size.width().max(1.0);
    let sy = h as f32 / size.height().max(1.0);
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Draws the SVG as a PNG at `scale` (2.0 = sharp on high-density screens).
pub fn png(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    raster(svg, scale)?
        .encode_png()
        .map_err(|e| format!("PNG encoding failed: {e}"))
}

/// The line every PNG file carries under its drawing.
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
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}" font-family="{FONT_FAMILY}"><rect width="100%" height="100%" fill="{bg}"/><svg x="{x}" y="0" width="{w}" height="{h}">{inner}</svg><text x="{tx}" y="{ty}" font-size="{MARK_SIZE}" text-anchor="end" fill="{ink}">{WATERMARK}</text></svg>"##,
        inner = svg,
        tx = total_w - 8.0,
        ty = h + MARK_STRIP - 6.0,
    )
}

/// A PNG file for the user: the drawing at `scale` with the [`WATERMARK`] strip under it.
/// (`png` alone, without the mark, is for terminal previews and the site's social cards.)
pub fn png_file(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    png(&watermark(svg), scale)
}

/// Draws the SVG as raw pixels: (width, height, RGBA bytes). Colours are premultiplied by
/// alpha, which is exact for Delta's drawings because their background is opaque.
pub fn rgba(svg: &str, scale: f32) -> Result<(u32, u32, Vec<u8>), String> {
    let p = raster(svg, scale)?;
    Ok((p.width(), p.height(), p.take()))
}

/// Converts the SVG to a one-page PDF with the fonts embedded as vector text.
pub fn pdf(svg: &str) -> Result<Vec<u8>, String> {
    let tree = tree(svg)?;
    svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions::default(),
    )
    .map_err(|e| format!("PDF conversion failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="30" font-family="IBM Plex Mono, Noto Sans Math, Noto Sans Mono, monospace"><rect width="100%" height="100%" fill="#fff"/><text x="4" y="20" font-size="14">q0 ⊔</text></svg>"##;

    #[test]
    fn png_has_the_right_size_and_ink() {
        let bytes = png(SVG, 2.0).unwrap_or_default();
        assert_eq!(bytes.get(1..4), Some(&b"PNG"[..]));
        // Width and height are big-endian at bytes 16..24 of the IHDR chunk.
        let be = |i: usize| {
            bytes
                .get(i..i + 4)
                .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        };
        assert_eq!((be(16), be(20)), (120, 60));
    }

    #[test]
    fn png_files_carry_the_watermark_below_the_drawing() {
        let marked = watermark(SVG);
        assert!(marked.contains(">Delta by Nandish</text>"));
        // The drawing is nested unchanged; a narrow one is widened to fit the mark.
        assert!(marked.contains(SVG));
        assert_eq!(root_attr(&marked, "height"), Some(30.0 + MARK_STRIP));
        assert!(root_attr(&marked, "width").is_some_and(|w| w > 60.0));
        // The strip takes the drawing's background.
        assert!(marked.contains(r##"fill="#fff"/><svg x="##));
        // The PNG file is taller than the bare drawing by the strip, at the same scale.
        let bytes = png_file(SVG, 2.0).unwrap_or_default();
        let h = bytes
            .get(20..24)
            .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
        assert_eq!(h, ((30.0 + MARK_STRIP) * 2.0) as u32);
        // A dark drawing gets a light mark; something that isn't an SVG root is left alone.
        let dark = watermark(&SVG.replace("#fff", "#0d0e10"));
        assert!(dark.contains("fill=\"#8c9199\">Delta by Nandish"));
        assert_eq!(watermark("<g/>"), "<g/>");
    }

    #[test]
    fn text_is_drawn_with_the_embedded_fonts() {
        let tree = tree(SVG);
        assert!(tree.is_ok());
        // A text-free picture and one with text differ: glyphs were found (no system fonts).
        let blank = png(&SVG.replace("q0 ⊔", ""), 1.0).unwrap_or_default();
        let inked = png(SVG, 1.0).unwrap_or_default();
        assert_ne!(blank, inked);
        // ⊔ comes from the Noto subset: drawing it alone still leaves ink.
        let blank_cup = png(&SVG.replace("q0 ⊔", " "), 1.0).unwrap_or_default();
        let cup = png(&SVG.replace("q0 ⊔", "⊔"), 1.0).unwrap_or_default();
        assert_ne!(blank_cup, cup);
    }

    /// The glyph ids each character of `line` is drawn with, with the font's family name;
    /// glyph 0 is the "missing glyph" box.
    fn glyphs(line: &str) -> Vec<(String, u16, String)> {
        glyphs_in(line, FONT_FAMILY, "normal")
    }

    fn glyphs_in(line: &str, family: &str, style: &str) -> Vec<(String, u16, String)> {
        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="2000" height="30" font-family="{family}" font-style="{style}"><text x="4" y="20" font-size="14">{line}</text></svg>"##
        );
        let Ok(tree) = tree(&svg) else {
            return Vec::new();
        };
        let db = options().fontdb;
        let mut out = Vec::new();
        for node in tree.root().children() {
            // `if let`: runs the block only when the pattern matches (here: a text node).
            if let resvg::usvg::Node::Text(text) = node {
                for span in text.layouted() {
                    for g in &span.positioned_glyphs {
                        let family = db
                            .face(g.font)
                            .and_then(|f| f.families.first().map(|(name, _)| name.clone()))
                            .unwrap_or_default();
                        out.push((g.text.clone(), g.id.0, family));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn every_delta_symbol_has_a_glyph_in_the_embedded_fonts() {
        let symbols = include_str!("../fonts/symbols.txt");
        let line: String = symbols
            .lines()
            .filter(|l| !l.starts_with('#'))
            .collect::<Vec<_>>()
            .join(" ");
        let drawn = glyphs(&line);
        let expected = line.chars().filter(|c| !c.is_whitespace()).count();
        let shown: Vec<_> = drawn.iter().filter(|(t, _, _)| t.trim() != "").collect();
        assert_eq!(shown.len(), expected, "every character was laid out");
        let fonts = ["IBM Plex Mono", "Noto Sans Math", "Noto Sans Mono"];
        let missing: Vec<_> = shown
            .iter()
            .filter(|(_, id, family)| *id == 0 || !fonts.contains(&family.as_str()))
            .map(|(t, _, f)| format!("{t} ({f})"))
            .collect();
        assert!(missing.is_empty(), "no glyph for: {}", missing.join(", "));
        // Plain code is set in Plex itself, not a fallback.
        assert!(glyphs("q0").iter().all(|(_, _, f)| f == "IBM Plex Mono"));
    }

    #[test]
    fn state_names_are_drawn_in_stix_italic() {
        let drawn = glyphs_in("q0 qa {:q1}", delta_names(), "italic");
        assert!(!drawn.is_empty());
        for (t, id, family) in &drawn {
            if t.trim().is_empty() {
                continue;
            }
            assert!(*id != 0, "no glyph for {t}");
            assert_eq!(family, "STIX Two Text", "{t}");
        }
    }

    /// The name font stack `delta-render` writes (kept in step with `NAME_FONT` there).
    fn delta_names() -> &'static str {
        "STIX Two Text, IBM Plex Mono, Noto Sans Math, Noto Sans Mono, serif"
    }

    #[test]
    fn a_character_no_font_has_is_reported_missing() {
        // ✨ is in none of the three fonts: the check above would catch it.
        assert!(glyphs("✨").iter().any(|(_, id, _)| *id == 0));
    }

    #[test]
    fn pdf_is_a_pdf_with_an_embedded_font() {
        let bytes = pdf(SVG).unwrap_or_default();
        assert!(bytes.starts_with(b"%PDF-"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("FontFile"), "font embedded");
    }

    #[test]
    fn bad_svg_is_an_error_not_a_panic() {
        assert!(png("<svg", 1.0).is_err());
        assert!(pdf("not svg").is_err());
    }
}
