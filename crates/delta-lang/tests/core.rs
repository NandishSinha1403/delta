//! Language-core tests: semantics that the conformance suite doesn't pin down, and the rule
//! that no input, however broken, makes the engine panic.

use proptest::prelude::*;
use serde_json::Value;

/// Runs on a thread with a large stack, as the CLI does (debug builds use big frames).
fn run(src: &str) -> Value {
    let src = src.to_string();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || delta_lang::run(&src))
        .and_then(|h| {
            h.join()
                .map_err(|_| std::io::Error::other("engine thread panicked"))
        })
        .unwrap_or(Value::Null)
}

/// The `pass` field of every test output.
fn passes(src: &str) -> Vec<bool> {
    run(src)["outputs"]
        .as_array()
        .map(|o| o.iter().filter_map(|x| x["pass"].as_bool()).collect())
        .unwrap_or_default()
}

fn codes(src: &str) -> Vec<String> {
    run(src)["diagnostics"]
        .as_array()
        .map(|d| {
            d.iter()
                .filter_map(|x| x["code"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn closures_capture_values_not_variables() {
    let src = "var k = 1\nlet f = fn(x) = x + k\nk = 100\nassert f(1) == 101\nfn g() {\n  var j = 1\n  let h = fn(x) = x + j\n  j = 100\n  return h(1)\n}\nassert g() == 2";
    // Top-level names are globals and are read when the function runs; locals are captured.
    assert_eq!(passes(src), vec![true, true]);
}

#[test]
fn inner_blocks_rebind_outer_vars() {
    assert_eq!(
        passes("var n = 0\nfor i ∈ 0..3 { if i > 0 { n = n + i } }\nassert n == 3"),
        vec![true]
    );
}

#[test]
fn integer_division_is_euclidean() {
    assert_eq!(
        passes("assert -7 % 3 == 2\nassert -7 / 3 == -3\nassert 7 / 2 == 3"),
        vec![true, true, true]
    );
    assert_eq!(codes("let x = 1 / 0"), vec!["E502"]);
}

#[test]
fn deep_recursion_hits_the_depth_budget() {
    let out = run("fn f(n) = f(n + 1)\nlet x = f(0)");
    assert_eq!(out["diagnostics"][0]["code"], "B603");
    assert_eq!(out["ok"], false);
}

#[test]
fn infinite_loops_hit_the_step_budget() {
    assert_eq!(codes("budget { steps: 500 }\nwhile true { }"), vec!["B601"]);
}

#[test]
fn word_sets_and_builders() {
    let src = "let S = {'a', 'b'}\nassert len(S^≤2) == 7\nassert [ len(w) | w ∈ sort(list(S^2)) ] == [2, 2, 2, 2]\nassert { x | x ∈ 0..10, x % 3 == 0 } == {0, 3, 6, 9}";
    assert_eq!(passes(src), vec![true, true, true]);
}

#[test]
fn unknown_names_suggest_a_fix() {
    let out = run("let total = 1\nprint totl");
    assert_eq!(out["diagnostics"][0]["code"], "E201");
    assert_eq!(
        out["diagnostics"][0]["fixes"][0]["edits"][0]["insert"],
        "total"
    );
}

#[test]
fn let_cannot_be_rebound() {
    assert_eq!(codes("let x = 1\nx = 2"), vec!["E202"]);
    assert_eq!(codes("y = 2"), vec!["E203"]);
}

#[test]
fn one_bad_line_does_not_hide_the_others() {
    // Two independent syntax errors are both reported.
    assert_eq!(codes("let = 1\nlet y = (\nlet z = 3"), vec!["E109", "E110"]);
}

#[test]
fn two_statements_on_a_line_need_a_semicolon() {
    assert_eq!(codes("let x = 3        var y = 0"), vec!["E101"]);
    assert_eq!(passes("let x = 3; var y = x; assert y == 3"), vec![true]);
}

#[test]
fn spans_are_utf16_columns() {
    let out = run("let s = ∅ ∪ ∅; print ok");
    let d = &out["diagnostics"][0];
    assert_eq!(d["code"], "E201");
    assert_eq!(d["col"], 22);
}

#[test]
fn garbage_never_panics() {
    for src in [
        "",
        "{",
        "}",
        "(((",
        "\"",
        "'",
        "re\"",
        "dfa {",
        "dfa M { q0 -",
        "dfa M { [ }",
        "fn",
        "fn f(",
        "explore {",
        "fixpoint",
        "let (a, = 1",
        ":::",
        "q0 -[a-> q1",
        "⋃⋃⋃",
        "1..",
        "x[",
        "{a: }",
        "machine nfa {",
        "model m {",
        "cfg {",
        "use",
        "budget {",
        "check",
        "run",
        "lang L =",
        "/*",
        "for x ∈ { }",
        "if {",
        "\u{0}",
        "∁∁",
        "a ^ ≤",
        "[1 | ]",
        "{ | }",
    ] {
        let out = run(src);
        assert!(out["ok"].is_boolean(), "no result for {src:?}");
    }
}

proptest! {
    #[test]
    fn random_programs_never_panic(src in "[a-z0-9 ():;{}\\[\\],.=+*<>|'\"\\-\n∪∈εδ→_]{0,60}") {
        let out = run(&src);
        prop_assert!(out["ok"].is_boolean());
    }
}
