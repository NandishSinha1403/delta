//! The v2 conformance suite (docs/v2/04-ROADMAP.md §3, "Language"): every example in
//! docs/v2/01-LANGUAGE.md, run through `deltac run --json`, checked against its `.expect.json`.
//!
//! Cases live in `conformance/cases/NAME.delta` + `NAME.expect.json`. Matching is partial (see
//! `conformance/README.md`): only what the expectation states is checked.
//!
//! Cases listed in `conformance/xfail.txt` are *expected to fail*: the language isn't built yet.
//! - `cargo test` passes iff exactly the listed cases fail. A listed case that starts passing is
//!   reported, so the list only ever shrinks.
//! - `cargo test -p delta-cli --test conformance -- --ignored` requires every case to pass.

// `use` brings names from other modules into scope, like `import` in TypeScript.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A case may run this long before it is killed and counted as failed.
const TIMEOUT: Duration = Duration::from_secs(10);

fn root() -> PathBuf {
    // `env!` reads an environment variable at compile time; Cargo sets this one to the crate's folder.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance")
}

/// Every case name (file stem), sorted.
fn cases() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root().join("cases"))
        .expect("conformance/cases exists")
        // `.filter_map` keeps the `Some` results: here, file names ending in `.delta`.
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".delta").map(str::to_string)
        })
        .collect();
    names.sort();
    names
}

