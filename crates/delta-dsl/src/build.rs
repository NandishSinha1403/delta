//! The checker: turns a machine block into a core `Dfa` or `Nfa`, reporting every violation of
//! Definition 1.5 / 1.37 it finds (docs/plan/08-ERRORS.md §4.0–4.2), all in one pass.
//!
//! Strict mode follows the book exactly; lenient mode (the playground default) fills in missing
//! pieces where the intent is clear, and says so.

use crate::ast::{DeltaEntry, DeltaForm, ElemKind, Field, Ident, Kind, MachineDecl, Mode, SetLit};
use crate::book;
use crate::diag::{Diagnostics, Span, TextEdit};
use delta_core::alphabet::{Alphabet, display_symbol};
use delta_core::bitset::BitSet;
use delta_core::dfa::Dfa;
use delta_core::ids::StateId;
use delta_core::nfa::{Label, Nfa};
use delta_core::object::Object;
use delta_core::state::{Origin, StateInfo};
// `HashMap<K, V>` maps keys to values (like a JS Map).
use std::collections::HashMap;

/// Names that would be confusing as state names (E110).
const RESERVED: [&str; 7] = [
    "start", "accept", "reject", "table", "partial", "eps", "empty",
];

/// The states and symbols a machine block mentions, even if it has errors (for completion).
#[derive(Debug, Clone, Default)]
// `#[derive(Default)]` gives `Mentions::default()`: every field set to its empty value.
pub struct Mentions {
    pub states: Vec<String>,
    pub symbols: Vec<String>,
}

/// Where fixes can insert text.
struct Anchors {
    q_close: Option<Span>,
    sigma_close: Option<Span>,
    /// The machine's closing `}`.
    close: Option<Span>,
}

fn insert_into_set(close: Option<Span>, item: &str, empty: bool) -> Option<TextEdit> {
    close.map(|c| TextEdit {
        span: Span::at(c.start),
        insert: if empty {
            format!("{item} ")
        } else {
            format!(", {item} ")
        },
    })
}

/// Collects the states and symbols a machine mentions, in order of appearance.
pub fn mentions(m: &MachineDecl) -> Mentions {
    let mut out = Mentions::default();
    let add_state = |s: &str, out: &mut Mentions| {
        if !out.states.iter().any(|x| x == s) {
            out.states.push(s.to_string());
        }
    };
    for (f, _) in &m.fields {
        match f {
            Field::Q(set) | Field::Accept(set) => {
                set.items.iter().for_each(|e| add_state(&e.text, &mut out))
            }
            Field::Start(i) => add_state(&i.text, &mut out),
            Field::Delta(entries, _) => {
                for e in entries {
                    add_state(&e.from.text, &mut out);
                    e.to.iter().for_each(|t| add_state(&t.text, &mut out));
                    if e.label.kind != ElemKind::Eps && !out.symbols.contains(&e.label.text) {
                        out.symbols.push(e.label.text.clone());
                    }
                }
            }
            Field::Sigma(set) => {
                for e in &set.items {
                    if !out.symbols.contains(&e.text) {
                        out.symbols.push(e.text.clone());
                    }
                }
            }
        }
    }
    out
}

