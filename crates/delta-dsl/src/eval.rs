//! The evaluator: runs a program's statements in order (docs/plan/04-DSL.md §7).
//!
//! Machines and regexes are checked and built, `let` applies the book's conversions, and `check` /
//! `run` simulate. Everything that goes wrong becomes a diagnostic; nothing panics.

use crate::ast::{Call, ElemKind, Item, Mode, RegexDecl, Rx, SetLit, StrLit, Value};
use crate::book;
use crate::build::{build_machine, mentions};
use crate::diag::{Diagnostics, Severity, Span, TextEdit, suggest};
use crate::document::*;
use crate::{lexer, parser};
use delta_core::alphabet::{Alphabet, display_symbol};
use delta_core::budget::{Budget, TlError};
use delta_core::convert::c01_dfa_to_nfa::dfa_to_nfa;
use delta_core::convert::c02_subset::{FULL_LIMIT, SubsetMode, subset_construction};
use delta_core::convert::c03_regex_to_nfa::regex_to_nfa;
use delta_core::convert::c04_to_gnfa::{dfa_to_gnfa, nfa_to_gnfa};
use delta_core::convert::c05_gnfa_to_regex::{RipOrder, gnfa_to_regex};
use delta_core::gnfa::Gnfa;
use delta_core::ids::{StateId, SymId};
use delta_core::nfa::Nfa;
use delta_core::object::{Object, RegexObj};
use delta_core::regex::{ReArena, ReId};
use delta_core::simulate::{RunView, s1_dfa, s2_nfa};
use delta_core::trace::Trace;
use delta_core::view::{MachineView, TableView, TraceView, machine_view, trace_view};

/// Deepest regex tree accepted after parsing (stars and parentheses together).
const MAX_RX_DEPTH: usize = 400;

/// A named object and where it came from.
struct Named {
    name: String,
    object: Object,
    decl: Span,
    derived: Option<String>,
    trace: Option<TraceView>,
}

// A struct with a derived Default starts with empty Vecs.
#[derive(Default)]
struct Env {
    named: Vec<Named>,
    /// Names whose declaration had errors (dependents are skipped, not reported again).
    errored: Vec<String>,
}

impl Env {
    fn get(&self, name: &str) -> Option<&Named> {
        self.named.iter().find(|n| n.name == name)
    }

    fn has(&self, name: &str) -> bool {
        self.get(name).is_some() || self.errored.iter().any(|e| e == name)
    }

    fn decl_of(&self, name: &str) -> Option<Span> {
        self.get(name).map(|n| n.decl)
    }
}

struct RawResult {
    kind: &'static str,
    object: String,
    span: Span,
    cases: Vec<(String, bool, bool, Span)>,
    run: Option<RunView>,
}

