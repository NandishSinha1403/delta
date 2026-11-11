//! Pictures in the terminal: the kitty, iTerm2 and sixel image protocols, chosen from the
//! environment. `DELTAC_IMAGES=kitty|iterm|sixel|none` overrides the guess.

// `IsTerminal` tells whether a stream is an interactive terminal (not a pipe or file).
use std::io::{IsTerminal, Write};

/// How this terminal shows images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Kitty,
    Iterm,
    Sixel,
    /// No images: draw machines as text.
    None,
}

/// Guesses the protocol from environment variables. Only when stdout is a terminal.
pub fn detect() -> Protocol {
    if !std::io::stdout().is_terminal() {
        return Protocol::None;
    }
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    from_env(
        &var("DELTAC_IMAGES"),
        &var("TERM"),
        &var("TERM_PROGRAM"),
        &var("LC_TERMINAL"),
        !var("KITTY_WINDOW_ID").is_empty(),
    )
}

/// The guess itself, separate from the process environment so it can be tested.
pub fn from_env(
    force: &str,
    term: &str,
    program: &str,
    lc_terminal: &str,
    kitty_window: bool,
) -> Protocol {
    match force {
        "kitty" => return Protocol::Kitty,
        "iterm" => return Protocol::Iterm,
        "sixel" => return Protocol::Sixel,
        "none" => return Protocol::None,
        _ => {}
    }
    if kitty_window || term == "xterm-kitty" || term == "xterm-ghostty" || program == "ghostty" {
        Protocol::Kitty
    } else if program == "iTerm.app" || program == "WezTerm" || lc_terminal == "iTerm2" {
        Protocol::Iterm
    } else if term.starts_with("foot")
        || term.starts_with("mlterm")
        || term.contains("sixel")
        || program == "contour"
    {
        Protocol::Sixel
    } else {
        Protocol::None
    }
}

/// Standard base64 (the image protocols' encoding).
pub fn base64(data: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18u32, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(char::from(
                    ABC.get(((n >> shift) & 63) as usize)
                        .copied()
                        .unwrap_or(b'A'),
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The escape sequence that shows `svg` with the given protocol, or None if it can't.
pub fn image(svg: &str, protocol: Protocol) -> Option<String> {
    match protocol {
        Protocol::None => None,
        Protocol::Iterm => {
            let png = delta_export::png(svg, 2.0).ok()?;
            let (w, _) = svg_size(svg);
            Some(format!(
                "\x1b]1337;File=inline=1;size={};width={}px;preserveAspectRatio=1:{}\x07\n",
                png.len(),
                w.ceil() as u32,
                base64(&png)
            ))
        }
        Protocol::Kitty => {
            let png = delta_export::png(svg, 2.0).ok()?;
            let data = base64(&png);
            // Sent in chunks of at most 4096 bytes; m=1 means "more follows".
            let chunks: Vec<&str> = data
                .as_bytes()
                .chunks(4096)
                .filter_map(|c| std::str::from_utf8(c).ok())
                .collect();
            let mut out = String::new();
            for (i, c) in chunks.iter().enumerate() {
                let more = u8::from(i + 1 < chunks.len());
                if i == 0 {
                    out.push_str(&format!("\x1b_Ga=T,f=100,m={more};{c}\x1b\\"));
                } else {
                    out.push_str(&format!("\x1b_Gm={more};{c}\x1b\\"));
                }
            }
            out.push('\n');
            Some(out)
        }
        Protocol::Sixel => {
            let (w, h, px) = delta_export::rgba(svg, 1.5).ok()?;
            let img = icy_sixel::SixelImage::from_rgba(px, w as usize, h as usize);
            img.encode().ok().map(|s| s + "\n")
        }
    }
}

/// Width and height from the `<svg width=… height=…>` that `delta-render` writes.
fn svg_size(svg: &str) -> (f64, f64) {
    let attr = |name: &str| -> f64 {
        let key = format!(" {name}=\"");
        svg.find(&key)
            .and_then(|i| svg.get(i + key.len()..))
            .and_then(|rest| rest.split('"').next())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0)
    };
    (attr("width"), attr("height"))
}

/// Prints a machine: as a picture if the terminal can, else as text.
pub fn show_machine(value: &serde_json::Value, protocol: Protocol, dark: bool) {
    if let Some(svg) = delta_lang::render::svg_of(value, dark)
        && let Some(seq) = image(&svg, protocol)
    {
        let mut out = std::io::stdout();
        let _ = out.write_all(seq.as_bytes());
        let _ = out.flush();
        return;
    }
    if let Some(text) = delta_lang::render::text_of(value) {
        print!("{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn protocols_come_from_the_environment() {
        assert_eq!(from_env("", "xterm-kitty", "", "", false), Protocol::Kitty);
        assert_eq!(
            from_env("", "xterm-256color", "iTerm.app", "", false),
            Protocol::Iterm
        );
        assert_eq!(from_env("", "foot", "", "", false), Protocol::Sixel);
        assert_eq!(
            from_env("", "xterm-256color", "Apple_Terminal", "", false),
            Protocol::None
        );
        assert_eq!(
            from_env("none", "xterm-kitty", "", "", true),
            Protocol::None
        );
        assert_eq!(from_env("sixel", "", "", "", false), Protocol::Sixel);
    }

    #[test]
    fn every_protocol_writes_its_escape() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="100%" height="100%" fill="#fff"/></svg>"##;
        assert!(
            image(svg, Protocol::Iterm)
                .is_some_and(|s| s.starts_with("\x1b]1337;File=") && s.contains("width=20px"))
        );
        assert!(image(svg, Protocol::Kitty).is_some_and(|s| s.starts_with("\x1b_Ga=T,f=100,m=0;")));
        assert!(image(svg, Protocol::Sixel).is_some_and(|s| s.starts_with("\x1bP")));
        assert!(image(svg, Protocol::None).is_none());
    }
}
