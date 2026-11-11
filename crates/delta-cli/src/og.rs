//! Social cards (1200×630 PNG) for docs pages, drawn by `delta-render` + `delta-export`: the
//! page title and the machine its first example shows. Written by `deltac gen-docs` to
//! `site/public/og/<page id>.png`; pages without a shown machine keep the site's default card.

use std::path::{Path, PathBuf};

const W: f64 = 1200.0;
const H: f64 = 630.0;
const PAD: f64 = 64.0;
const TITLE_SIZE: f64 = 50.0;

/// Every Markdown file under `dir`, sorted (so the output order is stable).
fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            markdown_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "md" || x == "mdx") {
            out.push(p);
        }
    }
}

/// The page's `title:` from its front matter.
fn title(md: &str) -> Option<String> {
    let front = md.strip_prefix("---\n")?.split("\n---").next()?;
    let line = front.lines().find(|l| l.starts_with("title:"))?;
    Some(
        line.trim_start_matches("title:")
            .trim()
            .trim_matches('"')
            .to_string(),
    )
}

/// The first ```delta block (not an `error` block).
fn first_example(md: &str) -> Option<String> {
    let mut lines = md.lines();
    while let Some(l) = lines.next() {
        let fence = l.trim_start();
        if fence.starts_with("```delta") && !fence.contains("error") && !fence.contains("outcome") {
            let body: Vec<&str> = lines
                .by_ref()
                .take_while(|l| !l.trim_start().starts_with("```"))
                .collect();
            return Some(body.join("\n"));
        }
    }
    None
}

/// Splits a title into lines of at most `width` characters, at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= width => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The card's SVG: title top left, the diagram scaled into the space below, the site's name.
pub fn card_svg(page_title: &str, value: &serde_json::Value) -> Option<String> {
    let g = delta_lang::render::graph_of(value)?;
    let layout = delta_render::layout(&g);
    let theme = delta_render::Theme {
        background: None,
        ..delta_render::Theme::light()
    };
    let inner = delta_render::svg(&g, &layout, &theme);
    let lines = wrap(page_title, ((W - 2.0 * PAD) / (TITLE_SIZE * 0.6)) as usize);
    let lines: Vec<String> = lines.into_iter().take(2).collect();
    let top = PAD + 20.0 + lines.len() as f64 * TITLE_SIZE * 1.15;
    let (bw, bh) = (W - 2.0 * PAD, H - top - PAD - 40.0);
    let k = (bw / layout.width).min(bh / layout.height).min(2.2);
    let (dw, dh) = (layout.width * k, layout.height * k);
    let (dx, dy) = (PAD + (bw - dw) / 2.0, top + (bh - dh) / 2.0);
    // Turn the diagram's own <svg …> into a nested, positioned and scaled one.
    let body_start = inner.find('>')? + 1;
    let nested = format!(
        r#"<svg x="{dx:.1}" y="{dy:.1}" width="{dw:.1}" height="{dh:.1}" viewBox="0 0 {:.2} {:.2}" font-family="{}">{}"#,
        layout.width,
        layout.height,
        delta_render::FONT,
        inner.get(body_start..)?.trim_end()
    );
    let mut title_svg = String::new();
    for (i, l) in lines.iter().enumerate() {
        title_svg.push_str(&format!(
            r##"<text x="{PAD}" y="{:.1}" font-size="{TITLE_SIZE}" fill="#1d3557">{}</text>"##,
            PAD + TITLE_SIZE * 0.9 + i as f64 * TITLE_SIZE * 1.15,
            esc(l)
        ));
    }
    Some(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="{font}"><rect width="100%" height="100%" fill="#f8fafa"/><rect x="0" y="0" width="12" height="{H}" fill="#2f6f8f"/>{title_svg}{nested}<text x="{PAD}" y="{:.1}" font-size="26" fill="#2f6f8f">δ delta.nandish.dev</text></svg>"##,
        H - PAD + 16.0,
        font = delta_render::FONT
    ))
}

/// Writes a card for every page whose first example shows a machine. Returns the files.
pub fn write_cards(site: &Path) -> std::io::Result<Vec<PathBuf>> {
    let content = site.join("src/content/docs");
    let mut pages = Vec::new();
    markdown_files(&content, &mut pages);
    let mut written = Vec::new();
    for page in pages {
        let md = std::fs::read_to_string(&page)?;
        let (Some(t), Some(src)) = (title(&md), first_example(&md)) else {
            continue;
        };
        let Some(out) = crate::run::evaluate(&src) else {
            continue;
        };
        let shown = out["outputs"]
            .as_array()
            .into_iter()
            .flatten()
            .rev()
            .find(|o| o["kind"] == "show" && o["value"]["type"] == "machine");
        let Some(svg) = shown.and_then(|o| card_svg(&t, &o["value"])) else {
            continue;
        };
        let Ok(png) = delta_export::png(&svg, 1.0) else {
            continue;
        };
        // The page id as route-data.ts computes it: the path without extension or `/index`.
        let rel = page
            .strip_prefix(&content)
            .unwrap_or(&page)
            .with_extension("");
        let id = rel.to_string_lossy().trim_end_matches("/index").to_string();
        let file = site.join("public/og").join(format!("{id}.png"));
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&file, png)?;
        written.push(file);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_title_and_first_runnable_example() {
        let md = "---\ntitle: \"NFA to DFA\"\n---\n\n```delta error\nbad\n```\n\n```delta embed\nshow 1\n```\n";
        assert_eq!(title(md).as_deref(), Some("NFA to DFA"));
        assert_eq!(first_example(md).as_deref(), Some("show 1"));
    }

    #[test]
    fn titles_wrap_at_spaces() {
        assert_eq!(
            wrap("NFA to DFA converter (subset construction)", 20),
            ["NFA to DFA converter", "(subset", "construction)"]
        );
    }

    #[test]
    fn a_card_is_1200_by_630() {
        let out = delta_lang::run("dfa M {\n start q0; accept q0\n q0 -a-> q0\n}\nshow M");
        let svg = card_svg("A title", &out["outputs"][0]["value"]).unwrap_or_default();
        let png = delta_export::png(&svg, 1.0).unwrap_or_default();
        let be = |i: usize| {
            png.get(i..i + 4)
                .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        };
        assert_eq!((be(16), be(20)), (1200, 630));
    }
}
