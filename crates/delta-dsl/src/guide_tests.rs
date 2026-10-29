//! Keeps the AI guides honest: every code block in `docs/DELTA-FOR-AI.md` and
//! `docs/DELTA-FOR-AI-SHORT.md` is evaluated here, so they can never drift from the language.
//!
//! - ```` ```delta ```` blocks must have no errors or warnings, every `check` must pass, and each
//!   block must follow the guide's own rule for answers: at least 5 `accepts` and 5 `rejects`
//!   cases, the empty string among them, and at least one `run`.
//! - ```` ```delta-fails ```` blocks start with `// expect: CODE …` and must report those codes.
//! - ```` ```text ```` blocks are prose templates; no other fences are allowed (untested code).

use crate::evaluate;

/// A fenced block of a guide: its info string, first line number and text.
struct Block {
    info: String,
    line: usize,
    text: String,
}

fn read(file: &str) -> String {
    let path = format!("{}/../../docs/{file}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn blocks(md: &str) -> Vec<Block> {
    let mut out = Vec::new();
    // `Option<Block>`: Some while we are inside a fence.
    let mut open: Option<Block> = None;
    for (i, line) in md.lines().enumerate() {
        match open.take() {
            None => {
                if let Some(info) = line.strip_prefix("```") {
                    open = Some(Block {
                        info: info.trim().to_string(),
                        line: i + 2,
                        text: String::new(),
                    });
                }
            }
            Some(mut b) => {
                if line.starts_with("```") {
                    out.push(b);
                } else {
                    b.text.push_str(line);
                    b.text.push('\n');
                    open = Some(b);
                }
            }
        }
    }
    assert!(open.is_none(), "unclosed code fence");
    out
}

/// Checks every block of one guide; returns how many valid Delta blocks it has.
fn check_guide(file: &str) -> usize {
    let all = blocks(&read(file));
    let mut valid = 0;
    for b in &all {
        let at = format!("{file} block at line {}", b.line);
        match b.info.as_str() {
            "delta" => {
                valid += 1;
                let doc = evaluate(&b.text);
                let problems: Vec<String> = doc
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == "error" || d.severity == "warning")
                    .map(|d| format!("{} {}", d.code, d.message))
                    .collect();
                assert!(problems.is_empty(), "{at}: {problems:?}");
                let cases: Vec<_> = doc.results.iter().flat_map(|r| &r.cases).collect();
                let failed: Vec<&str> = cases
                    .iter()
                    .filter(|c| c.expected != c.actual)
                    .map(|c| c.input.as_str())
                    .collect();
                assert!(failed.is_empty(), "{at}: checks fail on {failed:?}");
                let accepts = cases.iter().filter(|c| c.expected).count();
                let rejects = cases.len() - accepts;
                assert!(
                    accepts >= 5 && rejects >= 5,
                    "{at}: needs ≥5 accepts and ≥5 rejects (has {accepts} and {rejects})"
                );
                assert!(
                    cases.iter().any(|c| c.input.is_empty()),
                    "{at}: the empty string \"\" must be among the checks"
                );
                assert!(
                    doc.results.iter().any(|r| r.kind == "run"),
                    "{at}: needs at least one `run` line"
                );
            }
            "delta-fails" => {
                let first = b.text.lines().next().unwrap_or("");
                let codes: Vec<&str> = first
                    .strip_prefix("// expect:")
                    .unwrap_or_else(|| panic!("{at}: must start with `// expect: CODE`"))
                    .split_whitespace()
                    .collect();
                let doc = evaluate(&b.text);
                let got: Vec<&str> = doc.diagnostics.iter().map(|d| d.code.as_str()).collect();
                for code in codes {
                    assert!(got.contains(&code), "{at}: expected {code}, got {got:?}");
                }
            }
            "text" => {}
            other => panic!("{at}: untested code block `{other}` (use delta, delta-fails or text)"),
        }
    }
    valid
}

#[test]
fn every_block_of_the_full_guide_passes() {
    let valid = check_guide("DELTA-FOR-AI.md");
    assert!(
        valid >= 45,
        "the full guide should have its recipes and 25+ worked examples ({valid} blocks)"
    );
    let lines = read("DELTA-FOR-AI.md").lines().count();
    assert!(
        (1200..=3000).contains(&lines),
        "the full guide has {lines} lines"
    );
}

#[test]
fn every_block_of_the_short_guide_passes() {
    let valid = check_guide("DELTA-FOR-AI-SHORT.md");
    assert!(valid >= 3, "the short guide needs its 3 examples");
    let lines = read("DELTA-FOR-AI-SHORT.md").lines().count();
    assert!(
        lines <= 200,
        "the short guide is for small contexts ({lines} lines)"
    );
}