/// Lexes, parses, checks and evaluates a program.
pub fn evaluate(src: &str) -> Document {
    let (env, mut d, results, outline, mode) = eval_program(src);
    let map = Utf16Map::new(src);
    let objects = env
        .named
        .iter()
        .map(|n| ObjectView {
            name: n.name.clone(),
            kind: n.object.kind().to_string(),
            from: map.get(n.decl.start),
            to: map.get(n.decl.end),
            derived: n.derived.clone(),
            machine: machine_view(&n.object),
            regex: match &n.object {
                Object::Regex(r) => Some(r.text()),
                _ => None,
            },
            sigma: n
                .object
                .sigma()
                .names()
                .iter()
                .map(|s| display_symbol(s))
                .collect(),
            trace: n.trace.clone(),
        })
        .collect();
    // Errors first, then by position, so the problems panel reads top-down.
    d.list.sort_by_key(|x| (x.span.start, x.span.end));
    let diagnostics = d
        .list
        .iter()
        .map(|x| DiagnosticView {
            code: x.code.to_string(),
            severity: match x.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
                Severity::Hint => "hint",
            }
            .to_string(),
            from: map.get(x.span.start),
            to: map.get(x.span.end),
            message: x.message.clone(),
            book: x.book.map(|b| BookRefView {
                item: b.item.to_string(),
                quote: b.quote.to_string(),
            }),
            fixes: x
                .fixes
                .iter()
                .map(|f| FixView {
                    title: f.title.clone(),
                    edits: f
                        .edits
                        .iter()
                        .map(|e| EditView {
                            from: map.get(e.span.start),
                            to: map.get(e.span.end),
                            insert: e.insert.clone(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let results = results
        .into_iter()
        .map(|r| CommandResultView {
            kind: r.kind.to_string(),
            object: r.object,
            from: map.get(r.span.start),
            to: map.get(r.span.end),
            cases: r
                .cases
                .into_iter()
                .map(|(input, expected, actual, s)| CheckCaseView {
                    input,
                    expected,
                    actual,
                    from: map.get(s.start),
                    to: map.get(s.end),
                })
                .collect(),
            run: r.run,
        })
        .collect();
    let outline = outline
        .into_iter()
        .map(|(name, kind, span, states, symbols)| OutlineView {
            name,
            kind,
            from: map.get(span.start),
            to: map.get(span.end),
            states,
            symbols,
        })
        .collect();
    Document {
        objects,
        diagnostics,
        results,
        outline,
        mode: if mode == Mode::Strict {
            "strict"
        } else {
            "lenient"
        }
        .to_string(),
    }
}

/// Runs machine `name` of the program on `input` (the Run panel's input box).
pub fn run_input(src: &str, name: &str, input: &str) -> RunResult {
    let (env, ..) = eval_program(src);
    let Some(n) = env.get(name) else {
        return RunResult {
            run: None,
            error: Some(format!("`{name}` has errors or does not exist.")),
        };
    };
    let sigma = n.object.sigma();
    let symbols = match parse_input(input, sigma) {
        Ok(s) => s,
        Err((_, msg)) => {
            return RunResult {
                run: None,
                error: Some(msg),
            };
        }
    };
    match &n.object {
        Object::Dfa(m) => RunResult {
            run: Some(s1_dfa::run_view(name, m, &symbols)),
            error: None,
        },
        Object::Nfa(m) => RunResult {
            run: Some(s2_nfa::run_view(name, m, &symbols)),
            error: None,
        },
        _ => RunResult {
            run: None,
            error: Some(format!(
                "`{name}` is a {}; runs need a DFA or an NFA. Try `let N = nfa({name})`.",
                n.object.kind()
            )),
        },
    }
}

// `type X = …` gives a long type a short name.
type Outline = Vec<(String, String, Span, Vec<String>, Vec<String>)>;

fn eval_program(src: &str) -> (Env, Diagnostics, Vec<RawResult>, Outline, Mode) {
    let mut d = Diagnostics::default();
    let toks = lexer::lex(src, &mut d);
    let program = parser::parse(toks, &mut d);
    let budget = Budget::default();
    let mut env = Env::default();
    let mut results = Vec::new();
    let mut outline: Outline = Vec::new();

    for item in &program.items {
        match item {
            Item::Machine(m) => {
                let men = mentions(m);
                let kind = if m.kind == crate::ast::Kind::Dfa {
                    "dfa"
                } else {
                    "nfa"
                };
                outline.push((
                    m.name.text.clone(),
                    kind.into(),
                    m.span,
                    men.states,
                    men.symbols,
                ));
                if duplicate(&env, &m.name.text, m.name.span, src, &mut d) {
                    continue;
                }
                match build_machine(m, program.mode, &mut d) {
                    Some(object) => env.named.push(Named {
                        name: m.name.text.clone(),
                        object,
                        decl: m.span,
                        derived: None,
                        trace: None,
                    }),
                    None => env.errored.push(m.name.text.clone()),
                }
            }
            Item::Regex(r) => {
                let symbols = r
                    .over
                    .as_ref()
                    .map(|s| s.items.iter().map(|e| e.text.clone()).collect())
                    .unwrap_or_else(|| rx_symbols(&r.body));
                outline.push((
                    r.name.text.clone(),
                    "regex".into(),
                    r.span,
                    Vec::new(),
                    symbols,
                ));
                if duplicate(&env, &r.name.text, r.name.span, src, &mut d) {
                    continue;
                }
                match build_regex(r, &mut d) {
                    Some(object) => env.named.push(Named {
                        name: r.name.text.clone(),
                        object,
                        decl: r.span,
                        derived: None,
                        trace: None,
                    }),
                    None => env.errored.push(r.name.text.clone()),
                }
            }
            Item::Let { name, call, span } => {
                outline.push((
                    name.text.clone(),
                    call.func.text.clone(),
                    *span,
                    Vec::new(),
                    Vec::new(),
                ));
                if duplicate(&env, &name.text, name.span, src, &mut d) {
                    continue;
                }
                match eval_call(call, &name.text, &env, &mut d, &budget) {
                    Some((object, derived, trace)) => {
                        if let Some(o) = outline.last_mut() {
                            o.1 = object.kind().to_string();
                            o.3 = state_names(&object);
                            o.4 = object.sigma().names().to_vec();
                        }
                        env.named.push(Named {
                            name: name.text.clone(),
                            object,
                            decl: *span,
                            derived: Some(derived),
                            trace,
                        })
                    }
                    None => env.errored.push(name.text.clone()),
                }
            }
            Item::Check {
                target,
                accepts,
                inputs,
                span,
            } => {
                let Some(n) = lookup(&env, target, None, &mut d) else {
                    continue;
                };
                let mut cases = Vec::new();
                // Convert once per command for regexes and GNFAs (D3: C3 then D2).
                let as_nfa = match &n.object {
                    Object::Regex(r) => regex_to_nfa(r, &budget).ok().map(|(n, _)| n),
                    Object::Gnfa(g) => gnfa_nfa(g, &budget),
                    _ => None,
                };
                for s in inputs {
                    let Some(symbols) = input_or_report(s, n.object.sigma(), &mut d) else {
                        continue;
                    };
                    let actual = match (&n.object, &as_nfa) {
                        (Object::Dfa(m), _) => s1_dfa::accepts(m, &symbols),
                        (Object::Nfa(m), _) => s2_nfa::accepts(m, &symbols),
                        (_, Some(nfa)) => s2_nfa::accepts(nfa, &symbols),
                        _ => continue,
                    };
                    if actual != *accepts {
                        let shown = if s.text.is_empty() {
                            "ε (the empty string)".to_string()
                        } else {
                            format!("\"{}\"", s.text)
                        };
                        d.warning(
                            "R910",
                            s.span,
                            format!(
                                "Check failed: {} {} {shown}.",
                                target.text,
                                if actual { "accepts" } else { "rejects" }
                            ),
                        );
                    }
                    cases.push((s.text.clone(), *accepts, actual, s.span));
                }
                results.push(RawResult {
                    kind: "check",
                    object: target.text.clone(),
                    span: *span,
                    cases,
                    run: None,
                });
            }
            Item::Run {
                target,
                input,
                span,
            } => {
                let Some(n) = lookup(&env, target, None, &mut d) else {
                    continue;
                };
                let Some(symbols) = input_or_report(input, n.object.sigma(), &mut d) else {
                    continue;
                };
                let run = match &n.object {
                    Object::Dfa(m) => s1_dfa::run_view(&target.text, m, &symbols),
                    Object::Nfa(m) => s2_nfa::run_view(&target.text, m, &symbols),
                    other => {
                        d.error(
                            "E801",
                            target.span,
                            format!(
                                "`run` needs a DFA or an NFA; `{}` is a {}. Write `let N = nfa({})` and run N.",
                                target.text,
                                other.kind(),
                                target.text
                            ),
                        );
                        continue;
                    }
                };
                results.push(RawResult {
                    kind: "run",
                    object: target.text.clone(),
                    span: *span,
                    cases: Vec::new(),
                    run: Some(run),
                });
            }
        }
    }
    (env, d, results, outline, program.mode)
}

fn state_names(o: &Object) -> Vec<String> {
    match o {
        Object::Dfa(m) => m.states.iter().map(|s| s.name.clone()).collect(),
        Object::Nfa(m) => m.states.iter().map(|s| s.name.clone()).collect(),
        Object::Gnfa(g) => g.states.iter().map(|s| s.name.clone()).collect(),
        Object::Regex(_) => Vec::new(),
    }
}

fn gnfa_nfa(g: &Gnfa, budget: &Budget) -> Option<Nfa> {
    let (r, _) = gnfa_to_regex(g, &RipOrder::Index, false, budget).ok()?;
    regex_to_nfa(&r, budget).ok().map(|(n, _)| n)
}

/// E803: a name may be defined only once.
fn duplicate(env: &Env, name: &str, span: Span, src: &str, d: &mut Diagnostics) -> bool {
    if !env.has(name) {
        return false;
    }
    let line = env.decl_of(name).map(|s| {
        src.get(..s.start as usize)
            .unwrap_or("")
            .matches('\n')
            .count()
            + 1
    });
    d.error(
        "E803",
        span,
        match line {
            Some(l) => format!("`{name}` is already defined on line {l}."),
            None => format!("`{name}` is already defined."),
        },
    );
    true
}

/// Finds a named object, reporting E802 (with a suggestion), E809 or a dependency note.
// `<'a>` ties the returned reference's lifetime to `env`: the result borrows from the Env.
fn lookup<'a>(
    env: &'a Env,
    id: &crate::ast::Ident,
    self_name: Option<&str>,
    d: &mut Diagnostics,
) -> Option<&'a Named> {
    if let Some(n) = env.get(&id.text) {
        return Some(n);
    }
    if self_name == Some(id.text.as_str()) {
        d.error("E809", id.span, format!("`{}` refers to itself.", id.text));
    } else if env.errored.contains(&id.text) {
        d.info(
            "I811",
            id.span,
            format!("Skipped: depends on `{}`, which has errors.", id.text),
        );
    } else {
        let names = env.named.iter().map(|n| n.name.as_str());
        let diag = d.error("E802", id.span, format!("No object named `{}`.", id.text));
        if let Some(s) = suggest(&id.text, names) {
            diag.message = format!("No object named `{}`. Did you mean `{s}`?", id.text);
            diag.fix(
                format!("Change to `{s}`"),
                vec![TextEdit {
                    span: id.span,
                    insert: s.to_string(),
                }],
            );
        }
    }
    None
}

const CONVERSIONS: [&str; 4] = ["dfa", "nfa", "gnfa", "regex"];

fn eval_call(
    call: &Call,
    self_name: &str,
    env: &Env,
    d: &mut Diagnostics,
    budget: &Budget,
) -> Option<(Object, String, Option<TraceView>)> {
    let f = call.func.text.as_str();
    let later = match f {
        "minimize" | "union" | "intersect" | "complement" | "reverse" | "star" | "concat"
        | "plus" | "equivalent" => Some("Phase 2"),
        "cnf" | "pda" | "cfg" => Some("Phase 3"),
        _ => None,
    };
    if let Some(phase) = later {
        d.info(
            "E123",
            call.func.span,
            format!("`{f}()` arrives in {phase}."),
        );
        return None;
    }
    if !CONVERSIONS.contains(&f) {
        let diag = d.error(
            "E802",
            call.func.span,
            format!("Unknown function `{f}`. Phase 1 has dfa(), nfa(), gnfa() and regex()."),
        );
        if let Some(s) = suggest(f, CONVERSIONS) {
            diag.fix(
                format!("Change to `{s}`"),
                vec![TextEdit {
                    span: call.func.span,
                    insert: s.to_string(),
                }],
            );
        }
        return None;
    }
    let valid: &[&str] = match f {
        "dfa" => &["full"],
        "regex" => &["order", "mode"],
        _ => &[],
    };
    for (key, _) in &call.options {
        if !valid.contains(&key.text.as_str()) {
            let list = if valid.is_empty() {
                "none".to_string()
            } else {
                valid
                    .iter()
                    .map(|v| format!("`{v}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            d.error(
                "E804",
                key.span,
                format!(
                    "Unknown option `{}` for {f}(). Valid options: {list}.",
                    key.text
                ),
            );
        }
    }
    if call.args.len() != 1 {
        d.error(
            "E120",
            call.span,
            format!("{f}() takes exactly one machine or expression."),
        );
        return None;
    }
    let arg = call.args.first()?;
    let (input, input_name): (Object, String) = match arg {
        Value::Name(id) => {
            let n = lookup(env, id, Some(self_name), d)?;
            (n.object.clone(), id.text.clone())
        }
        Value::Call(inner) => {
            let (o, _, _) = eval_call(inner, self_name, env, d, budget)?;
            (o, format!("{}(…)", inner.func.text))
        }
        other => {
            d.error(
                "E801",
                other.span(),
                format!("{f}() needs a machine or a regular expression."),
            );
            return None;
        }
    };
    // `.find(pred)` returns the first matching item as an Option.
    let option = |name: &str| {
        call.options
            .iter()
            .find(|(k, _)| k.text == name)
            .map(|(_, v)| v)
    };
    let wrong_type = |d: &mut Diagnostics, needs: &str| {
        d.error(
            "E801",
            arg.span(),
            format!("{f}() needs {needs}; `{input_name}` is a {}.", input.kind()),
        );
    };
    let fail = |d: &mut Diagnostics, e: TlError, code: &'static str| match e {
        TlError::Budget { limit, which } => {
            d.error(
                "R902",
                call.span,
                format!(
                    "Stopped: the result needs more than {limit} ({which}). Try a smaller machine."
                ),
            );
        }
        TlError::Invalid(msg) => {
            d.error(code, call.span, msg);
        }
        TlError::Internal(msg) => {
            d.error(
                "X999",
                call.span,
                format!("Internal error (a bug in Delta, not your mistake): {msg}"),
            );
        }
    };

    match f {
        "dfa" => {
            let full = match option("full") {
                Some(Value::Bool(b, _)) => *b,
                Some(v) => {
                    d.error("E804", v.span(), "`full` must be true or false.");
                    return None;
                }
                None => false,
            };
            let mode = if full {
                SubsetMode::Full
            } else {
                SubsetMode::Reachable
            };
            let (nfa, via) = match &input {
                Object::Nfa(n) => (n.clone(), ""),
                Object::Regex(r) => match regex_to_nfa(r, budget) {
                    Ok((n, _)) => (n, "Lemma 1.55, then "),
                    Err(e) => {
                        fail(d, e, "E801");
                        return None;
                    }
                },
                _ => {
                    wrong_type(d, "an NFA or a regular expression");
                    return None;
                }
            };
            if full && nfa.num_states() > FULL_LIMIT {
                d.error(
                    "E806",
                    call.span,
                    format!(
                        "The full power set would have 2^{} states. The limit is 2^{FULL_LIMIT}. Use the default (reachable subsets).",
                        nfa.num_states()
                    ),
                );
                return None;
            }
            match subset_construction(&nfa, mode, budget) {
                Ok((m, trace)) => {
                    let obj = Object::Dfa(m);
                    let title = "Theorem 1.39 · subset construction";
                    let tv = trace_for(title, &trace, &obj);
                    let how = if full {
                        "all 2^k subsets"
                    } else {
                        "reachable subsets"
                    };
                    Some((
                        obj,
                        format!("dfa({input_name}) · {via}Theorem 1.39, {how}"),
                        Some(tv),
                    ))
                }
                Err(e) => {
                    fail(d, e, "E806");
                    None
                }
            }
        }
        "nfa" => match &input {
            Object::Dfa(m) => {
                let (n, trace) = dfa_to_nfa(m, budget);
                let obj = Object::Nfa(n);
                let tv = trace_for("p. 47 · every DFA is an NFA", &trace, &obj);
                Some((
                    obj,
                    format!("nfa({input_name}) · every DFA is an NFA (p. 47)"),
                    Some(tv),
                ))
            }
            Object::Regex(r) => match regex_to_nfa(r, budget) {
                Ok((n, trace)) => {
                    let obj = Object::Nfa(n);
                    let tv = trace_for("Lemma 1.55 · regular expression → NFA", &trace, &obj);
                    Some((obj, format!("nfa({input_name}) · Lemma 1.55"), Some(tv)))
                }
                Err(e) => {
                    fail(d, e, "E801");
                    None
                }
            },
            _ => {
                wrong_type(d, "a DFA or a regular expression");
                None
            }
        },
        "gnfa" => {
            let (g, trace) = match &input {
                Object::Dfa(m) => dfa_to_gnfa(m, budget),
                Object::Nfa(n) => nfa_to_gnfa(n, budget),
                _ => {
                    wrong_type(d, "a DFA or an NFA");
                    return None;
                }
            };
            let obj = Object::Gnfa(g);
            let tv = trace_for("Lemma 1.60 · GNFA in special form", &trace, &obj);
            Some((
                obj,
                format!("gnfa({input_name}) · Lemma 1.60, special form"),
                Some(tv),
            ))
        }
        _ => {
            // regex(…)
            let g = match &input {
                Object::Gnfa(g) => g.clone(),
                Object::Dfa(m) => dfa_to_gnfa(m, budget).0,
                Object::Nfa(n) => nfa_to_gnfa(n, budget).0,
                Object::Regex(_) => {
                    wrong_type(d, "a DFA, NFA or GNFA");
                    return None;
                }
            };
            let faithful = match option("mode") {
                Some(Value::Name(m)) if m.text == "faithful" => true,
                Some(Value::Name(m)) if m.text == "simplified" => false,
                Some(v) => {
                    d.error(
                        "E804",
                        v.span(),
                        "`mode` must be `faithful` or `simplified`.",
                    );
                    return None;
                }
                None => true,
            };
            let order = match option("order") {
                Some(Value::List(list, _)) => {
                    let mut ids = Vec::new();
                    for id in list {
                        match g
                            .states
                            .iter()
                            .position(|s| s.name == id.text)
                            .map(StateId::from_index)
                        {
                            Some(q) if q == g.start || q == g.accept => {
                                d.error(
                                    "E805",
                                    id.span,
                                    "The rip order may only contain states other than q_start and q_accept (Lemma 1.60).",
                                );
                                return None;
                            }
                            Some(q) => ids.push(q),
                            None => {
                                d.error(
                                    "E805",
                                    id.span,
                                    format!("The GNFA has no state named `{}`.", id.text),
                                );
                                return None;
                            }
                        }
                    }
                    RipOrder::Given(ids)
                }
                Some(Value::Name(n)) if n.text == "heuristic" => RipOrder::Heuristic,
                Some(Value::Name(n)) if n.text == "index" => RipOrder::Index,
                Some(v) => {
                    d.error(
                        "E804",
                        v.span(),
                        "`order` must be a list of states like [q2, q1], `index` or `heuristic`.",
                    );
                    return None;
                }
                None => RipOrder::Index,
            };
            match gnfa_to_regex(&g, &order, faithful, budget) {
                Ok((r, trace)) => {
                    let tv = trace_view(
                        "Lemma 1.60 · CONVERT(G), state elimination",
                        &trace,
                        &empty_view(),
                    );
                    let mode_text = if faithful {
                        "faithful"
                    } else {
                        "simplified with the identities of p. 66"
                    };
                    Some((
                        Object::Regex(r),
                        format!("regex({input_name}) · Lemma 1.60, {mode_text}"),
                        Some(tv),
                    ))
                }
                Err(e) => {
                    fail(d, e, "E805");
                    None
                }
            }
        }
    }
}

fn empty_view() -> MachineView {
    MachineView {
        kind: "regex".into(),
        states: Vec::new(),
        sigma: Vec::new(),
        edges: Vec::new(),
        table: TableView {
            columns: Vec::new(),
            rows: Vec::new(),
        },
    }
}

fn trace_for(title: &str, trace: &Trace, obj: &Object) -> TraceView {
    let view = machine_view(obj).unwrap_or_else(empty_view);
    trace_view(title, trace, &view)
}

// ---------- strings ----------

/// Splits an input string into symbols of Σ: one character each, `<NAME>` for multi-char.
/// On failure returns the byte offset (inside `text`) and length of the bad symbol, and a message.
fn parse_input(text: &str, sigma: &Alphabet) -> Result<Vec<SymId>, ((usize, usize), String)> {
    let mut out = Vec::new();
    let mut iter = text.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        let (name, len) = if c == '<' {
            let rest = text.get(i + 1..).unwrap_or("");
            match rest.find('>') {
                Some(j) => {
                    let name = rest.get(..j).unwrap_or("").to_string();
                    for _ in 0..name.chars().count() + 1 {
                        iter.next();
                    }
                    (name, j + 2)
                }
                None => (c.to_string(), 1),
            }
        } else {
            (c.to_string(), c.len_utf8())
        };
        match sigma.get(&name) {
            Some(a) => out.push(a),
            None => {
                let listed: Vec<String> = sigma.names().iter().map(|n| display_symbol(n)).collect();
                return Err((
                    (i, len),
                    format!(
                        "Symbol `{}` is not in Σ = {{{}}}. Strings are made of symbols from Σ.",
                        display_symbol(&name),
                        listed.join(", ")
                    ),
                ));
            }
        }
    }
    Ok(out)
}

fn input_or_report(s: &StrLit, sigma: &Alphabet, d: &mut Diagnostics) -> Option<Vec<SymId>> {
    match parse_input(&s.text, sigma) {
        Ok(v) => Some(v),
        Err(((off, len), msg)) => {
            // +1 skips the opening quote.
            let start = s.span.start as usize + 1 + off;
            d.error("R901", Span::new(start, start + len), msg)
                .book(book::STRINGS);
            None
        }
    }
}

// ---------- regular expressions ----------

fn rx_symbols(rx: &Rx) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut stack = vec![rx];
    while let Some(r) = stack.pop() {
        match r {
            Rx::Sym(s, _) => {
                if !out.contains(s) {
                    out.push(s.clone());
                }
            }
            Rx::Union(v) | Rx::Concat(v) => stack.extend(v.iter().rev()),
            Rx::Star(x) | Rx::Plus(x) | Rx::Pow(x, _) => stack.push(x),
            Rx::Eps(_) | Rx::Empty(_) | Rx::Sigma(_) => {}
        }
    }
    out
}

fn rx_depth(rx: &Rx) -> usize {
    let mut max = 0;
    let mut stack = vec![(rx, 1usize)];
    while let Some((r, depth)) = stack.pop() {
        max = max.max(depth);
        match r {
            Rx::Union(v) | Rx::Concat(v) => stack.extend(v.iter().map(|c| (c, depth + 1))),
            Rx::Star(x) | Rx::Plus(x) | Rx::Pow(x, _) => stack.push((x, depth + 1)),
            _ => {}
        }
    }
    max
}

fn first_span(rx: &Rx) -> Span {
    match rx {
        Rx::Sym(_, s) | Rx::Eps(s) | Rx::Empty(s) | Rx::Sigma(s) => *s,
        Rx::Union(v) | Rx::Concat(v) => v.first().map_or(Span::default(), first_span),
        Rx::Star(x) | Rx::Plus(x) | Rx::Pow(x, _) => first_span(x),
    }
}

fn alphabet_from_set(set: &SetLit, d: &mut Diagnostics) -> Alphabet {
    let mut a = Alphabet::new();
    if set.items.is_empty() {
        d.error(
            "E103",
            set.span,
            "The alphabet is empty, but an alphabet must be a nonempty finite set.",
        )
        .book(book::ALPHABET);
    }
    for e in &set.items {
        match e.kind {
            ElemKind::Eps | ElemKind::Empty => {
                d.error("E108", e.span, "ε and ∅ are not symbols.")
                    .book(book::EMPTY_STRING);
            }
            ElemKind::Word if e.text.chars().count() > 1 => {
                d.error(
                    "E114",
                    e.span,
                    format!("`{}` has several characters; write <{}>.", e.text, e.text),
                );
            }
            _ => {
                a.intern(&e.text);
            }
        }
    }
    a
}

fn build_regex(r: &RegexDecl, d: &mut Diagnostics) -> Option<Object> {
    let before = d.error_count();
    if rx_depth(&r.body) > MAX_RX_DEPTH {
        d.error(
            "E408",
            first_span(&r.body),
            format!("Expression nested more than {MAX_RX_DEPTH} levels deep."),
        );
        return None;
    }
    let sigma = match &r.over {
        Some(set) => alphabet_from_set(set, d),
        None => {
            let mut a = Alphabet::new();
            for s in rx_symbols(&r.body) {
                a.intern(&s);
            }
            a
        }
    };
    let mut arena = ReArena::new();
    let close = r.over.as_ref().and_then(|s| s.close);
    let root = lower(&r.body, &mut arena, &sigma, close, d);
    if d.error_count() > before {
        return None;
    }
    Some(Object::Regex(RegexObj {
        arena,
        root: root?,
        sigma,
    }))
}

/// Shorthand-free core expression, built faithfully (no simplification), so Lemma 1.55 sees
/// exactly what the user wrote. R⁺ = RR*, Rᵏ = R…R, Σ = (a₁ ∪ … ∪ aₙ) (p. 65).
fn lower(
    rx: &Rx,
    arena: &mut ReArena,
    sigma: &Alphabet,
    over_close: Option<Span>,
    d: &mut Diagnostics,
) -> Option<ReId> {
    Some(match rx {
        Rx::Sym(s, span) => match sigma.get(s) {
            Some(a) => arena.sym(a),
            None => {
                let shown = display_symbol(s);
                let diag = d
                    .error(
                        "E403",
                        *span,
                        format!("Symbol `{shown}` is not in the declared alphabet."),
                    )
                    .book(book::DEF_1_52);
                if let Some(c) = over_close {
                    diag.fix(
                        format!("Add `{shown}` to the alphabet"),
                        vec![TextEdit {
                            span: Span::at(c.start),
                            insert: format!(", {shown} "),
                        }],
                    );
                }
                return None;
            }
        },
        Rx::Eps(_) => arena.eps(),
        Rx::Empty(_) => arena.empty(),
        Rx::Sigma(span) => {
            let ids: Vec<SymId> = sigma.ids().collect();
            let Some((&first, rest)) = ids.split_first() else {
                d.error("E103", *span, "Σ needs an alphabet: write `over { … }`.")
                    .book(book::ALPHABET);
                return None;
            };
            let mut acc = arena.sym(first);
            for &a in rest {
                let x = arena.sym(a);
                acc = arena.union_raw(acc, x);
            }
            acc
        }
        Rx::Union(v) => {
            let mut acc: Option<ReId> = None;
            for x in v {
                let y = lower(x, arena, sigma, over_close, d)?;
                acc = Some(match acc {
                    Some(a) => arena.union_raw(a, y),
                    None => y,
                });
            }
            acc?
        }
        Rx::Concat(v) => {
            if let Some(Rx::Empty(s)) = v.iter().find(|x| matches!(x, Rx::Empty(_))) {
                d.warning(
                    "W406",
                    *s,
                    "Concatenating with ∅ makes the whole term ∅ (R∅ = ∅, Example 1.53).",
                );
            }
            let mut acc: Option<ReId> = None;
            for x in v {
                let y = lower(x, arena, sigma, over_close, d)?;
                acc = Some(match acc {
                    Some(a) => arena.concat_raw(a, y),
                    None => y,
                });
            }
            acc?
        }
        Rx::Star(x) => {
            let y = lower(x, arena, sigma, over_close, d)?;
            arena.star_raw(y)
        }
        Rx::Plus(x) => {
            let y = lower(x, arena, sigma, over_close, d)?;
            let s = arena.star_raw(y);
            arena.concat_raw(y, s)
        }
        Rx::Pow(x, k) => {
            let y = lower(x, arena, sigma, over_close, d)?;
            if *k == 0 {
                arena.eps()
            } else {
                let mut acc = y;
                for _ in 1..*k {
                    acc = arena.concat_raw(acc, y);
                }
                acc
            }
        }
    })
}
