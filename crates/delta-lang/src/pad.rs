//! What the Pad shows (03-PRODUCTS.md §3): the program's named objects as drawable views, its
//! diagnostics and tests, its outline (for completion), and interactive runs. Built from one
//! evaluation, as plain JSON for the Web Worker.

use crate::ast::*;
use crate::diag::{Diagnostic, LineMap};
use crate::hooks::{TraceVal, word_text};
use crate::interp::Interp;
use crate::lang::LangVal;
use crate::machine::Machine;
use crate::rx::RegexVal;
use crate::value::{Sym, Value, sym_text};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::rc::Rc;

fn u16(src: &str, byte: usize) -> usize {
    LineMap::utf16(src, byte)
}

/// A machine as the diagram and table draw it.
pub fn machine_view(m: &Machine) -> Json {
    let names: Vec<String> = m.states.iter().map(Machine::display_state).collect();
    let states: Vec<Json> = (0..m.states.len())
        .map(|q| {
            json!({
                "name": names.get(q).cloned().unwrap_or_default(),
                "start": q == m.start,
                "accept": m.is_accepting(q) && m.eff.accept != crate::model::AcceptMode::Unstated,
                "trap": false,
                "origin": Json::Null,
            })
        })
        .collect();
    // Parallel edges become one arrow with the labels joined.
    let mut merged: BTreeMap<(usize, usize), Vec<String>> = BTreeMap::new();
    for e in &m.edges {
        merged
            .entry((e.from, e.to))
            .or_default()
            .push(m.label_text(&e.label));
    }
    let edges: Vec<Json> = merged
        .iter()
        .map(|((f, t), ls)| json!({ "from": f, "to": t, "label": ls.join(if m.is_fa() { "," } else { "\n" }) }))
        .collect();
    let sigma: Vec<String> = m.sigma.iter().map(|s| sym_text(s)).collect();
    // The table: one column per symbol (and ε) for finite automata, one cell list otherwise.
    let has_eps = m.edges.iter().any(|e| e.label.ops.is_empty());
    let (columns, rows) = if m.is_fa() {
        let mut cols: Vec<String> = sigma.clone();
        if has_eps {
            cols.push("ε".to_string());
        }
        let rows: Vec<Json> = (0..m.states.len())
            .map(|q| {
                let cells: Vec<String> = cols
                    .iter()
                    .map(|c| {
                        let targets: Vec<String> = m
                            .out_edges(q)
                            .iter()
                            .filter_map(|i| m.edges.get(*i))
                            .filter(|e| match e.label.read() {
                                Some(s) => sym_text(s) == *c,
                                None => c == "ε",
                            })
                            .map(|e| names.get(e.to).cloned().unwrap_or_default())
                            .collect();
                        if m.eff.deterministic && targets.len() <= 1 {
                            targets.first().cloned().unwrap_or_else(|| "—".to_string())
                        } else if targets.is_empty() {
                            "∅".to_string()
                        } else {
                            format!("{{{}}}", targets.join(", "))
                        }
                    })
                    .collect();
                json!({ "state": names.get(q).cloned().unwrap_or_default(), "cells": cells })
            })
            .collect();
        (cols, rows)
    } else {
        let rows: Vec<Json> = (0..m.states.len())
            .map(|q| {
                let cells: Vec<String> = m
                    .out_edges(q)
                    .iter()
                    .filter_map(|i| m.edges.get(*i))
                    .map(|e| format!("{} → {}", m.label_text(&e.label), names.get(e.to).cloned().unwrap_or_default()))
                    .collect();
                json!({ "state": names.get(q).cloned().unwrap_or_default(), "cells": [cells.join("; ")] })
            })
            .collect();
        (vec!["moves".to_string()], rows)
    };
    json!({
        "kind": m.model.name,
        "model": m.model.display(),
        "states": states,
        "sigma": sigma,
        "edges": edges,
        "table": { "columns": columns, "rows": rows },
        "groups": [],
    })
}

