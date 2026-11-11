//! SVG → PNG and PDF, with Commit Mono and a Noto Sans Math subset (for ⊔, ⊢, ▸ …) built in.
//!
//! `deltac` and the Pad's export module (`delta-export-wasm`) both call these two functions on
//! the SVG from `delta-render`, so a file saved in the terminal and one downloaded from the Pad
//! are the same bytes. No system fonts are read.

// `Arc` is a shared, reference-counted pointer that can cross threads; usvg keeps fonts in one.
use std::sync::Arc;

/// The embedded fonts (OFL; licences in `fonts/`). Subsets: Latin, Greek, arrows, maths.
const COMMIT_MONO: &[u8] = include_bytes!("../fonts/CommitMono-400-Regular.subset.otf");
const NOTO_MATH: &[u8] = include_bytes!("../fonts/NotoSansMath-Regular.subset.ttf");

/// The largest PNG side we draw, in pixels (keeps a huge machine from using gigabytes).
pub const MAX_SIDE: u32 = 8192;

fn options() -> resvg::usvg::Options<'static> {
    let mut db = resvg::usvg::fontdb::Database::new();
    db.load_font_data(COMMIT_MONO.to_vec());
    db.load_font_data(NOTO_MATH.to_vec());
    db.set_monospace_family("CommitMono");
    db.set_sans_serif_family("CommitMono");
    resvg::usvg::Options {
        font_family: "CommitMono".to_string(),
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

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="30" font-family="CommitMono, Noto Sans Math, monospace"><rect width="100%" height="100%" fill="#fff"/><text x="4" y="20" font-size="14">q0 ⊔</text></svg>"##;

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
