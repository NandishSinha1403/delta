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