fn trace_view(m: &Machine, cite: Option<String>) -> Json {
    let Some(p) = &m.provenance else {
        return Json::Null;
    };
    let title = match cite {
        Some(c) => format!("{} · {c}", p.algorithm),
        None => p.algorithm.clone(),
    };
    let steps: Vec<Json> = p
        .steps
        .iter()
        .take(500)
        .map(|s| {
            json!({ "rule": s.kind, "title": s.kind, "message": s.text, "add_states": [], "add_edges": [],
                    "focus": [], "start": Json::Null, "accept": Json::Null, "snapshot": Json::Null })
        })
        .collect();
    json!({ "title": title, "steps": steps, "truncated": p.steps.len() > 500 })
}

/// The step budget for checking one sample word in `lang_info`.
const PREVIEW_STEPS: u64 = 1_000;

/// Sample words in and out of a language (up to length 3) and which questions are decidable.
fn lang_info(it: &mut Interp, v: &Value) -> Json {
    if !Interp::is_langlike(v) {
        return Json::Null;
    }
    let Ok(l) = it.to_lang(v, crate::diag::Span::default()) else {
        return Json::Null;
    };
    let syms: Vec<Sym> = l.sigma.iter().cloned().collect();
    let (mut ins, mut outs) = (Vec::new(), Vec::new());
    let mut layer: Vec<Vec<Sym>> = vec![Vec::new()];
    let saved = (it.steps, it.budget);
    // A preview, so each sample word gets a small step budget of its own, never more than the
    // program's: a machine that runs forever (e.g. a TM moving right over blanks) costs little.
    it.budget.steps = it.steps + PREVIEW_STEPS.min(saved.1.steps);
    'outer: for _ in 0..=3 {
        for w in &layer {
            match it.lang_member(&l, w, crate::diag::Span::default()) {
                Ok(Some(true)) if ins.len() < 12 => ins.push(word_text(w)),
                Ok(Some(false)) if outs.len() < 12 => outs.push(word_text(w)),
                Ok(_) => {}
                Err(_) => break 'outer,
            }
        }
        layer = layer
            .iter()
            .flat_map(|w| {
                syms.iter().map(move |s| {
                    let mut x = w.clone();
                    x.push(s.clone());
                    x
                })
            })
            .collect();
    }
    it.steps = saved.0;
    it.budget = saved.1;
    use crate::lang::{Decidability, Question, decidable};
    let questions: Vec<Json> = [
        ("w ∈ L", None),
        ("L.is_empty()", Some(Question::IsEmpty)),
        ("L.is_finite()", Some(Question::IsFinite)),
        ("L.is_universal()", Some(Question::IsUniversal)),
        ("L ≡ K", Some(Question::Equiv)),
        ("L ⊆ K", Some(Question::Subset)),
    ]
    .iter()
    .map(|(name, q)| {
        let d = match q {
            None => "yes",
            Some(q) => match decidable(*q, l.class) {
                Decidability::Yes => "yes",
                Decidability::NotImplemented => "not-implemented",
                Decidability::No => "no",
            },
        };
        json!({ "name": name, "decidable": d })
    })
    .collect();
    json!({ "in": ins, "out": outs, "questions": questions })
}

/// A named value of the program as a Pad object, if it is something to show.
fn object_view(it: &Interp, name: &str, v: &Value, from: usize, to: usize) -> Option<Json> {
    if let Some(m) = v.downcast::<Machine>() {
        let cite = m
            .provenance
            .as_ref()
            .and_then(|p| it.citation(&p.algorithm));
        let derived = m.provenance.as_ref().map(|p| match &cite {
            Some(c) => format!("{} · {c}", p.algorithm),
            None => p.algorithm.clone(),
        });
        return Some(json!({
            "name": name, "kind": m.model.name, "from": from, "to": to,
            "derived": derived, "machine": machine_view(m), "regex": Json::Null,
            "sigma": m.sigma.iter().map(|s| sym_text(s)).collect::<Vec<_>>(),
            "trace": trace_view(m, cite), "class": crate::lang::model_class(&m.eff).name(),
        }));
    }
    if let Some(r) = v.downcast::<RegexVal>() {
        return Some(json!({
            "name": name, "kind": "regex", "from": from, "to": to, "derived": Json::Null,
            "machine": Json::Null, "regex": r.rx.text(),
            "sigma": r.alphabet.iter().map(|s| sym_text(s)).collect::<Vec<_>>(), "trace": Json::Null,
            "class": "Regular",
        }));
    }
    if let Some(l) = v.downcast::<LangVal>() {
        return Some(json!({
            "name": name, "kind": "lang", "from": from, "to": to, "derived": l.describe(),
            "machine": Json::Null, "regex": Json::Null,
            "sigma": l.sigma.iter().map(|s| sym_text(s)).collect::<Vec<_>>(), "trace": Json::Null,
            "class": l.class.name(), "assumed": l.assumed,
        }));
    }
    if let Some(g) = v.downcast::<crate::grammar::Grammar>() {
        return Some(json!({
            "name": name, "kind": "cfg", "from": from, "to": to, "derived": Json::Null,
            "machine": Json::Null, "regex": g.text(),
            "sigma": g.terminals.iter().map(|s| sym_text(s)).collect::<Vec<_>>(), "trace": Json::Null,
            "class": "CF",
        }));
    }
    None
}