/// The names in `xfail.txt` (one per line; `#` starts a comment).
fn xfail() -> Vec<String> {
    let text = std::fs::read_to_string(root().join("xfail.txt")).unwrap_or_default();
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Runs one case. `Ok(())` if it matches its expectation, otherwise why not.
// `Result<(), String>`: either Ok(nothing) or Err(a message).
fn run_case(name: &str) -> Result<(), String> {
    let dir = root().join("cases");
    let expect_text = std::fs::read_to_string(dir.join(format!("{name}.expect.json")))
        .map_err(|e| format!("missing {name}.expect.json: {e}"))?;
    let expected: Value =
        serde_json::from_str(&expect_text).map_err(|e| format!("bad expect.json: {e}"))?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_deltac"))
        .args(["run", "--json"])
        .arg(dir.join(format!("{name}.delta")))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start delta: {e}"))?;
    let started = Instant::now();
    // Poll instead of blocking, so a hanging engine can be killed.
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > TIMEOUT => {
                let _ = child.kill();
                return Err(format!("timed out after {TIMEOUT:?}"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("wait failed: {e}")),
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("no output: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let actual: Value = serde_json::from_str(&stdout).map_err(|_| {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let first = stderr.lines().next().unwrap_or("").to_string();
        format!("stdout is not JSON (stderr: {first})")
    })?;
    matches(&expected, &actual, "$")
}

/// Partial match: does `actual` satisfy `expected`? See conformance/README.md for the rules.
fn matches(expected: &Value, actual: &Value, path: &str) -> Result<(), String> {
    match expected {
        // `null` means "absent or null".
        Value::Null => match actual {
            Value::Null => Ok(()),
            _ => Err(format!("{path}: expected null, got {actual}")),
        },
        Value::Object(exp) if exp.contains_key("$len") || exp.contains_key("$each") => {
            let arr = actual
                .as_array()
                .ok_or_else(|| format!("{path}: expected an array, got {actual}"))?;
            if let Some(n) = exp.get("$len").and_then(Value::as_u64)
                && arr.len() as u64 != n
            {
                return Err(format!("{path}: expected {n} items, got {}", arr.len()));
            }
            if let Some(each) = exp.get("$each") {
                for (i, a) in arr.iter().enumerate() {
                    matches(each, a, &format!("{path}[{i}]"))?;
                }
            }
            Ok(())
        }
        Value::Object(exp) => {
            let act = actual
                .as_object()
                .ok_or_else(|| format!("{path}: expected an object, got {actual}"))?;
            for (key, e) in exp {
                let a = act.get(key).unwrap_or(&Value::Null);
                matches(e, a, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        Value::Array(exp) => {
            let act = actual
                .as_array()
                .ok_or_else(|| format!("{path}: expected an array, got {actual}"))?;
            if exp.len() != act.len() {
                return Err(format!(
                    "{path}: expected {} items, got {}",
                    exp.len(),
                    act.len()
                ));
            }
            for (i, (e, a)) in exp.iter().zip(act).enumerate() {
                matches(e, a, &format!("{path}[{i}]"))?;
            }
            Ok(())
        }
        // A string ending in `*` is a prefix pattern ("E7*" matches "E701").
        Value::String(s) if s.ends_with('*') => match actual.as_str() {
            Some(a) if a.starts_with(&s[..s.len() - 1]) => Ok(()),
            _ => Err(format!("{path}: expected {s}, got {actual}")),
        },
        _ if expected == actual => Ok(()),
        _ => Err(format!("{path}: expected {expected}, got {actual}")),
    }
}

/// Runs every case and returns the failures as (name, reason).
fn failures() -> Vec<(String, String)> {
    cases()
        .into_iter()
        .filter_map(|name| run_case(&name).err().map(|why| (name, why)))
        .collect()
}

#[test]
fn matcher_follows_the_readme_rules() {
    // `serde_json::json!` builds a JSON value from JSON-like syntax.
    use serde_json::json;
    let ok = |e: Value, a: Value| matches(&e, &a, "$").is_ok();
    assert!(
        ok(json!({"a": 1}), json!({"a": 1, "b": 2})),
        "extra keys are ignored"
    );
    assert!(!ok(json!({"a": 1}), json!({"a": 2})));
    assert!(
        !ok(json!({"a": 1}), json!({})),
        "a stated key must be present"
    );
    assert!(ok(json!({"a": null}), json!({})), "null matches absent");
    assert!(ok(json!({"a": null}), json!({"a": null})));
    assert!(!ok(json!({"a": null}), json!({"a": 3})));
    assert!(
        !ok(json!([1]), json!([1, 2])),
        "arrays must have the same length"
    );
    assert!(ok(json!([{"k": 1}]), json!([{"k": 1, "x": 0}])));
    assert!(ok(json!({"$len": 2}), json!([0, 0])));
    assert!(!ok(json!({"$len": 2}), json!([0])));
    assert!(ok(
        json!({"$len": 2, "$each": {"p": true}}),
        json!([{"p": true}, {"p": true}])
    ));
    assert!(!ok(
        json!({"$each": {"p": true}}),
        json!([{"p": true}, {"p": false}])
    ));
    assert!(ok(json!("E7*"), json!("E701")));
    assert!(!ok(json!("E7*"), json!("E201")));
    assert!(!ok(json!("E7*"), json!(7)));
    assert!(!ok(json!(true), json!("true")), "no type coercion");
}

#[test]
fn every_case_has_an_expectation_and_xfail_names_real_cases() {
    let all = cases();
    assert!(!all.is_empty(), "no conformance cases found");
    for name in &all {
        let path = root().join("cases").join(format!("{name}.expect.json"));
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            serde_json::from_str::<Value>(&text).is_ok(),
            "{name}.expect.json is missing or not valid JSON"
        );
    }
    for name in xfail() {
        assert!(all.contains(&name), "xfail.txt lists unknown case {name}");
    }
}

#[test]
fn conformance_matches_the_xfail_list() {
    let expected_to_fail = xfail();
    let failed = failures();
    let mut problems = Vec::new();
    for (name, why) in &failed {
        if !expected_to_fail.contains(name) {
            problems.push(format!("FAIL  {name}: {why}"));
        }
    }
    for name in &expected_to_fail {
        if !failed.iter().any(|(n, _)| n == name) {
            problems.push(format!(
                "PASS  {name}: now passes, remove it from xfail.txt"
            ));
        }
    }
    let total = cases().len();
    println!(
        "conformance: {} of {total} cases pass, {} expected failures",
        total - failed.len(),
        expected_to_fail.len()
    );
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
#[ignore = "the v2 language is not implemented yet; run with --ignored to see every failure"]
fn conformance_all_cases_pass() {
    let failed = failures();
    let report: Vec<String> = failed
        .iter()
        .map(|(n, why)| format!("FAIL  {n}: {why}"))
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} cases fail:\n{}",
        failed.len(),
        cases().len(),
        report.join("\n")
    );
}