/// Builds a machine, or returns None if it has errors (they are in `d`).
pub fn build_machine(m: &MachineDecl, mode: Mode, d: &mut Diagnostics) -> Option<Object> {
    let errors_before = d.error_count();
    let strict = mode == Mode::Strict;

    // ---- gather fields (E102: declared twice) ----
    // `Option<&SetLit>` holds either nothing or a borrowed reference to a SetLit inside `m`.
    let mut q_set: Option<&SetLit> = None;
    let mut sigma_set: Option<&SetLit> = None;
    let mut start: Option<&Ident> = None;
    let mut accept: Option<&SetLit> = None;
    let mut entries: Vec<&DeltaEntry> = Vec::new();
    let mut edges_close: Option<Span> = None;
    for (f, key) in &m.fields {
        let dup = |d: &mut Diagnostics, name: &str| {
            d.error(
                "E102",
                *key,
                format!("{name} is declared twice; only the first one is used."),
            );
        };
        match f {
            Field::Q(s) => {
                if q_set.is_some() {
                    dup(d, "Q");
                } else {
                    q_set = Some(s);
                }
            }
            Field::Sigma(s) => {
                if sigma_set.is_some() {
                    dup(d, "Σ");
                } else {
                    sigma_set = Some(s);
                }
            }
            Field::Start(i) => {
                if start.is_some() {
                    dup(d, "q0");
                } else {
                    start = Some(i);
                }
            }
            Field::Accept(s) => {
                if accept.is_some() {
                    dup(d, "F");
                } else {
                    accept = Some(s);
                }
            }
            Field::Delta(es, form) => {
                entries.extend(es.iter());
                if let DeltaForm::Edges(close) = form {
                    edges_close = Some(*close);
                }
            }
        }
    }
    let anchors = Anchors {
        q_close: q_set.and_then(|s| s.close),
        sigma_close: sigma_set.and_then(|s| s.close),
        close: (m.span.end > m.span.start)
            .then(|| Span::new(m.span.end as usize - 1, m.span.end as usize)),
    };
    let ment = mentions(m);

    // ---- Σ ----
    let mut sigma = Alphabet::new();
    let mut sigma_spans: HashMap<String, Span> = HashMap::new();
    if let Some(set) = sigma_set {
        if set.items.is_empty() {
            d.error(
                "E103",
                set.span,
                "Σ is empty, but an alphabet must be a nonempty finite set.",
            )
            .book(book::ALPHABET);
        }
        for e in &set.items {
            match e.kind {
                ElemKind::Eps => {
                    d.error(
                        "E108",
                        e.span,
                        "ε is the empty string, not a symbol: it cannot be a member of Σ.",
                    )
                    .book(book::EMPTY_STRING);
                }
                ElemKind::Empty => {
                    d.error("E108", e.span, "∅ is not a symbol.");
                }
                ElemKind::Word if e.text.chars().count() > 1 => {
                    multi_char(d, e.span, &e.text);
                }
                _ => {
                    if sigma.get(&e.text).is_some() {
                        d.warning(
                            "E109",
                            e.span,
                            format!("`{}` is listed twice in Σ.", e.text),
                        )
                        .book(book::SETS);
                    } else {
                        sigma.intern(&e.text);
                        sigma_spans.insert(e.text.clone(), e.span);
                    }
                }
            }
        }
    } else {
        for s in &ment.symbols {
            if s.chars().count() == 1
                || entries
                    .iter()
                    .any(|e| e.label.text == *s && e.label.kind == ElemKind::Angle)
            {
                sigma.intern(s);
            }
        }
        let names: Vec<String> = sigma.names().iter().map(|n| display_symbol(n)).collect();
        let listed = format!("{{{}}}", names.join(", "));
        if strict {
            d.error(
                "E101",
                m.name.span,
                format!("Σ is missing (strict mode). It would be {listed}."),
            )
            .fix("Insert Σ", insert_field(&anchors, &format!("Σ = {listed}")));
        } else if !sigma.is_empty() {
            d.info(
                "I115",
                m.name.span,
                format!("Σ was not written, so it is inferred from δ: Σ = {listed}."),
            )
            .fix(
                "Write Σ explicitly",
                insert_field(&anchors, &format!("Σ = {listed}")),
            );
        } else {
            d.error("E103", m.name.span, "Σ is empty: write Σ = { … }.")
                .book(book::ALPHABET);
        }
    }

    // ---- Q ----
    let mut names: Vec<String> = Vec::new();
    let mut state_spans: HashMap<String, Span> = HashMap::new();
    if let Some(set) = q_set {
        for e in &set.items {
            if e.kind != ElemKind::Word {
                d.error(
                    "E120",
                    e.span,
                    "A state name must be a plain name such as q1.",
                );
                continue;
            }
            if RESERVED.contains(&e.text.as_str()) {
                d.error(
                    "E110",
                    e.span,
                    format!("`{}` is a reserved word and cannot name a state.", e.text),
                );
                continue;
            }
            if names.contains(&e.text) {
                d.warning(
                    "E109",
                    e.span,
                    format!("State `{}` is listed twice in Q.", e.text),
                )
                .book(book::SETS);
                continue;
            }
            names.push(e.text.clone());
            state_spans.insert(e.text.clone(), e.span);
        }
        if names.is_empty() {
            d.error(
                "E101",
                set.span,
                "Q is empty: it must contain at least the start state.",
            )
            .book(book::DEF_1_5_START);
        }
    } else {
        names = ment
            .states
            .iter()
            .filter(|s| !RESERVED.contains(&s.as_str()))
            .cloned()
            .collect();
        let listed = format!("{{{}}}", names.join(", "));
        if strict {
            d.error(
                "E101",
                m.name.span,
                format!("Q is missing (strict mode). It would be {listed}."),
            )
            .fix("Insert Q", insert_field(&anchors, &format!("Q = {listed}")));
        } else {
            d.info(
                "I116",
                m.name.span,
                format!("Q was not written, so it is inferred: Q = {listed}."),
            )
            .fix(
                "Write Q explicitly",
                insert_field(&anchors, &format!("Q = {listed}")),
            );
        }
    }
    let index: HashMap<String, StateId> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), StateId::from_index(i)))
        .collect();

    // Unknown states (E104), reported once per name with a fix that adds them to Q.
    let mut reported: Vec<String> = Vec::new();
    // This closure captures `index`, `names` and `reported` from the surrounding function.
    let mut unknown = |d: &mut Diagnostics, id: &Ident, code: &'static str| {
        if index.contains_key(&id.text) || reported.contains(&id.text) {
            return;
        }
        reported.push(id.text.clone());
        let diag = d
            .error(
                code,
                id.span,
                format!(
                    "State `{}` is not in Q = {{{}}}.",
                    id.text,
                    names.join(", ")
                ),
            )
            .book(book::DEF_1_5_Q);
        if let Some(edit) = insert_into_set(anchors.q_close, &id.text, names.is_empty()) {
            diag.fix(format!("Add `{}` to Q", id.text), vec![edit]);
        }
    };

    // ---- q₀ ----
    let start_id = match start {
        Some(s) => {
            unknown(d, s, "E106");
            index.get(&s.text).copied()
        }
        None => {
            let first = names.first().cloned();
            if strict || first.is_none() {
                d.error("E101", m.name.span, "The start state q0 is missing.")
                    .book(book::DEF_1_5_START);
            } else if let Some(f) = &first {
                d.warning(
                    "W117",
                    m.name.span,
                    format!("q0 was not written, so the first state, {f}, is used."),
                )
                .fix(
                    format!("Write q0 = {f}"),
                    insert_field(&anchors, &format!("q0 = {f}")),
                );
            }
            first.and_then(|f| index.get(&f).copied())
        }
    };

    // ---- F ----
    let mut accept_set = BitSet::new();
    match accept {
        Some(set) => {
            for e in &set.items {
                let id = Ident {
                    text: e.text.clone(),
                    span: e.span,
                };
                unknown(d, &id, "E107");
                if let Some(q) = index.get(&e.text) {
                    accept_set.insert(q.index());
                }
            }
            if set.items.is_empty() {
                d.warning(
                    "W112",
                    set.span,
                    "F = ∅, so this machine accepts no strings (legal, p. 36).",
                )
                .book(book::EMPTY_F);
            }
        }
        None if strict => {
            d.error(
                "E101",
                m.name.span,
                "F, the set of accept states, is missing.",
            )
            .book(book::DEF_1_5_ACCEPT);
        }
        None => {
            d.warning(
                "W112",
                m.name.span,
                "F was not written, so F = ∅ and this machine accepts no strings.",
            )
            .book(book::EMPTY_F)
            .fix("Add F", insert_field(&anchors, "F = { }"));
        }
    }

    // ---- δ ----
    let change_kind = |to: &str| TextEdit {
        span: m.kind_span,
        insert: to.to_string(),
    };
    for e in &entries {
        unknown(d, &e.from, "E104");
        for t in &e.to {
            unknown(d, t, "E104");
        }
        match e.label.kind {
            ElemKind::Eps if m.kind == Kind::Dfa => {
                d.error(
                    "E203",
                    e.label.span,
                    "A DFA cannot have ε-transitions: its δ takes symbols of Σ, not Σ_ε.",
                )
                .book(book::DEF_1_5_DELTA)
                .fix("Change `dfa` to `nfa`", vec![change_kind("nfa")]);
            }
            ElemKind::Eps => {}
            ElemKind::Empty => {
                d.error(
                    "E120",
                    e.label.span,
                    "∅ is not a label: leave the arrow out instead.",
                );
            }
            ElemKind::Word if e.label.text.chars().count() > 1 => {
                multi_char(d, e.label.span, &e.label.text)
            }
            _ => {
                if sigma.get(&e.label.text).is_none() {
                    let code = if m.kind == Kind::Dfa { "E105" } else { "E301" };
                    let shown = display_symbol(&e.label.text);
                    let diag = d
                        .error(code, e.label.span, format!("Symbol `{shown}` is not in Σ."))
                        .book(if m.kind == Kind::Dfa {
                            book::DEF_1_5_DELTA
                        } else {
                            book::DEF_1_37_DELTA
                        });
                    if let Some(edit) =
                        insert_into_set(anchors.sigma_close, &shown, sigma.is_empty())
                    {
                        diag.fix(format!("Add `{shown}` to Σ"), vec![edit]);
                    }
                }
            }
        }
        if m.kind == Kind::Dfa && (e.set || e.to.len() != 1) {
            d.error(
                "E204",
                e.span,
                "A DFA's δ returns one state, not a set of states.",
            )
            .book(book::DEF_1_5_DELTA)
            .fix("Change `dfa` to `nfa`", vec![change_kind("nfa")]);
        }
    }

    if d.error_count() > errors_before {
        return None;
    }
    // `?` on an Option returns None from this function if the value is None.
    let start_id = start_id?;
    let states: Vec<StateInfo> = names.iter().map(StateInfo::user).collect();

    let object = match m.kind {
        Kind::Nfa => {
            let mut n = Nfa::empty(sigma.clone());
            for s in &states {
                n.add_state(s.clone());
            }
            let mut seen: Vec<(StateId, Label, StateId)> = Vec::new();
            for e in &entries {
                let from = index.get(&e.from.text).copied()?;
                let label = match e.label.kind {
                    ElemKind::Eps => Label::Eps,
                    _ => Label::Sym(sigma.get(&e.label.text)?),
                };
                for t in &e.to {
                    let to = index.get(&t.text).copied()?;
                    if seen.contains(&(from, label, to)) {
                        d.warning(
                            "W302",
                            e.span,
                            "This arrow is listed twice; δ returns a set, so it has no effect.",
                        );
                    } else {
                        seen.push((from, label, to));
                        n.add_edge(from, label, to);
                    }
                }
            }
            n.start = start_id;
            n.accept = accept_set;
            if n.is_deterministic_and_total() && !n.states.is_empty() {
                d.info(
                    "I304",
                    m.name.span,
                    format!(
                        "{} is deterministic and total, so it is also a DFA.",
                        m.name.text
                    ),
                )
                .book(book::DFA_IS_NFA)
                .fix("Change `nfa` to `dfa`", vec![change_kind("dfa")]);
            }
            unreachable_nfa(&n, &state_spans, d);
            Object::Nfa(n)
        }
        Kind::Dfa => {
            let k = sigma.len();
            let mut table: Vec<Option<(StateId, Span)>> = vec![None; names.len() * k];
            for e in &entries {
                let from = index.get(&e.from.text).copied()?;
                let a = sigma.get(&e.label.text)?;
                let to = index.get(&e.to.first()?.text).copied()?;
                let slot = table.get_mut(from.index() * k + a.index())?;
                match slot {
                    Some((prev, prev_span)) if *prev != to => {
                        let (prev, prev_span) = (*prev, *prev_span);
                        d.error(
                            "E202",
                            e.span,
                            format!(
                                "δ({}, {}) is defined twice ({} and {}). A DFA's δ returns exactly one state.",
                                e.from.text,
                                display_symbol(&e.label.text),
                                names.get(prev.index()).map_or("?", String::as_str),
                                t_name(&names, to),
                            ),
                        )
                        .book(book::DEF_1_5_DELTA)
                        .fix("Change `dfa` to `nfa`", vec![change_kind("nfa")]);
                        let _ = prev_span;
                    }
                    Some(_) => {
                        d.warning("W302", e.span, "This transition is listed twice.");
                    }
                    None => *slot = Some((to, e.span)),
                }
            }
            if d.error_count() > errors_before {
                return None;
            }
            // Totality (E201): every (q, a) needs exactly one transition.
            let mut missing: Vec<(StateId, Vec<usize>)> = Vec::new();
            for q in 0..names.len() {
                let syms: Vec<usize> = (0..k)
                    .filter(|&a| table.get(q * k + a).is_some_and(Option::is_none))
                    .collect();
                if !syms.is_empty() {
                    missing.push((StateId::from_index(q), syms));
                }
            }
            let mut states = states;
            let mut trap: Option<StateId> = None;
            if !missing.is_empty() {
                let trap_name = fresh(&names, "trap");
                let total: usize = missing.iter().map(|(_, s)| s.len()).sum();
                let first = missing.first().map(|(q, syms)| {
                    format!(
                        "δ({}, {})",
                        t_name(&names, *q),
                        syms.first().map_or(String::new(), |&a| sigma
                            .display(delta_core::ids::SymId::from_index(a)))
                    )
                });
                let list = missing
                    .iter()
                    .flat_map(|(q, syms)| {
                        syms.iter().map(|&a| {
                            format!(
                                "δ({}, {})",
                                t_name(&names, *q),
                                sigma.display(delta_core::ids::SymId::from_index(a))
                            )
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let msg = if total == 1 {
                    format!(
                        "{} is undefined. Definition 1.5 requires δ : Q × Σ → Q to be total.",
                        first.unwrap_or_default()
                    )
                } else {
                    format!(
                        "δ is not total: {list} are undefined. Definition 1.5 requires δ : Q × Σ → Q to be total."
                    )
                };
                let mut fixes: Vec<(String, Vec<TextEdit>)> = vec![(
                    "Mark as `partial` (missing transitions go to a trap state)".into(),
                    vec![TextEdit {
                        span: Span::at(m.name.span.end),
                        insert: " partial".into(),
                    }],
                )];
                if let Some(close) = edges_close {
                    let mut text = String::new();
                    for (q, syms) in &missing {
                        let labels: Vec<String> = syms
                            .iter()
                            .map(|&a| sigma.display(delta_core::ids::SymId::from_index(a)))
                            .collect();
                        text.push_str(&format!(
                            "  {} -{}-> {trap_name}\n  ",
                            t_name(&names, *q),
                            bracket(&labels)
                        ));
                    }
                    let all: Vec<String> = sigma.ids().map(|a| sigma.display(a)).collect();
                    text.push_str(&format!("{trap_name} -{}-> {trap_name}\n  ", bracket(&all)));
                    let mut edits = vec![TextEdit {
                        span: Span::at(close.start),
                        insert: text,
                    }];
                    if let Some(e) = insert_into_set(anchors.q_close, &trap_name, false) {
                        edits.push(e);
                    }
                    fixes.push((
                        format!("Send the missing transitions to a new state `{trap_name}`"),
                        edits,
                    ));
                }
                if !m.partial {
                    let diag = if strict {
                        d.error("E201", m.name.span, msg)
                    } else {
                        d.warning(
                            "W201",
                            m.name.span,
                            format!("{msg} An implicit trap state was added (drawn dashed)."),
                        )
                    };
                    diag.book(book::DEF_1_5_TOTAL);
                    for (title, edits) in fixes {
                        diag.fix(title, edits);
                    }
                    if strict {
                        return None;
                    }
                }
                let t = StateId::from_index(states.len());
                states.push(StateInfo {
                    name: trap_name,
                    origin: Origin::Trap,
                });
                trap = Some(t);
            }
            let n = states.len();
            let mut delta = Vec::with_capacity(n * k);
            for q in 0..n {
                for a in 0..k {
                    let to = table.get(q * k + a).copied().flatten().map(|(t, _)| t);
                    delta.push(to.or(trap).unwrap_or(StateId::from_index(q)));
                }
            }
            let dfa = Dfa::new(states, sigma, delta, start_id, accept_set).ok()?;
            unreachable_dfa(&dfa, &state_spans, d);
            Object::Dfa(dfa)
        }
    };
    Some(object)
}

fn t_name(names: &[String], q: StateId) -> &str {
    names.get(q.index()).map_or("?", String::as_str)
}

fn bracket(labels: &[String]) -> String {
    if labels.len() == 1 {
        labels.join("")
    } else {
        format!("[{}]", labels.join(","))
    }
}

fn fresh(names: &[String], base: &str) -> String {
    let mut n = base.to_string();
    while names.contains(&n) {
        n.push('\'');
    }
    n
}

fn multi_char(d: &mut Diagnostics, span: Span, text: &str) {
    d.error(
        "E114",
        span,
        format!("`{text}` has several characters. Symbols are single characters; write <{text}> for one multi-character symbol."),
    )
    .fix(format!("Write <{text}>"), vec![TextEdit { span, insert: format!("<{text}>") }]);
}

/// A fix that inserts a new field line just before the machine's closing `}`.
fn insert_field(a: &Anchors, line: &str) -> Vec<TextEdit> {
    match a.close {
        Some(c) => vec![TextEdit {
            span: Span::at(c.start),
            insert: format!("  {line}\n"),
        }],
        None => Vec::new(),
    }
}

fn unreachable_dfa(m: &Dfa, spans: &HashMap<String, Span>, d: &mut Diagnostics) {
    let mut seen = BitSet::of([m.start.index()]);
    let mut stack = vec![m.start];
    while let Some(q) = stack.pop() {
        for a in m.sigma.ids() {
            let r = m.next(q, a);
            if seen.insert(r.index()) {
                stack.push(r);
            }
        }
    }
    report_unreachable(&m.states, &seen, spans, d);
}

fn unreachable_nfa(m: &Nfa, spans: &HashMap<String, Span>, d: &mut Diagnostics) {
    let mut seen = BitSet::of([m.start.index()]);
    let mut stack = vec![m.start];
    while let Some(q) = stack.pop() {
        for &(_, r) in m.edges(q) {
            if seen.insert(r.index()) {
                stack.push(r);
            }
        }
    }
    report_unreachable(&m.states, &seen, spans, d);
}

fn report_unreachable(
    states: &[StateInfo],
    seen: &BitSet,
    spans: &HashMap<String, Span>,
    d: &mut Diagnostics,
) {
    for (i, s) in states.iter().enumerate() {
        if !seen.contains(i)
            && s.origin == Origin::User
            && let Some(span) = spans.get(&s.name)
        {
            d.warning(
                "W111",
                *span,
                format!("State `{}` is unreachable from the start state.", s.name),
            );
        }
    }
}