/// The names a statement declares at top level, with what kind of thing they are.
fn declared(s: &Stmt) -> Option<(String, &'static str)> {
    match &s.kind {
        StmtKind::Let(Pattern::Var(n, _), e) => Some((n.clone(), expr_kind(e))),
        StmtKind::Var(n, e) => Some((n.clone(), expr_kind(e))),
        StmtKind::MachineDecl(n, _, e) => Some((n.clone(), expr_kind(e))),
        StmtKind::Lang { name, .. } => Some((name.clone(), "lang")),
        StmtKind::Fn(def) => def.name.clone().map(|n| (n, "fn")),
        StmtKind::Model(m) => Some((m.name.clone(), "model")),
        _ => None,
    }
}

fn expr_kind(e: &Expr) -> &'static str {
    match &e.kind {
        ExprKind::Machine(_) | ExprKind::Canonical(_) | ExprKind::Explore(_) => "machine",
        ExprKind::Regex { .. } => "regex",
        ExprKind::Cfg { .. } => "cfg",
        ExprKind::Lambda(_) => "fn",
        _ => "let",
    }
}

/// State names and symbols written in a machine literal (for completion).
fn literal_names(e: &Expr) -> (Vec<String>, Vec<String>) {
    let mut states = Vec::new();
    let mut syms = Vec::new();
    fn walk(items: &[MItem], states: &mut Vec<String>, syms: &mut Vec<String>) {
        for it in items {
            match it {
                MItem::Edge {
                    from, to, label, ..
                } => {
                    for s in [from, to] {
                        if let MState::Name(n, _) = s
                            && !states.contains(n)
                        {
                            states.push(n.clone());
                        }
                    }
                    for c in label.text.split(|c: char| c == ',' || c.is_whitespace()) {
                        if c.chars().count() == 1 && !syms.iter().any(|x| x == c) {
                            syms.push(c.to_string());
                        }
                    }
                }
                MItem::For { body, .. } => walk(body, states, syms),
                MItem::If { then, els, .. } => {
                    walk(then, states, syms);
                    walk(els, states, syms);
                }
                _ => {}
            }
        }
    }
    if let ExprKind::Machine(m) = &e.kind {
        walk(&m.items, &mut states, &mut syms);
    }
    (states, syms)
}

