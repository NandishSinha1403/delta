//! End-to-end tests of the language: source text in, `Document` out.

use crate::document::Document;
use crate::evaluate;

fn codes(doc: &Document) -> Vec<&str> {
    doc.diagnostics.iter().map(|d| d.code.as_str()).collect()
}

fn errors(doc: &Document) -> Vec<String> {
    doc.diagnostics
        .iter()
        .filter(|d| d.severity == "error")
        .map(|d| format!("{} {}", d.code, d.message))
        .collect()
}

fn object<'a>(doc: &'a Document, name: &str) -> &'a crate::document::ObjectView {
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("no object {name}: {:?}", errors(doc)))
}

fn read_example(file: &str) -> String {
    let path = format!("{}/../../examples/{file}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every example in `examples/` evaluates without errors and all its checks pass.
#[test]
fn golden_examples_have_no_errors_and_pass_their_checks() {
    let dir = format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"));
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("delta") {
            continue;
        }
        count += 1;
        let doc = evaluate(&std::fs::read_to_string(&path).unwrap());
        assert!(
            errors(&doc).is_empty(),
            "{}: {:?}",
            path.display(),
            errors(&doc)
        );
        for r in &doc.results {
            for c in &r.cases {
                assert_eq!(
                    c.expected,
                    c.actual,
                    "{}: check {} on {:?}",
                    path.display(),
                    r.object,
                    c.input
                );
            }
        }
    }
    assert!(count >= 7, "expected the book examples in {dir}");
}

#[test]
fn example_1_41_gives_figure_1_44_and_figure_1_43() {
    let doc = evaluate(&read_example("ex1-41-N4.delta"));
    let d4 = object(&doc, "D4").machine.as_ref().unwrap();
    assert_eq!(d4.states.len(), 6);
    let start = d4.states.iter().find(|s| s.start).unwrap();
    assert_eq!(start.name, "{1,3}");
    let full = object(&doc, "D4full").machine.as_ref().unwrap();
    assert_eq!(full.states.len(), 8);
    assert!(object(&doc, "D4").trace.as_ref().unwrap().steps.len() > 10);
}

#[test]
fn examples_1_56_and_1_58_have_the_books_state_counts() {
    let doc = evaluate(&read_example("ex1-56.delta"));
    assert_eq!(object(&doc, "N").machine.as_ref().unwrap().states.len(), 8);
    let doc = evaluate(&read_example("ex1-58.delta"));
    assert_eq!(object(&doc, "N").machine.as_ref().unwrap().states.len(), 12);
}

#[test]
fn example_1_66_gives_the_books_regex() {
    let doc = evaluate(&read_example("ex1-66.delta"));
    assert_eq!(object(&doc, "R").regex.as_deref(), Some("a*b(a∪b)*"));
    assert_eq!(object(&doc, "G").machine.as_ref().unwrap().states.len(), 4);
}

#[test]
fn the_three_delta_forms_are_equivalent() {
    let table = "dfa M { Q = {p, q}\n Σ = {0, 1}\n δ = table {\n | 0 | 1\n p | q | p\n q | q | p\n }\n q0 = p\n F = {q} }";
    let arrows = "dfa M { Q = {p, q}\n Σ = {0, 1}\n δ {\n p -0-> q\n p -1-> p\n q -0-> q\n q -1-> p\n }\n q0 = p\n F = {q} }";
    let func = "dfa M { Q = {p, q}\n Σ = {0, 1}\n δ(p, 0) = q\n δ(p, 1) = p\n δ(q, 0) = q\n δ(q, 1) = p\n q0 = p\n F = {q} }";
    let views: Vec<_> = [table, arrows, func]
        .iter()
        .map(|src| {
            let doc = evaluate(src);
            assert!(errors(&doc).is_empty(), "{src}: {:?}", errors(&doc));
            object(&doc, "M").machine.clone().unwrap()
        })
        .collect();
    assert_eq!(views[0], views[1]);
    assert_eq!(views[1], views[2]);
}

#[test]
fn missing_transition_is_an_error_in_strict_mode_and_a_warning_in_lenient_mode() {
    let body = "dfa M {\n Q = {p, q}\n Σ = {0, 1}\n δ {\n p -0-> q\n p -1-> p\n q -0-> q\n }\n q0 = p\n F = {q}\n}";
    let strict = evaluate(&format!("#mode strict\n{body}"));
    assert!(codes(&strict).contains(&"E201"), "{:?}", codes(&strict));
    assert!(strict.objects.is_empty());
    let lenient = evaluate(body);
    assert!(codes(&lenient).contains(&"W201"));
    let m = object(&lenient, "M").machine.as_ref().unwrap();
    assert!(m.states.iter().any(|s| s.trap), "a trap state completes δ");
    let w = lenient
        .diagnostics
        .iter()
        .find(|d| d.code == "W201")
        .unwrap();
    assert!(w.fixes.iter().any(|f| f.title.contains("partial")));
    assert!(w.fixes.iter().any(|f| f.title.contains("trap")));
    let marked = evaluate(&body.replace("dfa M {", "dfa M partial {"));
    assert!(!codes(&marked).contains(&"W201"));
}

#[test]
fn dfa_rule_violations() {
    let two =
        evaluate("dfa M { Q = {p}\n Σ = {0}\n δ {\n p -0-> p\n p -0-> q\n }\n q0 = p\n F = {p} }");
    assert!(codes(&two).contains(&"E104"), "{:?}", codes(&two));
    let eps =
        evaluate("dfa M { Q = {p}\n Σ = {0}\n δ {\n p -ε-> p\n p -0-> p\n }\n q0 = p\n F = {p} }");
    assert!(codes(&eps).contains(&"E203"));
    let fix = &eps
        .diagnostics
        .iter()
        .find(|d| d.code == "E203")
        .unwrap()
        .fixes[0];
    assert_eq!(fix.edits[0].insert, "nfa");
    let dup = evaluate(
        "dfa M { Q = {p, q}\n Σ = {0}\n δ {\n p -0-> p\n p -0-> q\n q -0-> q\n }\n q0 = p\n F = {p} }",
    );
    assert!(codes(&dup).contains(&"E202"));
    let sym =
        evaluate("dfa M { Q = {p}\n Σ = {0}\n δ {\n p -0-> p\n p -1-> p\n }\n q0 = p\n F = {p} }");
    let e105 = sym.diagnostics.iter().find(|d| d.code == "E105").unwrap();
    assert_eq!(e105.fixes[0].edits[0].insert, ", 1 ");
}

#[test]
fn unknown_state_fix_adds_it_to_q() {
    let src = "nfa N {\n Q = { a }\n Σ = { 0 }\n δ { a -0-> b }\n q0 = a\n F = { b }\n}";
    let doc = evaluate(src);
    let d = doc
        .diagnostics
        .iter()
        .find(|d| d.fixes.iter().any(|f| f.title == "Add `b` to Q"))
        .unwrap();
    let edit = &d.fixes[0].edits[0];
    // Offsets are UTF-16 (what the editor uses), so apply the edit on UTF-16 code units.
    let mut units: Vec<u16> = src.encode_utf16().collect();
    units.splice(
        edit.from as usize..edit.to as usize,
        edit.insert.encode_utf16(),
    );
    let fixed = String::from_utf16(&units).unwrap();
    assert!(errors(&evaluate(&fixed)).is_empty(), "{fixed}");
}

#[test]
fn regex_shorthands_and_the_plus_warning() {
    let doc = evaluate(
        "regex R over {0, 1} = Σ* 001 Σ*\nregex P over {a, b} = a+b\nregex Q over {0,1} = (ΣΣ)^3",
    );
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    assert!(codes(&doc).contains(&"W405"));
    assert_eq!(object(&doc, "R").regex.as_deref(), Some("(0∪1)*001(0∪1)*"));
}

#[test]
fn regex_errors() {
    let doc = evaluate("regex R over {a} = (a ∪ b\nregex S = a ∪\nregex T = ()");
    let c = codes(&doc);
    assert!(c.contains(&"E401"), "{c:?}");
    assert!(c.contains(&"E402"), "{c:?}");
    assert!(c.contains(&"E403"), "{c:?}");
}

#[test]
fn input_symbols_must_be_in_sigma() {
    let doc = evaluate(&format!(
        "{}\nrun M1 on \"012\"",
        read_example("fig1-6-M1.delta")
    ));
    let d = doc.diagnostics.iter().find(|d| d.code == "R901").unwrap();
    assert!(d.message.contains("`2`"));
}

#[test]
fn unknown_names_get_suggestions_and_dependents_are_skipped() {
    let doc = evaluate(
        "nfa N1 { Q = {p}\n Σ = {a}\n δ { p -a-> p }\n q0 = p\n F = {p} }\nlet D = dfa(N2)",
    );
    let d = doc.diagnostics.iter().find(|d| d.code == "E802").unwrap();
    assert!(d.message.contains("Did you mean `N1`"), "{}", d.message);
    let doc =
        evaluate("nfa N { Q = {p}\n Σ = {a}\n δ { p -a-> x }\n q0 = p\n F = {p} }\nlet D = dfa(N)");
    assert!(codes(&doc).contains(&"I811"));
}

#[test]
fn syntax_errors_recover_and_the_rest_of_the_file_still_works() {
    let src = format!(
        "dfa Broken {{ Q = {{ p \n Sigam = {{0}} }}\n\n{}",
        read_example("fig1-6-M1.delta")
    );
    let doc = evaluate(&src);
    assert!(codes(&doc).contains(&"E122"));
    assert!(
        doc.objects.iter().any(|o| o.name == "M1"),
        "M1 still builds: {:?}",
        errors(&doc)
    );
}

#[test]
fn spans_are_utf16() {
    // "Σ" is 2 bytes in UTF-8 but 1 unit in UTF-16; the E105 squiggle must land on `x`.
    let src = "dfa M { Q = {p}\n Σ = {0}\n δ { p -x-> p }\n q0 = p\n F = {p} }";
    let doc = evaluate(src);
    let d = doc.diagnostics.iter().find(|d| d.code == "E105").unwrap();
    let utf16: Vec<u16> = src.encode_utf16().collect();
    let text = String::from_utf16(&utf16[d.from as usize..d.to as usize]).unwrap();
    assert_eq!(text, "x");
}

#[test]
fn nothing_panics_on_garbage() {
    for src in [
        "",
        "}",
        "dfa",
        "dfa M {",
        "regex R = ((((",
        "let = (",
        "δ(",
        "\"",
        "/*",
        "dfa M { δ = table { | | } }",
        "#mode",
        "run M on",
    ] {
        let _ = evaluate(src);
    }
    let deep = format!("regex R = {}a{}", "(".repeat(5000), ")".repeat(5000));
    assert!(codes(&evaluate(&deep)).contains(&"E408"));
    let stars = format!("regex R = a{}", "*".repeat(5000));
    let _ = evaluate(&stars);
}

#[test]
fn run_input_from_the_ui() {
    let src = read_example("ex1-38-N1.delta");
    let r = crate::run_input(&src, "N1", "010110");
    assert!(r.run.unwrap().accepted);
    let bad = crate::run_input(&src, "N1", "012");
    assert!(bad.error.unwrap().contains("`2`"));
}
