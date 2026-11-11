//! `tm/single-tape` (SEMANTICS §11.12): a deterministic k-tape machine as a one-tape machine.
//!
//! Each cell of the one tape holds either a plain symbol c of the old tape alphabet (read as c on
//! track 1 and ⊔ on the others, no heads) or a track symbol `[s₁,…,sₖ]`, with `^` after sᵢ where
//! tape i's head is and `▸` in front on cell 0. One step of the old machine takes two sweeps:
//! from cell 0 rightwards, collecting the k symbols under the heads; then leftwards, writing each
//! tape's symbol and moving its head mark; then back to cell 0. Only states and cell symbols that
//! can actually occur are built, by a worklist over (state, symbol) pairs.

use crate::label::{Dir, LOp, Label};
use crate::machine::{Machine, Parts, Provenance};
use crate::model::{InputMode, Model, StorageKind};
use crate::value::{Sym, Value, sym_text};
use delta_core::fa::Step;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::rc::Rc;

pub const ID: &str = "tm/single-tape";

/// Why the construction did not produce a machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The input already has one tape: use it as it is.
    OneTape,
    /// Not a deterministic on-tape machine with tapes (the message says why).
    Wrong(String),
    /// More states than the budget.
    Budget,
}

/// The contents of one cell, decoded.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Cell {
    syms: Vec<Sym>,
    /// Bit i set: tape i's head is on this cell.
    heads: u32,
    first: bool,
}

/// A state of the one-tape machine.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum St {
    Init,
    /// At old state q, sweeping right; `seen[i]` is tape i's head symbol once found.
    Collect(usize, Vec<Option<Sym>>),
    /// Sweeping left applying old edge e; bit i of `done` set once tape i is updated.
    Apply(usize, u32),
    /// Moving tape t's mark one cell (`right`: came by R, so go back L afterwards).
    Place(usize, u32, usize, bool),
    /// Walking back to cell 0, then collecting at old state q.
    Return(usize),
    Accept,
    Reject,
}

/// One old edge per tape: what it reads (if anything), what it writes (if anything), how it moves.
type TapeOps = (Vec<Option<Sym>>, Vec<Option<Sym>>, Vec<Dir>);

struct Build<'m> {
    m: &'m Machine,
    k: usize,
    blank: Sym,
    /// Each tape's Read/Write/Move per old edge: (reads, writes, moves), indexed by tape.
    ops: Vec<TapeOps>,
}