/// The Pad's document for `src`.
pub fn document(src: &str) -> Json {
    let mut ex = crate::execute(src, &crate::RunOptions::default());
    let lines = LineMap::new(src);
    let diagnostics: Vec<Json> = ex
        .diags
        .iter()
        .map(|d: &Diagnostic| {
            let j = d.to_json(src, &lines);
            json!({ "code": j.code, "severity": j.severity, "from": j.from, "to": j.to, "line": j.line, "col": j.col,
                    "message": j.message, "book": Json::Null, "fixes": j.fixes })
        })
        .collect();
    let mut objects = Vec::new();
    let mut outline = Vec::new();
    for s in &ex.program.stmts {
        let Some((name, kind)) = declared(s) else {
            continue;
        };
        let (from, to) = (u16(src, s.span.from), u16(src, s.span.to));
        let (states, symbols) = match &s.kind {
            StmtKind::MachineDecl(_, _, e) | StmtKind::Let(_, e) => literal_names(e),
            _ => (Vec::new(), Vec::new()),
        };
        outline.push(json!({ "name": name, "kind": kind, "from": from, "to": to, "states": states, "symbols": symbols }));
        if let Some(it) = &mut ex.interp
            && let Some(b) = it.globals.get(&name).cloned()
            && let Some(mut o) = object_view(it, &name, &b.value, from, to)
        {
            o["lang"] = lang_info(it, &b.value);
            objects.push(o);
        }
    }
    // Tests, prints and the `check accepts/rejects` and `run` results the Tests and Run views use.
    let mut results = Vec::new();
    let mut tests = Vec::new();
    let mut prints = Vec::new();
    let mut by_line: BTreeMap<u64, Vec<&Json>> = BTreeMap::new();
    // manual:begin pad-tests
    for o in &ex.outputs {
        if let Some(l) = o["line"].as_u64() {
            by_line.entry(l).or_default().push(o);
        }
        match o["kind"].as_str() {
            Some("test") => tests.push(o.clone()),
            Some("print") => prints.push(o.clone()),
            _ => {}
        }
    }
    // manual:end pad-tests
    for s in &ex.program.stmts {
        let line = lines.line_col(src, s.span.from).0 as u64;
        let (from, to) = (u16(src, s.span.from), u16(src, s.span.to));
        match &s.kind {
            StmtKind::Check(subject, k @ (CheckKind::Accepts(ws) | CheckKind::Rejects(ws))) => {
                let want = matches!(k, CheckKind::Accepts(_));
                let outs = by_line.get(&line).cloned().unwrap_or_default();
                let cases: Vec<Json> = ws
                    .iter()
                    .zip(outs.iter())
                    .map(|(w, o)| {
                        let pass = o["pass"] == true;
                        let input = o["name"].as_str().and_then(|n| n.rsplit_once(" \"")).map(|(_, x)| x.trim_end_matches('"').to_string()).unwrap_or_default();
                        json!({ "input": input, "expected": want, "actual": if pass { want } else { !want },
                                "from": u16(src, w.span.from), "to": u16(src, w.span.to) })
                    })
                    .collect();
                let object = src
                    .get(subject.span.from..subject.span.to)
                    .unwrap_or("")
                    .to_string();
                results.push(json!({ "kind": "check", "object": object, "from": from, "to": to, "cases": cases, "run": Json::Null }));
            }
            StmtKind::Run(m, w) => {
                let object = src.get(m.span.from..m.span.to).unwrap_or("").to_string();
                let input = src
                    .get(w.span.from..w.span.to)
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string();
                let run = run_view_named(&ex.interp, &object, &input);
                results.push(json!({ "kind": "run", "object": object, "from": from, "to": to, "cases": [], "run": run }));
            }
            _ => {}
        }
    }
    json!({
        "objects": objects,
        "diagnostics": diagnostics,
        "results": results,
        "outline": outline,
        "mode": if src.contains("use strict") { "strict" } else { "lenient" },
        "tests": tests,
        "prints": prints,
        "outputs": ex.outputs,
        "ok": ex.ok,
        "steps": ex.steps,
    })
}

fn run_view_named(it: &Option<Interp>, name: &str, input: &str) -> Json {
    let Some(it) = it else { return Json::Null };
    let Some(m) = it.globals.get(name).and_then(|b| b.value.rc::<Machine>()) else {
        return Json::Null;
    };
    let w = parse_input(input);
    run_view(it, name, &m, &w).unwrap_or(Json::Null)
}

/// The input box's text as symbols (`<NAME>` is one symbol; spaces are ignored).
pub fn parse_input(s: &str) -> Vec<Sym> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        if c == '<' {
            let name: String = chars.by_ref().take_while(|c| *c != '>').collect();
            out.push(Sym::from(name.as_str()));
        } else if c != 'ε' {
            out.push(Sym::from(c.to_string().as_str()));
        }
    }
    out
}

