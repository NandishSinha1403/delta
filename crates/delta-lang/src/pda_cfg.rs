//! Grammars ↔ pushdown automata (SEMANTICS §11.5–11.6):
//! - `cfg/to-pda`: three states; the stack holds what is left of a leftmost sentential form.
//! - `pda/to-cfg`: normalize the machine (one start that pushes `$`, every edge pushes or pops
//!   exactly one symbol, accepting runs drain the stack into one state), then one variable
//!   ⟨p,q⟩ per pair of states for "from p to q, stack as it was".

use crate::grammar::{GSym, Grammar};
use crate::label::{LOp, Label};
use crate::machine::{Machine, Parts, Provenance};
use crate::model::{AcceptMode, InputMode, Model, StorageKind};
use crate::value::{Sym, Value, sym_text};
use delta_core::fa::Step;
// `BTreeSet`/`BTreeMap` keep their items sorted, so results come out in a stable order.
use std::collections::{BTreeMap, BTreeSet};
// `Rc` shares one value between several owners (here: the model of the result).
use std::rc::Rc;

fn atom(s: &str) -> Value {
    Value::Atom(Rc::from(s))
}

/// A stack symbol not in `used`: `base`, else `base1`, `base2`, …
fn fresh_sym(used: &BTreeSet<Sym>, base: &str) -> Sym {
    let mut name = base.to_string();
    let mut i = 1;
    while used.contains(&Sym::from(name.as_str())) {
        name = format!("{base}{i}");
        i += 1;
    }
    Sym::from(name.as_str())
}

/// The pushdown automaton of a grammar (`cfg/to-pda`).
pub fn to_pda(g: &Grammar, model: Rc<Model>) -> Machine {
    let var = |v: usize| Sym::from(g.vars.get(v).map(String::as_str).unwrap_or("S"));
    let mut used: BTreeSet<Sym> = g.terminals.clone();
    used.extend((0..g.vars.len()).map(var));
    let bottom = fresh_sym(&used, "$");
    let sym = |s: &GSym| match s {
        GSym::V(v) => var(*v),
        GSym::T(t) => t.clone(),
    };
    let (start, lp, acc) = (atom("start"), atom("loop"), atom("accept"));
    let mut edges = vec![(
        start.clone(),
        Label::new(vec![LOp::Push(0, vec![var(g.start), bottom.clone()])]),
        lp.clone(),
    )];
    let mut steps = vec![Step::new(
        "AddEdge",
        format!(
            "start: push {}{} and go to loop",
            sym_text(&var(g.start)),
            sym_text(&bottom)
        ),
    )];
    // manual:begin to-pda-rules
    for (l, rhs) in &g.rules {
        let mut ops = vec![LOp::Pop(0, var(*l))];
        if !rhs.is_empty() {
            ops.push(LOp::Push(0, rhs.iter().map(sym).collect()));
        }
        edges.push((lp.clone(), Label::new(ops), lp.clone()));
    }
    // manual:end to-pda-rules
    steps.push(Step::new(
        "AddEdge",
        format!(
            "{} rules: replace the variable on top by a right side",
            g.rules.len()
        ),
    ));
    for t in &g.terminals {
        edges.push((
            lp.clone(),
            Label::new(vec![LOp::Read(0, t.clone()), LOp::Pop(0, t.clone())]),
            lp.clone(),
        ));
    }
    steps.push(Step::new(
        "AddEdge",
        "each terminal: read it and pop it".to_string(),
    ));
    edges.push((
        lp.clone(),
        Label::new(vec![LOp::Pop(0, bottom.clone())]),
        acc.clone(),
    ));
    steps.push(Step::new(
        "AddEdge",
        format!("pop {} and accept", sym_text(&bottom)),
    ));
    let mut stack_alpha = used;
    stack_alpha.insert(bottom);
    let eff = (*model).clone();
    Machine::assemble(Parts {
        model,
        eff,
        name: None,
        states: [start.clone(), lp, acc.clone()].into_iter().collect(),
        sigma: g.terminals.clone(),
        stack_alpha,
        tape_alpha: BTreeSet::new(),
        out_alpha: BTreeSet::new(),
        edges,
        start,
        accept: [acc].into_iter().collect(),
        reject: None,
        blank: crate::label::blank(),
        initial_stack: None,
        lambda: BTreeMap::new(),
        provenance: Some(Provenance {
            algorithm: "cfg/to-pda".to_string(),
            steps,
        }),
    })
}

/// One normalized edge: read (or ε), then exactly one push or one pop.
// A `struct` groups named fields into one type.
struct Move {
    from: usize,
    read: Option<Sym>,
    push: bool,
    sym: Sym,
    to: usize,
}

