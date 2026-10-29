//! Keeps `docs/DELTA-FOR-AI.md` honest: every code block in it is evaluated here.
//!
//! - ```` ```delta ```` blocks must have no errors or warnings, every `check` must pass, and each
//!   block must contain both `accepts` and `rejects` checks (the guide tells models to always
//!   write them, so its own examples do too).
//! - ```` ```delta-fails ```` blocks start with `// expect: CODE …` and must report those codes.

use crate::evaluate;

/// A fenced block of the guide: its info string, first line number and text.
struct Block {
    info: String,
    line: usize,
    text: String,
}

fn guide() -> String {
    let path = format!("{}/../../docs/DELTA-FOR-AI.md", env!("CARGO_MANIFEST_DIR"));
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
    assert!(open.is_none(), "unclosed code fence in the guide");
    out
}

#[test]
fn every_valid_block_in_the_guide_passes() {
    let all = blocks(&guide());
    let valid: Vec<&Block> = all.iter().filter(|b| b.info == "delta").collect();
    assert!(valid.len() >= 7, "found only {} delta blocks", valid.len());
    for b in valid {
        let doc = evaluate(&b.text);
        let problems: Vec<String> = doc
            .diagnostics
            .iter()
            .filter(|d| d.severity == "error" || d.severity == "warning")
            .map(|d| format!("{} {}", d.code, d.message))
            .collect();
        assert!(
            problems.is_empty(),
            "guide block at line {}: {problems:?}",
            b.line
        );
        let cases: Vec<_> = doc.results.iter().flat_map(|r| &r.cases).collect();
        assert!(
            cases.iter().all(|c| c.expected == c.actual),
            "guide block at line {}: a check fails",
            b.line
        );
        assert!(
            cases.iter().any(|c| c.expected) && cases.iter().any(|c| !c.expected),
            "guide block at line {} needs both `check … accepts` and `check … rejects`",
            b.line
        );
    }
}

#[test]
fn every_error_example_reports_its_code() {
    let all = blocks(&guide());
    let failing: Vec<&Block> = all.iter().filter(|b| b.info == "delta-fails").collect();
    assert!(
        failing.len() >= 4,
        "found only {} delta-fails blocks",
        failing.len()
    );
    for b in failing {
        let expect_line = b.text.lines().next().unwrap_or("");
        let codes: Vec<&str> = expect_line
            .strip_prefix("// expect:")
            .unwrap_or_else(|| panic!("block at line {} must start with `// expect: CODE`", b.line))
            .split_whitespace()
            .collect();
        let doc = evaluate(&b.text);
        let got: Vec<&str> = doc.diagnostics.iter().map(|d| d.code.as_str()).collect();
        for code in codes {
            assert!(
                got.contains(&code),
                "block at line {}: expected {code}, got {got:?}",
                b.line
            );
        }
    }
}

#[test]
fn the_guide_stays_compact_and_has_no_untested_code() {
    let md = guide();
    assert!(
        md.lines().count() <= 310,
        "the guide should stay around 300 lines"
    );
    for b in blocks(&md) {
        // Only tested block kinds may contain code (an untagged fence would escape the tests).
        assert!(
            b.info == "delta" || b.info == "delta-fails",
            "untested code block `{}` at line {}",
            b.info,
            b.line
        );
    }
}
