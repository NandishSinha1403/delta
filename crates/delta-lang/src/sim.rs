//! The generic simulator (SEMANTICS.md §3, §6): breadth-first search over configurations with
//! a visited set, under a step budget. One definition serves every model. Finite automata also
//! have a fast path (sets of states), which the tests check against this one.

use crate::label::{Dir, LOp};
use crate::machine::Machine;
use crate::model::{AcceptMode, InputMode, Missing, StorageKind};
use crate::value::{Sym, sym_text};
use std::collections::{HashMap, HashSet};

/// The four verdicts of a run (01 §13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accept,
    Reject,
    Loops,
    Unknown,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Accept => "accept",
            Verdict::Reject => "reject",
            Verdict::Loops => "loops",
            Verdict::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tape {
    /// Cells left of the head, nearest last.
    pub left: Vec<Sym>,
    /// The head's cell and everything to its right (never empty; trailing blanks trimmed).
    pub right: Vec<Sym>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    pub q: usize,
    /// Input head (one-way: 0..=|w|; two-way: 0..=|w|+1 over ⊢w⊣).
    pub h: usize,
    /// Stacks, top last.
    pub stacks: Vec<Vec<Sym>>,
    pub tapes: Vec<Tape>,
    pub queue: Vec<Sym>,
    pub counters: Vec<u64>,
    pub out: Vec<Sym>,
}

/// A run's result. `steps` holds the configurations of each BFS depth (one per step for a
/// deterministic machine), when a trace was asked for.
pub struct RunResult {
    pub verdict: Verdict,
    pub steps: Vec<Vec<String>>,
    /// The output of an accepting (or, for transducers, the finished) run.
    pub output: Option<Vec<Sym>>,
}

/// A stuck configuration under `missing error` (a runtime error E5xx).
pub struct Stuck(pub String);

fn trim_blanks(right: &mut Vec<Sym>, blank: &Sym) {
    while right.len() > 1 && right.last() == Some(blank) {
        right.pop();
    }
    if right.is_empty() {
        right.push(blank.clone());
    }
}

impl Machine {
    pub fn initial(&self, w: &[Sym]) -> Config {
        let mut stacks = Vec::new();
        let mut tapes = Vec::new();
        let mut counters = Vec::new();
        for (i, s) in self.eff.storage.iter().enumerate() {
            match s {
                StorageKind::Stack => {
                    let first = !self
                        .eff
                        .storage
                        .iter()
                        .take(i)
                        .any(|k| *k == StorageKind::Stack);
                    match (&self.initial_stack, first) {
                        (Some(z), true) => stacks.push(vec![z.clone()]),
                        _ => stacks.push(Vec::new()),
                    }
                }
                StorageKind::Tape => {
                    let first = tapes.is_empty();
                    let mut right = if first && self.eff.input == InputMode::OnTape {
                        w.to_vec()
                    } else {
                        Vec::new()
                    };
                    trim_blanks(&mut right, &self.blank);
                    tapes.push(Tape {
                        left: Vec::new(),
                        right,
                    });
                }
                StorageKind::Counter => counters.push(0),
                StorageKind::Queue => {}
            }
        }
        Config {
            q: self.start,
            h: 0,
            stacks,
            tapes,
            queue: Vec::new(),
            counters,
            out: Vec::new(),
        }
    }

    /// The input cell under a two-way head (⊢ = 0, ⊣ = |w|+1).
    fn two_way_cell(w: &[Sym], h: usize) -> Option<Sym> {
        if h == 0 {
            Some(Sym::from("⊢"))
        } else if h == w.len() + 1 {
            Some(Sym::from("⊣"))
        } else {
            w.get(h - 1).cloned()
        }
    }