/// The grammar of a one-stack, one-way machine (`pda/to-cfg`). Err names what isn't supported.
pub fn to_cfg(m: &Machine, max_rules: usize) -> Result<Grammar, String> {
    if m.eff.input != InputMode::OneWay || m.eff.storage != [StorageKind::Stack] {
        return Err("to_cfg needs a pushdown automaton (one stack, one-way input)".into());
    }
    // State names: P's states, then helpers.
    let mut names: Vec<String> = m.states.iter().map(Machine::display_state).collect();
    let new_state = |names: &mut Vec<String>, base: &str| -> usize {
        let mut name = base.to_string();
        let mut i = 1;
        while names.contains(&name) {
            name = format!("{base}{i}");
            i += 1;
        }
        names.push(name);
        names.len() - 1
    };
    let mut gamma: BTreeSet<Sym> = m.stack_alpha.clone();
    for e in &m.edges {
        for op in &e.label.ops {
            match op {
                LOp::Pop(_, x) => {
                    gamma.insert(x.clone());
                }
                LOp::Push(_, w) => gamma.extend(w.iter().cloned()),
                _ => {}
            }
        }
    }
    if let Some(z) = &m.initial_stack {
        gamma.insert(z.clone());
    }
    let bottom = fresh_sym(&gamma, "$");
    let mut used = gamma.clone();
    used.insert(bottom.clone());
    let hash = fresh_sym(&used, "#");

    let mut moves: Vec<Move> = Vec::new();
    let mut steps = Vec::new();
    // A closure (`|args| body`) is an unnamed function that can use the variables around it.
    // Chains of single moves through fresh states; `read` goes on the first.
    let chain = |moves: &mut Vec<Move>,
                 names: &mut Vec<String>,
                 from: usize,
                 read: Option<Sym>,
                 ops: Vec<(bool, Sym)>,
                 to: usize| {
        let mut at = from;
        let mut read = read;
        let n = ops.len();
        for (i, (push, sym)) in ops.into_iter().enumerate() {
            let next = if i + 1 == n {
                to
            } else {
                new_state(names, "m")
            };
            moves.push(Move {
                from: at,
                read: read.take(),
                push,
                sym,
                to: next,
            });
            at = next;
        }
    };
    let s0 = new_state(&mut names, "s0");
    let mut init = vec![(true, bottom.clone())];
    if let Some(z) = &m.initial_stack {
        init.push((true, z.clone()));
    }
    chain(&mut moves, &mut names, s0, None, init, m.start);
    steps.push(Step::new(
        "GrammarEdit",
        format!(
            "new start {}: push {} and go to {}",
            names[s0],
            sym_text(&bottom),
            names[m.start]
        ),
    ));
    for e in &m.edges {
        let mut read = None;
        let mut ops: Vec<(bool, Sym)> = Vec::new();
        for op in &e.label.ops {
            match op {
                LOp::Read(0, a) => read = Some(a.clone()),
                LOp::Pop(0, x) => ops.push((false, x.clone())),
                LOp::Push(0, w) => ops.extend(w.iter().rev().map(|s| (true, s.clone()))),
                other => return Err(format!("to_cfg can't convert the operation `{other:?}`")),
            }
        }
        // Pops before pushes (the label's canonical order already puts Pop first).
        if ops.is_empty() {
            ops = vec![(true, hash.clone()), (false, hash.clone())];
        }
        chain(&mut moves, &mut names, e.from, read, ops, e.to);
    }
    steps.push(Step::new(
        "GrammarEdit",
        "split every edge so it pushes or pops exactly one symbol".to_string(),
    ));
    let drain = new_state(&mut names, "drain");
    let acc = new_state(&mut names, "acc");
    match m.eff.accept {
        AcceptMode::EmptyStack => {
            for p in 0..m.states.len() {
                moves.push(Move {
                    from: p,
                    read: None,
                    push: false,
                    sym: bottom.clone(),
                    to: acc,
                });
            }
        }
        _ => {
            for &f in &m.accept {
                chain(
                    &mut moves,
                    &mut names,
                    f,
                    None,
                    vec![(true, hash.clone()), (false, hash.clone())],
                    drain,
                );
            }
            for t in gamma.iter().chain(std::iter::once(&hash)) {
                moves.push(Move {
                    from: drain,
                    read: None,
                    push: false,
                    sym: t.clone(),
                    to: drain,
                });
            }
            moves.push(Move {
                from: drain,
                read: None,
                push: false,
                sym: bottom.clone(),
                to: acc,
            });
        }
    }
    steps.push(Step::new(
        "GrammarEdit",
        format!("accepting runs empty the stack and end in {}", names[acc]),
    ));

    // Variables ⟨p,q⟩.
    let n = names.len();
    let vars: Vec<String> = (0..n * n)
        .map(|i| format!("{},{}", names[i / n], names[i % n]))
        .collect();
    let v = |p: usize, q: usize| p * n + q;
    let mut rules: Vec<(usize, Vec<GSym>)> = Vec::new();
    for p in 0..n {
        rules.push((v(p, p), Vec::new()));
    }
    // Only states that occur on a move matter for the split rules.
    let live: BTreeSet<usize> = moves.iter().flat_map(|mv| [mv.from, mv.to]).collect();
    if live.len().pow(3) > max_rules {
        return Err(format!(
            "the machine has {} states after normalizing; the grammar would need more than {max_rules} rules (raise budget states)",
            live.len()
        ));
    }
    for &p in &live {
        for &q in &live {
            for &r in &live {
                rules.push((v(p, q), vec![GSym::V(v(p, r)), GSym::V(v(r, q))]));
            }
        }
    }
    // manual:begin to-cfg-pairs
    for a in moves.iter().filter(|mv| mv.push) {
        for b in moves.iter().filter(|mv| !mv.push && mv.sym == a.sym) {
            let mut rhs = Vec::new();
            rhs.extend(a.read.iter().cloned().map(GSym::T));
            rhs.push(GSym::V(v(a.to, b.from)));
            rhs.extend(b.read.iter().cloned().map(GSym::T));
            rules.push((v(a.from, b.to), rhs));
        }
    }
    // manual:end to-cfg-pairs
    steps.push(Step::new(
        "GrammarEdit",
        format!(
            "{} rules over ⟨p,q⟩ before removing useless variables",
            rules.len()
        ),
    ));
    let prov = Provenance {
        algorithm: "pda/to-cfg".to_string(),
        steps,
    };
    Ok(Grammar::build(None, vars, v(s0, acc), rules, Some(prov)).trim())
}
