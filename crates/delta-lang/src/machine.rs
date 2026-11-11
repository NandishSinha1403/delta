//! The generic machine (SEMANTICS.md §2): any values as states, edges labelled by operation
//! lists, and a model that says how to read them. One struct serves every model.

use crate::label::{LOp, Label};
use crate::model::{AcceptMode, Model};
use crate::value::{Object, Sym, TAG_MACHINE, Value, sym_text};
use delta_core::fa::{Fa, Step};
use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: usize,
    pub label: Label,
    pub to: usize,
}

/// Where a machine came from: the algorithm id and its trace.
#[derive(Clone, Debug)]
pub struct Provenance {
    pub algorithm: String,
    pub steps: Vec<Step>,
}

#[derive(Debug)]
pub struct Machine {
    /// The model as declared (`M.model`).
    pub model: Rc<Model>,
    /// The model with this machine's options applied (`(partial)`, `(missing: …)`, …).
    pub eff: Model,
    pub name: Option<String>,
    /// States in canonical order; an edge refers to a state by its index here.
    pub states: Vec<Value>,
    pub sigma: BTreeSet<Sym>,
    pub stack_alpha: BTreeSet<Sym>,
    pub tape_alpha: BTreeSet<Sym>,
    pub out_alpha: BTreeSet<Sym>,
    pub edges: Vec<Edge>,
    pub start: usize,
    pub accept: BTreeSet<usize>,
    pub reject: Option<usize>,
    pub blank: Sym,
    pub initial_stack: Option<Sym>,
    /// Moore outputs λ(q) by state index (states without one output ε).
    pub lambda: std::collections::BTreeMap<usize, Vec<Sym>>,
    pub provenance: Option<Provenance>,
    /// Outgoing edges per state, built on first use.
    out: OnceCell<Vec<Vec<usize>>>,
    key: OnceCell<String>,
    /// A pushdown machine's grammar (`pda/to-cfg`), built on first membership question.
    cfg: OnceCell<Option<Rc<crate::grammar::Grammar>>>,
}

/// The parts of a machine before indexing.
pub struct Parts {
    pub model: Rc<Model>,
    pub eff: Model,
    pub name: Option<String>,
    pub states: BTreeSet<Value>,
    pub sigma: BTreeSet<Sym>,
    pub stack_alpha: BTreeSet<Sym>,
    pub tape_alpha: BTreeSet<Sym>,
    pub out_alpha: BTreeSet<Sym>,
    pub edges: Vec<(Value, Label, Value)>,
    pub start: Value,
    pub accept: BTreeSet<Value>,
    pub reject: Option<Value>,
    pub blank: Sym,
    pub initial_stack: Option<Sym>,
    /// Moore outputs λ(q).
    pub lambda: std::collections::BTreeMap<Value, Vec<Sym>>,
    pub provenance: Option<Provenance>,
}

impl Machine {
    /// Indexes the parts. Every state mentioned must be in `parts.states`.
    pub fn assemble(p: Parts) -> Machine {
        let states: Vec<Value> = p.states.into_iter().collect();
        let ix = |v: &Value| states.binary_search(v).unwrap_or(0);
        let mut edges: Vec<Edge> = p
            .edges
            .iter()
            .map(|(f, l, t)| Edge {
                from: ix(f),
                label: l.clone(),
                to: ix(t),
            })
            .collect();
        edges.sort();
        edges.dedup();
        let accept = p.accept.iter().map(ix).collect();
        let lambda = p.lambda.iter().map(|(q, w)| (ix(q), w.clone())).collect();
        Machine {
            start: ix(&p.start),
            reject: p.reject.as_ref().map(ix),
            accept,
            edges,
            model: p.model,
            eff: p.eff,
            name: p.name,
            sigma: p.sigma,
            stack_alpha: p.stack_alpha,
            tape_alpha: p.tape_alpha,
            out_alpha: p.out_alpha,
            blank: p.blank,
            initial_stack: p.initial_stack,
            lambda,
            provenance: p.provenance,
            states,
            out: OnceCell::new(),
            key: OnceCell::new(),
            cfg: OnceCell::new(),
        }
    }

    /// Back to parts (to derive a changed machine).
    pub fn parts(&self) -> Parts {
        Parts {
            model: self.model.clone(),
            eff: self.eff.clone(),
            name: self.name.clone(),
            states: self.states.iter().cloned().collect(),
            sigma: self.sigma.clone(),
            stack_alpha: self.stack_alpha.clone(),
            tape_alpha: self.tape_alpha.clone(),
            out_alpha: self.out_alpha.clone(),
            edges: self
                .edges
                .iter()
                .map(|e| (self.state(e.from), e.label.clone(), self.state(e.to)))
                .collect(),
            start: self.state(self.start),
            accept: self.accept.iter().map(|q| self.state(*q)).collect(),
            reject: self.reject.map(|q| self.state(q)),
            blank: self.blank.clone(),
            initial_stack: self.initial_stack.clone(),
            lambda: self
                .lambda
                .iter()
                .map(|(q, w)| (self.state(*q), w.clone()))
                .collect(),
            provenance: self.provenance.clone(),
        }
    }

    pub fn state(&self, q: usize) -> Value {
        self.states.get(q).cloned().unwrap_or(Value::None)
    }

    pub fn index_of(&self, v: &Value) -> Option<usize> {
        self.states.binary_search(v).ok()
    }

