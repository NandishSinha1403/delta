//! Machine and language tests: the generic simulator agrees with the finite-automaton fast
//! path (02 §2.2), a user-written `determinize` equals the built-in (04 §2, P3 exit), and the
//! engine has no textbook words (02 §1).

use proptest::prelude::*;
use serde_json::Value;

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

fn all_pass(out: &Value) -> bool {
    out["ok"] == true
        && out["outputs"].as_array().is_some_and(|o| {
            !o.is_empty() && o.iter().all(|x| x["kind"] != "test" || x["pass"] == true)
        })
}

/// An NFA literal from (from, symbol or ε, to) triples over states 0..n.
fn nfa_src(n: usize, edges: &[(usize, Option<u8>, usize)], accept: &[usize]) -> String {
    let mut s = String::from("nfa N {\n  alphabet a, b\n  start [0]\n");
    let acc: Vec<String> = accept.iter().map(|q| format!("[{q}]")).collect();
    if !acc.is_empty() {
        s.push_str(&format!("  accept {}\n", acc.join(", ")));
    }
    s.push_str(&format!(
        "  states {}\n",
        (0..n)
            .map(|q| format!("[{q}]"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for (p, a, q) in edges {
        let l = match a {
            Some(0) => "a",
            Some(_) => "b",
            None => "ε",
        };
        s.push_str(&format!("  [{p}] -{l}-> [{q}]\n"));
    }
    s.push_str("}\n");
    s
}

/// A random NFA: state count, edges and accepting states.
type RandomNfa = (usize, Vec<(usize, Option<u8>, usize)>, Vec<usize>);

fn edges() -> impl Strategy<Value = RandomNfa> {
    (1usize..5).prop_flat_map(|n| {
        (
            Just(n),
            prop::collection::vec((0..n, prop::option::weighted(0.8, 0u8..2), 0..n), 0..10),
            prop::collection::vec(0..n, 0..3),
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// `M.run(w).verdict` (generic BFS) agrees with `M.accepts(w)` (fast path) on all short words.
    #[test]
    fn generic_simulator_agrees_with_the_fast_path((n, es, acc) in edges()) {
        let mut src = nfa_src(n, &es, &acc);
        src.push_str("for w ∈ {'a','b'}^≤4 {\n  assert (N.run(w).verdict == :accept) == N.accepts(w)\n}\n");
        let out = run(&src);
        prop_assert!(all_pass(&out), "{src}\n{out}");
    }

    /// A determinize written with `explore` (01 §1) equals the built-in one.
    #[test]
    fn explore_determinize_equals_the_builtin((n, es, acc) in edges()) {
        let mut src = nfa_src(n, &es, &acc);
        src.push_str(
            "fn det(N) = explore {\n  alphabet N.alphabet\n  start  N.eclose({ N.start })\n  step   R on a -> N.eclose(⋃ { N.δ(q, a) | q ∈ R })\n  accept R if R ∩ N.accept ≠ ∅\n}\nlet D = det(N)\ncheck D ≡ determinize(N)\ncheck D ≡ N\nassert D.size == determinize(N).size\n",
        );
        let out = run(&src);
        prop_assert!(all_pass(&out), "{src}\n{out}");
    }

    /// The four minimization methods agree, and every method keeps the language.
    #[test]
    fn minimization_methods_agree((n, es, acc) in edges()) {
        let mut src = nfa_src(n, &es, &acc);
        src.push_str("let D = determinize(N)\nlet H = minimize(D, method: \"hopcroft\")\nfor m ∈ [\"moore\", \"brzozowski\", \"table\"] {\n  assert minimize(D, method: m).size == H.size\n}\ncheck H ≡ N\ncheck to_regex(H) ≡ N\ncheck to_nfa(to_regex(N), method: \"glushkov\") ≡ N\n");
        let out = run(&src);
        prop_assert!(all_pass(&out), "{src}\n{out}");
    }
}

#[test]
fn the_engine_has_no_textbook_words() {
    // delta-lang and the v2 algorithms in delta-core; citations live only in profiles/.
    let roots = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/src"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../delta-core/src/fa"),
    ];
    let words = [
        "Theorem",
        "Lemma",
        "Sipser",
        "Definition",
        "Corollary",
        "Hopcroft–Motwani",
    ];
    for root in roots {
        for entry in std::fs::read_dir(root).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs" || e == "delta") {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                for w in words {
                    assert!(
                        !text.contains(w),
                        "{} contains the textbook word {w:?}",
                        path.display()
                    );
                }
            }
        }
    }
}

#[test]
fn verdicts_and_traces() {
    let out = run(
        "dfa M {\n  start q0\n  accept q1\n  q0 -a-> q1\n  q1 -a-> q0\n}\nrun M on \"aa\"\nassert M.run(\"a\").verdict == :accept",
    );
    assert_eq!(out["outputs"][0]["kind"], "trace");
    assert_eq!(
        out["outputs"][0]["steps"].as_array().map(|s| s.len()),
        Some(3)
    );
    assert_eq!(out["outputs"][1]["pass"], true);
}

#[test]
fn explain_profiles_add_citations_only() {
    let src = "let N = to_nfa(re\"a b\")\nshow determinize(N)";
    let plain = run(src);
    let cited = run(&format!("use explain \"sipser3e\"\n{src}"));
    assert!(plain["outputs"][0]["value"]["cite"].is_null());
    assert!(
        cited["outputs"][0]["value"]["cite"]
            .as_str()
            .is_some_and(|c| c.contains("1.39"))
    );
    assert_eq!(
        plain["outputs"][0]["value"]["states"],
        cited["outputs"][0]["value"]["states"]
    );
}

/// A two-tape counter machine: tape 2 holds one 0 per unmatched 0 read, above a $ floor; each
/// 1 removes one; it accepts at the end of the input when the count is back to the floor.
const TWO_TAPE_EQUAL: &str = r#"model two { input on-tape; storage tapes(2); deterministic; accept by halting; missing reject }
two M {
  alphabet 0, 1
  start qs
  accept qa
  qs -on 2: write $; on 2: move R-> q
  q -on 1: read 0; on 1: move R; on 2: read ⊔; on 2: write 0; on 2: move R-> q
  q -on 1: read 0; on 1: move R; on 2: read 0; on 2: write 0; on 2: move R-> q
  q -on 1: read 1; on 2: move L-> d
  q -on 1: read ⊔; on 2: move L-> e
  d -on 1: read 1; on 2: read 0; on 2: write ⊔-> n
  n -on 1: move R-> q
  e -on 2: read $-> qa
}
"#;

#[test]
fn single_tape_machines_agree_with_their_multi_tape_originals() {
    // Every word up to length 8 gets the same verdict from both machines, and some are accepted.
    let src = format!(
        "{TWO_TAPE_EQUAL}let S = to_single_tape(M)\n\
         assert S.model == tm\n\
         assert [w | w ∈ {{'0','1'}}^≤8, S.accepts(w) != M.accepts(w)] == []\n\
         assert len([w | w ∈ {{'0','1'}}^≤8, S.accepts(w)]) >= 3\n"
    );
    let out = run(&src);
    assert!(all_pass(&out), "{out}");
}

#[test]
fn single_tape_leaves_one_tape_machines_alone_and_refuses_automata() {
    let out = run(
        "tm T {\n  alphabet 0\n  start q0\n  accept qa\n  q0 -0 → R-> q0\n  q0 -⊔ → R-> qa\n}\nassert to_single_tape(T) == T",
    );
    assert!(all_pass(&out), "{out}");
    let out = run("dfa D {\n  start q0\n  accept q0\n  q0 -a-> q0\n}\nlet S = to_single_tape(D)");
    assert_eq!(out["diagnostics"][0]["code"], "E504", "{out}");
}