    /// Applies one edge's label to a configuration, if it is applicable.
    pub fn apply(&self, c: &Config, ops: &[LOp], to: usize, w: &[Sym]) -> Option<Config> {
        let mut c = c.clone();
        let bounded_len = w.len().max(1);
        for op in ops {
            match op {
                LOp::Read(t, a) => match self.eff.input {
                    InputMode::OneWay => {
                        if w.get(c.h) != Some(a) {
                            return None;
                        }
                        c.h += 1;
                    }
                    InputMode::TwoWay => {
                        if Self::two_way_cell(w, c.h).as_ref() != Some(a) {
                            return None;
                        }
                    }
                    InputMode::OnTape => {
                        if c.tapes.get(*t)?.right.first() != Some(a) {
                            return None;
                        }
                    }
                },
                LOp::Pop(i, x) => {
                    let s = c.stacks.get_mut(*i)?;
                    if s.last() != Some(x) {
                        return None;
                    }
                    s.pop();
                }
                LOp::Push(i, word) => {
                    let s = c.stacks.get_mut(*i)?;
                    for x in word.iter().rev() {
                        s.push(x.clone());
                    }
                }
                LOp::Dequeue(x) => {
                    if c.queue.first() != Some(x) {
                        return None;
                    }
                    c.queue.remove(0);
                }
                LOp::Enqueue(word) => c.queue.extend(word.iter().cloned()),
                LOp::IfZero(k) => {
                    if *c.counters.get(*k)? != 0 {
                        return None;
                    }
                }
                LOp::IfPos(k) => {
                    if *c.counters.get(*k)? == 0 {
                        return None;
                    }
                }
                LOp::Dec(k) => {
                    let n = c.counters.get_mut(*k)?;
                    if *n == 0 {
                        return None;
                    }
                    *n -= 1;
                }
                LOp::Inc(k) => {
                    let n = c.counters.get_mut(*k)?;
                    *n = n.saturating_add(1);
                }
                LOp::Write(t, x) => {
                    let tape = c.tapes.get_mut(*t)?;
                    if let Some(cell) = tape.right.first_mut() {
                        *cell = x.clone();
                    }
                    trim_blanks(&mut tape.right, &self.blank);
                }
                LOp::Move(t, d) => {
                    let blank = self.blank.clone();
                    let bounded = self.eff.bounded;
                    let tape = c.tapes.get_mut(*t)?;
                    match d {
                        Dir::S => {}
                        Dir::L => {
                            if let Some(x) = tape.left.pop() {
                                tape.right.insert(0, x);
                            }
                        }
                        Dir::R => {
                            if bounded && tape.left.len() + 1 >= bounded_len {
                                // the head stays within the input cells
                            } else {
                                let x = if tape.right.is_empty() {
                                    blank.clone()
                                } else {
                                    tape.right.remove(0)
                                };
                                tape.left.push(x);
                                if tape.right.is_empty() {
                                    tape.right.push(blank.clone());
                                }
                            }
                        }
                    }
                    trim_blanks(&mut tape.right, &blank);
                }
                LOp::MoveInput(d) => match d {
                    Dir::S => {}
                    Dir::L => {
                        if c.h == 0 {
                            return None;
                        }
                        c.h -= 1;
                    }
                    Dir::R => {
                        if c.h > w.len() {
                            return None;
                        }
                        c.h += 1;
                    }
                },
                LOp::Emit(word) => c.out.extend(word.iter().cloned()),
                LOp::Regex(_) => return None,
            }
        }
        c.q = to;
        Some(c)
    }

    fn halts_in(&self, q: usize) -> bool {
        match self.eff.accept {
            AcceptMode::Halting => self.accept.contains(&q) || self.reject == Some(q),
            _ => self.eff.input == InputMode::TwoWay && self.accept.contains(&q),
        }
    }