    /// Edge indexes leaving each state.
    pub fn out_edges(&self, q: usize) -> &[usize] {
        let out = self.out.get_or_init(|| {
            let mut out = vec![Vec::new(); self.states.len()];
            for (i, e) in self.edges.iter().enumerate() {
                if let Some(list) = out.get_mut(e.from) {
                    list.push(i);
                }
            }
            out
        });
        out.get(q).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn is_accepting(&self, q: usize) -> bool {
        self.eff.accept == AcceptMode::Unstated || self.accept.contains(&q)
    }

    /// A finite automaton whose labels are all "read a" or ε.
    /// For a one-stack, one-way machine: its grammar, so membership is decided exactly (Earley)
    /// instead of by searching configurations, which never ends when ε-moves can grow the stack.
    pub fn pda_grammar(&self) -> Option<Rc<crate::grammar::Grammar>> {
        self.cfg
            .get_or_init(|| crate::pda_cfg::to_cfg(self, 200_000).ok().map(Rc::new))
            .clone()
    }

    pub fn is_fa(&self) -> bool {
        self.eff.is_finite_automaton() && self.edges.iter().all(|e| e.label.is_simple())
    }

    /// As an index automaton over `syms` (sorted; must contain every symbol read).
    pub fn to_fa(&self, syms: &[Sym]) -> Option<Fa> {
        if !self.is_fa() {
            return None;
        }
        let mut f = Fa::new(syms.len(), self.states.len(), self.start);
        for q in &self.accept {
            if let Some(a) = f.accept.get_mut(*q) {
                *a = true;
            }
        }
        if self.eff.accept == AcceptMode::Unstated {
            f.accept.iter_mut().for_each(|a| *a = true);
        }
        for e in &self.edges {
            let sym = match e.label.read() {
                Some(s) => Some(syms.binary_search(s).ok()? as u32),
                None => None,
            };
            f.add_edge(e.from, sym, e.to);
        }
        Some(f)
    }

    pub fn display_state(v: &Value) -> String {
        match v {
            Value::Atom(a) => a.to_string(),
            other => other.display(),
        }
    }

    pub fn label_text(&self, l: &Label) -> String {
        l.display(&self.eff)
    }

    /// `M.edges` labels as values: a symbol, ε (the empty word) or the label text as a word.
    pub fn label_value(&self, l: &Label) -> Value {
        if l.ops.is_empty() {
            return Value::empty_word();
        }
        if let [LOp::Read(0, s)] = l.ops.as_slice() {
            return Value::Sym(s.clone());
        }
        let text = self.label_text(l);
        Value::Word(Rc::new(
            text.chars()
                .map(|c| Sym::from(c.to_string().as_str()))
                .collect(),
        ))
    }

    fn compute_key(&self) -> String {
        let syms = |s: &BTreeSet<Sym>| s.iter().map(|x| sym_text(x)).collect::<Vec<_>>().join(",");
        let states: Vec<String> = self.states.iter().map(|v| v.display()).collect();
        let edges: Vec<String> = self
            .edges
            .iter()
            .map(|e| format!("{}-{:?}->{}", e.from, e.label.ops, e.to))
            .collect();
        format!(
            "{}|{:?}|{:?}|{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}|{}|{:?}|{}",
            self.model.display(),
            self.eff.missing,
            self.eff.accept,
            self.eff.total,
            states.join(","),
            syms(&self.sigma),
            syms(&self.stack_alpha),
            syms(&self.tape_alpha),
            syms(&self.out_alpha),
            edges.join(";"),
            self.start,
            self.accept,
            self.reject.map(|r| r.to_string()).unwrap_or_default(),
            self.initial_stack,
            self.blank
        )
    }

    pub fn text(&self) -> String {
        let name = self.name.clone().unwrap_or_default();
        let mut lines = vec![format!("{} {} {{", self.model.display(), name).replace("  ", " ")];
        lines.push(format!(
            "  states {}",
            self.states
                .iter()
                .map(Self::display_state)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(format!(
            "  alphabet {}",
            self.sigma
                .iter()
                .map(|s| sym_text(s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(format!(
            "  start {}",
            Self::display_state(&self.state(self.start))
        ));
        if !self.accept.is_empty() {
            lines.push(format!(
                "  accept {}",
                self.accept
                    .iter()
                    .map(|q| Self::display_state(&self.state(*q)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        for (q, w) in &self.lambda {
            let out: String = w.iter().map(|s| sym_text(s)).collect();
            lines.push(format!(
                "  λ({}) = {}",
                Self::display_state(&self.state(*q)),
                if out.is_empty() {
                    crate::notation::eps()
                } else {
                    format!("\"{out}\"")
                }
            ));
        }
        for e in &self.edges {
            lines.push(format!(
                "  {} -[{}]-> {}",
                Self::display_state(&self.state(e.from)),
                self.label_text(&e.label),
                Self::display_state(&self.state(e.to))
            ));
        }
        lines.push("}".to_string());
        lines.join("\n")
    }
}

impl Object for Machine {
    fn tag(&self) -> u8 {
        TAG_MACHINE
    }
    fn key(&self) -> String {
        self.key.get_or_init(|| self.compute_key()).clone()
    }
    fn type_name(&self) -> &'static str {
        "machine"
    }
    fn display(&self) -> String {
        match &self.name {
            Some(n) => format!(
                "{} {n} ({} states)",
                self.model.display(),
                self.states.len()
            ),
            None => format!("{} ({} states)", self.model.display(), self.states.len()),
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}