/// A step-by-step run. Finite automata: the active states after each symbol (ε-closed).
/// Other models: the configurations of each breadth-first step.
fn run_view(it: &Interp, name: &str, m: &Rc<Machine>, w: &[Sym]) -> Option<Json> {
    let names: Vec<String> = m.states.iter().map(Machine::display_state).collect();
    let set_text = |s: &[usize]| -> String {
        let n: Vec<String> = s.iter().filter_map(|q| names.get(*q).cloned()).collect();
        if m.eff.deterministic && n.len() == 1 {
            n.join("")
        } else {
            format!("{{{}}}", n.join(", "))
        }
    };
    let mut steps = Vec::new();
    let accepted;
    if m.is_fa() {
        let close = |set: &mut Vec<usize>| {
            let mut i = 0;
            while let Some(&q) = set.get(i) {
                for &ei in m.out_edges(q) {
                    if let Some(e) = m.edges.get(ei)
                        && e.label.ops.is_empty()
                        && !set.contains(&e.to)
                    {
                        set.push(e.to);
                    }
                }
                i += 1;
            }
            set.sort_unstable();
        };
        let mut cur = vec![m.start];
        close(&mut cur);
        steps.push(
            json!({ "pos": 0, "active": cur, "message": format!("Start in {}.", set_text(&cur)),
                           "explain": format!("The machine starts in {}.", set_text(&cur)) }),
        );
        for (i, a) in w.iter().enumerate() {
            let mut next = Vec::new();
            for &q in &cur {
                for &ei in m.out_edges(q) {
                    if let Some(e) = m.edges.get(ei)
                        && e.label.read() == Some(a)
                        && !next.contains(&e.to)
                    {
                        next.push(e.to);
                    }
                }
            }
            close(&mut next);
            let a_text = sym_text(a);
            let explain = if next.is_empty() {
                format!("No state has an arrow for {a_text}, so the run dies.")
            } else {
                format!("Reading {a_text} moves to {}.", set_text(&next))
            };
            steps.push(json!({ "pos": i + 1, "active": next, "message": format!("Read {a_text}: {} → {}", set_text(&cur), set_text(&next)), "explain": explain }));
            cur = next;
        }
        accepted = cur.iter().any(|q| m.is_accepting(*q));
    } else {
        let r = m.run(w, it.budget.steps, true).ok()?;
        accepted = r.verdict == crate::sim::Verdict::Accept;
        for (i, layer) in r.steps.iter().enumerate() {
            let active: Vec<usize> = layer
                .iter()
                .filter_map(|c| {
                    let first = c.split([' ', '|']).next().unwrap_or("");
                    names.iter().position(|n| n == first)
                })
                .collect();
            steps.push(json!({ "pos": i.min(w.len()), "active": active, "message": layer.join("   "), "explain": layer.join("; ") }));
        }
        steps.push(json!({ "pos": w.len(), "active": [], "message": format!("Verdict: {}", r.verdict.name()), "explain": format!("The run's verdict is {}.", r.verdict.name()) }));
    }
    Some(json!({
        "object": name,
        "kind": m.model.name,
        "input": w.iter().map(|s| sym_text(s)).collect::<Vec<_>>(),
        "steps": steps,
        "accepted": accepted,
    }))
}

/// An interactive run from the Run panel: machine `name` of the program on `input`.
pub fn run(src: &str, name: &str, input: &str) -> Json {
    let ex = crate::execute(src, &crate::RunOptions::default());
    let Some(it) = &ex.interp else {
        return json!({ "run": Json::Null, "error": "The program has errors; fix them first." });
    };
    let Some(m) = it.globals.get(name).and_then(|b| b.value.rc::<Machine>()) else {
        return json!({ "run": Json::Null, "error": format!("`{name}` is not a machine.") });
    };
    let w = parse_input(input);
    if let Some(bad) = w.iter().find(|s| !m.sigma.contains(*s)) {
        return json!({ "run": Json::Null, "error": format!("'{}' is not in the alphabet {{{}}}.", sym_text(bad), m.sigma.iter().map(|s| sym_text(s)).collect::<Vec<_>>().join(", ")) });
    }
    match run_view(it, name, &m, &w) {
        Some(r) => json!({ "run": r, "error": Json::Null }),
        None => json!({ "run": Json::Null, "error": "The run stopped with an error." }),
    }
}

/// Keeps `word_text` and `TraceVal` reachable for the trace panel's future use.
pub fn trace_text(t: &TraceVal) -> String {
    format!(
        "{} on {}: {}",
        t.machine,
        word_text(&t.input),
        t.verdict.name()
    )
}