    pub fn accepting(&self, c: &Config, w: &[Sym]) -> bool {
        let at_end = self.eff.input != InputMode::OneWay || c.h == w.len();
        let stack_empty = c.stacks.first().is_none_or(|s| s.is_empty());
        match self.eff.accept {
            AcceptMode::Final => {
                self.accept.contains(&c.q) && (at_end || self.eff.input == InputMode::TwoWay)
            }
            AcceptMode::Unstated => at_end,
            AcceptMode::EmptyStack => c.h == w.len() && stack_empty,
            AcceptMode::FinalAndEmpty => {
                self.accept.contains(&c.q) && c.h == w.len() && stack_empty
            }
            AcceptMode::Halting => self.accept.contains(&c.q),
        }
    }

    pub fn config_text(&self, c: &Config, w: &[Sym]) -> String {
        let word = |s: &[Sym]| s.iter().map(|x| sym_text(x)).collect::<String>();
        let q = Machine::display_state(&self.state(c.q));
        let mut parts = Vec::new();
        match self.eff.input {
            InputMode::OneWay => {
                let (done, rest) = w.split_at(c.h.min(w.len()));
                parts.push(format!("{q} {}▸{}", word(done), word(rest)));
            }
            InputMode::TwoWay => {
                let mut cells: Vec<Sym> = vec![Sym::from("⊢")];
                cells.extend(w.iter().cloned());
                cells.push(Sym::from("⊣"));
                let (a, b) = cells.split_at(c.h.min(cells.len()));
                parts.push(format!("{q} {}▸{}", word(a), word(b)));
            }
            InputMode::OnTape => parts.push(q),
        }
        for t in &c.tapes {
            parts.push(format!("{}▸{}", word(&t.left), word(&t.right)));
        }
        for s in &c.stacks {
            let top_first: Vec<Sym> = s.iter().rev().cloned().collect();
            parts.push(format!(
                "stack {}",
                if s.is_empty() {
                    "ε".to_string()
                } else {
                    word(&top_first)
                }
            ));
        }
        if self.eff.storage.contains(&StorageKind::Queue) {
            parts.push(format!(
                "queue {}",
                if c.queue.is_empty() {
                    "ε".to_string()
                } else {
                    word(&c.queue)
                }
            ));
        }
        for (i, n) in c.counters.iter().enumerate() {
            parts.push(format!("c{}={n}", i + 1));
        }
        if !c.out.is_empty() {
            parts.push(format!("out {}", word(&c.out)));
        }
        parts.join(" | ")
    }

    /// Runs the machine on `w` with at most `fuel` configurations explored.
    pub fn run(&self, w: &[Sym], fuel: u64, trace: bool) -> Result<RunResult, Stuck> {
        if !trace && self.is_fa() {
            return Ok(self.run_fa(w, fuel));
        }
        let start = self.initial(w);
        let mut seen: HashMap<Config, usize> = HashMap::new();
        let mut graph: Vec<Vec<usize>> = Vec::new();
        let mut nodes: Vec<Config> = Vec::new();
        seen.insert(start.clone(), 0);
        nodes.push(start.clone());
        graph.push(Vec::new());
        let mut frontier = vec![0usize];
        let mut steps: Vec<Vec<String>> = Vec::new();
        let mut explored: u64 = 0;
        let mut final_out: Option<Vec<Sym>> = None;
        while !frontier.is_empty() {
            if trace {
                steps.push(
                    frontier
                        .iter()
                        .filter_map(|i| nodes.get(*i))
                        .map(|c| self.config_text(c, w))
                        .collect(),
                );
            }
            let mut next = Vec::new();
            for &i in &frontier {
                explored += 1;
                if explored > fuel {
                    return Ok(RunResult {
                        verdict: Verdict::Unknown,
                        steps,
                        output: None,
                    });
                }
                let Some(c) = nodes.get(i).cloned() else {
                    continue;
                };
                if self.accepting(&c, w) {
                    return Ok(RunResult {
                        verdict: Verdict::Accept,
                        steps,
                        output: Some(c.out.clone()),
                    });
                }
                if self.halts_in(c.q) {
                    continue;
                }
                let mut any = false;
                for &ei in self.out_edges(c.q) {
                    let Some(e) = self.edges.get(ei) else {
                        continue;
                    };
                    if let Some(c2) = self.apply(&c, &e.label.ops, e.to, w) {
                        any = true;
                        let j = match seen.get(&c2) {
                            Some(&j) => j,
                            None => {
                                let j = nodes.len();
                                seen.insert(c2.clone(), j);
                                nodes.push(c2);
                                graph.push(Vec::new());
                                next.push(j);
                                j
                            }
                        };
                        if let Some(g) = graph.get_mut(i) {
                            g.push(j);
                        }
                    }
                }
                if !any {
                    // A finished transducer run: the whole input was read.
                    if self.eff.output != crate::model::OutputMode::None && c.h == w.len() {
                        final_out = Some(c.out.clone());
                    }
                    if self.eff.missing == Missing::Error {
                        return Err(Stuck(format!(
                            "no transition from configuration {} (the model says a missing transition is an error)",
                            self.config_text(&c, w)
                        )));
                    }
                }
            }
            frontier = next;
        }
        // Everything reachable was explored without accepting.
        let verdict = if self.eff.accept == AcceptMode::Halting && has_cycle(&graph) {
            Verdict::Loops
        } else {
            Verdict::Reject
        };
        Ok(RunResult {
            verdict,
            steps,
            output: final_out,
        })
    }

