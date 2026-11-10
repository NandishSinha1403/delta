//! Mealy ↔ Moore (SEMANTICS §11.7):
//! - `mealy/to-moore`: a state remembers the output of the edge that entered it, (q, o).
//! - `moore/to-mealy`: each edge emits the output of the state it enters.

use crate::label::{LOp, Label};
use crate::machine::{Machine, Parts, Provenance};
use crate::model::{Model, OutputMode};
use crate::value::{Sym, Value};
use delta_core::fa::Step;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

// `Option<T>` is either `Some(value)` or `None`; `?` on an Option returns None early.
/// An edge as (read symbol, output word); None if it does something else.
fn read_emit(l: &Label) -> Option<(Sym, Vec<Sym>)> {
    let mut read = None;
    let mut out = Vec::new();
    for op in &l.ops {
        match op {
            LOp::Read(0, a) => read = Some(a.clone()),
            LOp::Emit(w) => out.extend(w.iter().cloned()),
            _ => return None,
        }
    }
    Some((read?, out))
}

fn word(w: &[Sym]) -> Value {
    Value::Word(Rc::new(w.to_vec()))
}

// `Result<T, E>` is either `Ok(value)` or `Err(error)`: Rust's way of returning errors.
pub fn mealy_to_moore(m: &Machine, model: Rc<Model>) -> Result<Machine, String> {
    if m.eff.output != OutputMode::PerTransition || !m.eff.deterministic {
        return Err(
            "mealy_to_moore needs a deterministic per-transition transducer (mealy)".into(),
        );
    }
    let mut edges_in = Vec::new();
    for e in &m.edges {
        let (a, o) = read_emit(&e.label)
            .ok_or("mealy_to_moore: every edge must read one symbol and emit a word")?;
        edges_in.push((e.from, a, o, e.to));
    }
    let pair = |q: usize, o: &[Sym]| Value::tuple(vec![m.state(q), word(o)]);
    let mut states: BTreeSet<(usize, Vec<Sym>)> = BTreeSet::new();
    states.insert((m.start, Vec::new()));
    for (_, _, o, q) in &edges_in {
        states.insert((*q, o.clone()));
    }
    let mut edges = Vec::new();
    for (p, a, o, q) in &edges_in {
        for (p2, o2) in states.iter().filter(|(x, _)| x == p) {
            edges.push((
                pair(*p2, o2),
                Label::new(vec![LOp::Read(0, a.clone())]),
                pair(*q, o),
            ));
        }
    }
    let lambda: BTreeMap<Value, Vec<Sym>> = states
        .iter()
        .map(|(q, o)| (pair(*q, o), o.clone()))
        .collect();
    let all: BTreeSet<Value> = states.iter().map(|(q, o)| pair(*q, o)).collect();
    let steps = vec![Step::new(
        "AddState",
        format!(
            "{} states (q, o): q with the output of the edge that entered it",
            all.len()
        ),
    )];
    let eff = (*model).clone();
    Ok(Machine::assemble(Parts {
        model,
        eff,
        name: None,
        accept: all.clone(),
        states: all,
        sigma: m.sigma.clone(),
        stack_alpha: BTreeSet::new(),
        tape_alpha: BTreeSet::new(),
        out_alpha: m.out_alpha.clone(),
        edges,
        start: pair(m.start, &[]),
        reject: None,
        blank: m.blank.clone(),
        initial_stack: None,
        lambda,
        provenance: Some(Provenance {
            algorithm: "mealy/to-moore".to_string(),
            steps,
        }),
    }))
}

pub fn moore_to_mealy(m: &Machine, model: Rc<Model>) -> Result<Machine, String> {
    if m.eff.output != OutputMode::PerState {
        return Err("moore_to_mealy needs a per-state transducer (moore)".into());
    }
    let mut edges = Vec::new();
    for e in &m.edges {
        let (a, _) =
            read_emit(&e.label).ok_or("moore_to_mealy: every edge must read one symbol")?;
        let out = m.lambda.get(&e.to).cloned().unwrap_or_default();
        edges.push((
            m.state(e.from),
            Label::new(vec![LOp::Read(0, a), LOp::Emit(out)]),
            m.state(e.to),
        ));
    }
    let states: BTreeSet<Value> = m.states.iter().cloned().collect();
    let steps = vec![Step::new(
        "AddEdge",
        "each edge emits the output of the state it enters".to_string(),
    )];
    let eff = (*model).clone();
    Ok(Machine::assemble(Parts {
        model,
        eff,
        name: None,
        accept: states.clone(),
        states,
        sigma: m.sigma.clone(),
        stack_alpha: BTreeSet::new(),
        tape_alpha: BTreeSet::new(),
        out_alpha: m.out_alpha.clone(),
        edges,
        start: m.state(m.start),
        reject: None,
        blank: m.blank.clone(),
        initial_stack: None,
        lambda: BTreeMap::new(),
        provenance: Some(Provenance {
            algorithm: "moore/to-mealy".to_string(),
            steps,
        }),
    }))
}