impl Build<'_> {
    fn decode(&self, s: &Sym) -> Cell {
        let mut syms = vec![self.blank.clone(); self.k];
        if let Some(first) = syms.first_mut() {
            *first = s.clone();
        }
        Cell {
            syms,
            heads: 0,
            first: false,
        }
    }

    fn encode(&self, c: &Cell) -> Sym {
        // A cell with no heads and only track 1 filled is just that plain symbol.
        if c.heads == 0
            && !c.first
            && c.syms.iter().skip(1).all(|s| *s == self.blank)
            && let Some(s) = c.syms.first()
        {
            return s.clone();
        }
        let parts: Vec<String> = c
            .syms
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let head = if c.heads & (1 << i) != 0 { "^" } else { "" };
                format!("{}{head}", sym_text(s))
            })
            .collect();
        let mark = if c.first { "▸" } else { "" };
        Sym::from(format!("{mark}[{}]", parts.join(",")).as_str())
    }

    /// The state to enter at old state q (halting states map to :accept / :reject).
    fn enter(&self, q: usize) -> St {
        if self.m.accept.contains(&q) {
            St::Accept
        } else if self.m.reject == Some(q) {
            St::Reject
        } else {
            St::Collect(q, vec![None; self.k])
        }
    }

    fn all(&self) -> u32 {
        (1u32 << self.k) - 1
    }

    /// The move from state `st` reading cell `c`: (cell to write, direction, next state).
    fn step(&self, st: &St, c: &Cell) -> Option<(Cell, Dir, St)> {
        match st {
            St::Init => {
                let mut c = c.clone();
                c.heads = self.all();
                c.first = true;
                Some((c, Dir::S, self.enter(self.m.start)))
            }
            St::Collect(q, seen) => {
                let mut seen = seen.clone();
                for (i, slot) in seen.iter_mut().enumerate() {
                    if c.heads & (1 << i) != 0 {
                        *slot = c.syms.get(i).cloned();
                    }
                }
                if seen.iter().any(Option::is_none) {
                    return Some((c.clone(), Dir::R, St::Collect(*q, seen)));
                }
                // The (only) old edge from q whose reads match the head symbols.
                let e = self.m.edges.iter().enumerate().position(|(ix, e)| {
                    e.from == *q
                        && self.ops.get(ix).is_some_and(|(reads, _, _)| {
                            reads.iter().zip(&seen).all(|(r, s)| r.is_none() || r == s)
                        })
                })?;
                Some((c.clone(), Dir::S, St::Apply(e, 0)))
            }
            // manual:begin single-tape-apply
            St::Apply(e, done) => {
                let (_, writes, moves) = self.ops.get(*e)?;
                let pending = (0..self.k).find(|i| c.heads & (1 << i) != 0 && done & (1 << i) == 0);
                match pending {
                    Some(t) => {
                        let mut out = c.clone();
                        if let Some(Some(x)) = writes.get(t)
                            && let Some(slot) = out.syms.get_mut(t)
                        {
                            *slot = x.clone();
                        }
                        let done = done | (1 << t);
                        match moves.get(t).copied().unwrap_or(Dir::S) {
                            Dir::R => {
                                out.heads &= !(1 << t);
                                Some((out, Dir::R, St::Place(*e, done, t, true)))
                            }
                            Dir::L if !c.first => {
                                out.heads &= !(1 << t);
                                Some((out, Dir::L, St::Place(*e, done, t, false)))
                            }
                            // S, or L on cell 0 (the head stays).
                            _ => Some((out, Dir::S, St::Apply(*e, done))),
                        }
                    }
                    None if *done == self.all() => {
                        let to = self.m.edges.get(*e)?.to;
                        if c.first {
                            Some((c.clone(), Dir::S, self.enter(to)))
                        } else {
                            Some((c.clone(), Dir::L, St::Return(to)))
                        }
                    }
                    None if c.first => None,
                    None => Some((c.clone(), Dir::L, St::Apply(*e, *done))),
                }
            }
            // manual:end single-tape-apply
            St::Place(e, done, t, right) => {
                let mut out = c.clone();
                out.heads |= 1 << t;
                let d = if *right { Dir::L } else { Dir::S };
                Some((out, d, St::Apply(*e, *done)))
            }
            St::Return(q) => {
                if c.first {
                    Some((c.clone(), Dir::S, self.enter(*q)))
                } else {
                    Some((c.clone(), Dir::L, St::Return(*q)))
                }
            }
            St::Accept | St::Reject => None,
        }
    }

    fn value(&self, st: &St) -> Value {
        let q = |i: usize| self.m.state(i);
        let word = |s: &str| Value::Word(Rc::new(vec![Sym::from(s)]));
        let set = |done: u32| {
            Value::set(
                (0..self.k)
                    .filter(|i| done & (1 << i) != 0)
                    .map(|i| Value::Int(i as i64 + 1))
                    .collect(),
            )
        };
        let from = |e: usize| {
            self.m
                .edges
                .get(e)
                .map(|x| q(x.from))
                .unwrap_or(Value::None)
        };
        match st {
            St::Init => Value::Atom(Rc::from("init")),
            St::Accept => Value::Atom(Rc::from("accept")),
            St::Reject => Value::Atom(Rc::from("reject")),
            St::Collect(p, seen) => {
                let s: Vec<Sym> = seen
                    .iter()
                    .map(|x| x.clone().unwrap_or_else(|| Sym::from("_")))
                    .collect();
                Value::tuple(vec![q(*p), word("collect"), Value::Word(Rc::new(s))])
            }
            St::Apply(e, done) => Value::tuple(vec![
                from(*e),
                word("apply"),
                Value::Int(*e as i64),
                set(*done),
            ]),
            St::Place(e, done, t, _) => Value::tuple(vec![
                from(*e),
                word("place"),
                Value::Int(*e as i64),
                set(*done),
                Value::Int(*t as i64 + 1),
            ]),
            St::Return(p) => Value::tuple(vec![q(*p), word("return")]),
        }
    }
}