    /// The fast path for finite automata: sets of states, ε-closed.
    pub fn run_fa(&self, w: &[Sym], fuel: u64) -> RunResult {
        let close = |set: &mut Vec<usize>| {
            let mut i = 0;
            while let Some(&q) = set.get(i) {
                for &ei in self.out_edges(q) {
                    if let Some(e) = self.edges.get(ei)
                        && e.label.ops.is_empty()
                        && !set.contains(&e.to)
                    {
                        set.push(e.to);
                    }
                }
                i += 1;
            }
        };
        let mut cur = vec![self.start];
        close(&mut cur);
        let mut used: u64 = 0;
        for a in w {
            let mut next: Vec<usize> = Vec::new();
            for &q in &cur {
                used += 1;
                for &ei in self.out_edges(q) {
                    if let Some(e) = self.edges.get(ei)
                        && e.label.read() == Some(a)
                        && !next.contains(&e.to)
                    {
                        next.push(e.to);
                    }
                }
            }
            if used > fuel {
                return RunResult {
                    verdict: Verdict::Unknown,
                    steps: Vec::new(),
                    output: None,
                };
            }
            close(&mut next);
            cur = next;
            if cur.is_empty() {
                break;
            }
        }
        let accept = cur.iter().any(|q| self.is_accepting(*q));
        RunResult {
            verdict: if accept {
                Verdict::Accept
            } else {
                Verdict::Reject
            },
            steps: Vec::new(),
            output: None,
        }
    }
}

/// Is there a cycle in the explored configuration graph?
fn has_cycle(graph: &[Vec<usize>]) -> bool {
    let n = graph.len();
    let mut color = vec![0u8; n];
    for root in 0..n {
        if color.get(root).copied() != Some(0) {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        if let Some(c) = color.get_mut(root) {
            *c = 1;
        }
        while let Some(top) = stack.last_mut() {
            let (q, i) = *top;
            match graph.get(q).and_then(|g| g.get(i)) {
                Some(&r) => {
                    top.1 += 1;
                    match color.get(r).copied() {
                        Some(1) => return true,
                        Some(0) => {
                            if let Some(c) = color.get_mut(r) {
                                *c = 1;
                            }
                            stack.push((r, 0));
                        }
                        _ => {}
                    }
                }
                None => {
                    if let Some(c) = color.get_mut(q) {
                        *c = 2;
                    }
                    stack.pop();
                }
            }
        }
    }
    false
}

/// A set of configurations seen, for callers that need it.
pub type Seen = HashSet<Config>;
