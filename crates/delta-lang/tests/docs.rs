//! Every Delta code block in the docs runs (04-ROADMAP.md §3, "Docs"). A ```delta block must run
//! with no error diagnostics and all its tests passing; a ```delta error block must report an
//! error (it documents one).

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.extension().is_some_and(|e| e == "md" || e == "mdx") {
            out.push(p);
        }
    }
}

/// (info string, code, line) of every fenced block whose language is `delta`.
fn blocks(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    let mut cur: Option<(String, String, usize)> = None;
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        match &mut cur {
            None if t.starts_with("```delta") => {
                cur = Some((t.trim_start_matches('`').to_string(), String::new(), i + 1))
            }
            Some(_) if t.starts_with("```") => out.extend(cur.take()),
            Some((_, code, _)) => {
                code.push_str(line);
                code.push('\n');
            }
            None => {}
        }
    }
    out
}

fn run(src: String) -> serde_json::Value {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || delta_lang::run(&src))
        .and_then(|h| {
            h.join()
                .map_err(|_| std::io::Error::other("engine thread panicked"))
        })
        .unwrap_or(serde_json::Value::Null)
}

#[test]
fn every_docs_code_block_runs() {
    let root = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../site/src/content/docs"
    ));
    let mut paths = Vec::new();
    files(root, &mut paths);
    let mut count = 0;
    let mut failures = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        for (info, code, line) in blocks(&text) {
            count += 1;
            let out = run(code.clone());
            let errors: Vec<&serde_json::Value> = out["diagnostics"]
                .as_array()
                .map(|d| d.iter().filter(|x| x["severity"] == "error").collect())
                .unwrap_or_default();
            let where_ = format!(
                "{}:{line}",
                path.strip_prefix(root).unwrap_or(&path).display()
            );
            if info.contains("outcome") {
                if out["diagnostics"].as_array().is_none_or(|d| d.is_empty()) {
                    failures.push(format!("{where_}: expected a budget outcome, got none"));
                }
                continue;
            }
            if info.contains("error") {
                if errors.is_empty() {
                    failures.push(format!("{where_}: expected an error, got none"));
                }
                continue;
            }
            if !errors.is_empty() || out["ok"] != true {
                failures.push(format!("{where_}: {}", out["diagnostics"]));
                continue;
            }
            for o in out["outputs"].as_array().into_iter().flatten() {
                if o["kind"] == "test" && o["pass"] != true {
                    failures.push(format!("{where_}: test failed: {o}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {count} docs blocks failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(count > 0, "no delta blocks found in the docs");
}