/// Builds the one-tape machine. `tm` is the prelude's `tm` model; `max` bounds the states.
pub fn to_single_tape(m: &Machine, tm: Rc<Model>, max: usize) -> Result<Machine, Refusal> {
    let k = m
        .eff
        .storage
        .iter()
        .filter(|s| **s == StorageKind::Tape)
        .count();
    if m.eff.input != InputMode::OnTape || k == 0 || m.eff.storage.len() != k {
        return Err(Refusal::Wrong(format!(
            "to_single_tape needs a Turing machine with tapes, not a {}",
            m.model.display()
        )));
    }
    if k == 1 {
        return Err(Refusal::OneTape);
    }
    if !m.eff.deterministic {
        return Err(Refusal::Wrong(
            "to_single_tape needs a deterministic machine (one move per configuration)".into(),
        ));
    }
    if k > 16 {
        return Err(Refusal::Budget);
    }
    let ops = m
        .edges
        .iter()
        .map(|e| {
            let mut reads = vec![None; k];
            let mut writes = vec![None; k];
            let mut moves = vec![Dir::S; k];
            for op in &e.label.ops {
                match op {
                    LOp::Read(t, a) => {
                        if let Some(s) = reads.get_mut(*t) {
                            *s = Some(a.clone());
                        }
                    }
                    LOp::Write(t, x) => {
                        if let Some(s) = writes.get_mut(*t) {
                            *s = Some(x.clone());
                        }
                    }
                    LOp::Move(t, d) => {
                        if let Some(s) = moves.get_mut(*t) {
                            *s = *d;
                        }
                    }
                    _ => {}
                }
            }
            (reads, writes, moves)
        })
        .collect();
    let b = Build {
        m,
        k,
        blank: m.blank.clone(),
        ops,
    };

    // Worklist over (state, symbol): a new state meets every known symbol and vice versa.
    let mut plain: BTreeSet<Sym> = m.tape_alpha.clone();
    plain.extend(m.sigma.iter().cloned());
    plain.insert(m.blank.clone());
    let mut symbols: Vec<(Sym, Cell)> = plain.iter().map(|s| (s.clone(), b.decode(s))).collect();
    let mut sym_ix: HashMap<Sym, usize> = symbols
        .iter()
        .enumerate()
        .map(|(i, (s, _))| (s.clone(), i))
        .collect();
    let mut states: Vec<St> = vec![St::Init, St::Accept, St::Reject];
    let mut st_ix: HashMap<St, usize> = states
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let mut edges: Vec<(usize, Sym, Sym, Dir, usize)> = Vec::new();
    let mut queue: VecDeque<(usize, usize)> = (0..symbols.len()).map(|y| (0, y)).collect();
    while let Some((x, y)) = queue.pop_front() {
        let (Some(st), Some((read, cell))) = (states.get(x).cloned(), symbols.get(y).cloned())
        else {
            continue;
        };
        // :init reads the plain input symbol on cell 0 (nothing else is there yet).
        if matches!(st, St::Init) && !plain.contains(&read) {
            continue;
        }
        let Some((out, d, next)) = b.step(&st, &cell) else {
            continue;
        };
        let written = if out == cell {
            read.clone()
        } else {
            b.encode(&out)
        };
        if !sym_ix.contains_key(&written) {
            sym_ix.insert(written.clone(), symbols.len());
            symbols.push((written.clone(), out));
            let new = symbols.len() - 1;
            queue.extend((0..states.len()).map(|s| (s, new)));
        }
        let to = match st_ix.get(&next) {
            Some(i) => *i,
            None => {
                if states.len() >= max {
                    return Err(Refusal::Budget);
                }
                st_ix.insert(next.clone(), states.len());
                states.push(next);
                let new = states.len() - 1;
                queue.extend((0..symbols.len()).map(|s| (new, s)));
                new
            }
        };
        edges.push((x, read, written, d, to));
    }

    let values: Vec<Value> = states.iter().map(|s| b.value(s)).collect();
    let v = |i: usize| values.get(i).cloned().unwrap_or(Value::None);
    let edges: Vec<(Value, Label, Value)> = edges
        .into_iter()
        .map(|(p, read, write, d, q)| {
            let mut ops = vec![LOp::Read(0, read.clone())];
            if write != read {
                ops.push(LOp::Write(0, write));
            }
            ops.push(LOp::Move(0, d));
            (v(p), Label::new(ops), v(q))
        })
        .collect();
    let steps = vec![
        Step::new(
            "AddState",
            format!("{} states simulate the {k} tapes on one", states.len()),
        ),
        Step::new(
            "AddEdge",
            format!("{} cell symbols, {} edges", symbols.len(), edges.len()),
        ),
    ];
    let mut eff = (*tm).clone();
    eff.missing = m.eff.missing;
    Ok(Machine::assemble(Parts {
        model: tm,
        eff,
        name: None,
        states: values.iter().cloned().collect(),
        sigma: m.sigma.clone(),
        stack_alpha: BTreeSet::new(),
        tape_alpha: symbols.iter().map(|(s, _)| s.clone()).collect(),
        out_alpha: BTreeSet::new(),
        edges,
        start: v(0),
        accept: BTreeSet::from([v(1)]),
        reject: Some(v(2)),
        blank: m.blank.clone(),
        initial_stack: None,
        lambda: BTreeMap::new(),
        provenance: Some(Provenance {
            algorithm: ID.to_string(),
            steps,
        }),
    }))
}
