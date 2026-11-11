//! Building machines from literals (01-LANGUAGE.md §5, SEMANTICS.md §10.3): fields and their
//! aliases, `for`/`if` expansion, the four transition forms, inference of states and alphabets,
//! options, and the model rules (E3xx).

use crate::ast::*;
use crate::diag::{Diagnostic, Span};
use crate::interp::{Flow, Interp, R};
use crate::label::{self, LOp, Label};
use crate::machine::{Machine, Parts};
use crate::model::{AcceptMode, InputMode, Missing, Model};
use crate::value::{Sym, Value, sym_text};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

/// What the body says, before checking.
#[derive(Default)]
struct Body {
    states: Vec<(Value, Span)>,
    states_declared: bool,
    sigma: Option<Vec<(Sym, Span)>>,
    stack: Option<Vec<(Sym, Span)>>,
    tape: Option<Vec<(Sym, Span)>>,
    output: Option<Vec<(Sym, Span)>>,
    start: Option<(Value, Span)>,
    accept: Vec<(Value, Span)>,
    reject: Option<(Value, Span)>,
    blank: Option<Sym>,
    /// (from, label, to, span of the edge)
    edges: Vec<(Value, Label, Value, Span)>,
    mentioned: Vec<(Value, Span)>,
    /// `λ(q) = w` lines: (state, output, span).
    lambda: Vec<(Value, Vec<Sym>, Span)>,
}

fn e3(code: &'static str, span: Span, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::error(code, span, msg)
}

impl Interp<'_> {
    /// A model by name, from the prelude or a `model` declaration.
    pub fn model_named(&self, name: &str) -> Option<Rc<Model>> {
        self.models.get(name).cloned()
    }

    fn state_value(&mut self, s: &MState) -> R<(Value, Span)> {
        match s {
            MState::Name(n, sp) => Ok((Value::Atom(Rc::from(n.as_str())), *sp)),
            MState::Expr(e) => Ok((self.eval(e)?, e.span)),
        }
    }

    /// Values in state context: bare names are atoms, sets are spliced.
    fn mval_states(&mut self, v: &MVal, out: &mut Vec<(Value, Span)>) -> R<()> {
        match v {
            MVal::Name(n, s) => out.push((Value::Atom(Rc::from(n.as_str())), *s)),
            MVal::Sym(x, s) => out.push((Value::sym(x), *s)),
            MVal::Eps(s) => out.push((Value::empty_word(), *s)),
            MVal::Blank(s) => out.push((Value::Sym(self.blank_sym()), *s)),
            MVal::Set(vs, _) => {
                for v in vs {
                    self.mval_states(v, out)?;
                }
            }
            MVal::Expr(e) => {
                let x = self.eval(e)?;
                out.push((x, e.span));
            }
        }
        Ok(())
    }

    fn blank_sym(&self) -> Sym {
        label::blank()
    }

    /// Values in symbol context: bare `0` is the symbol '0' (Q-04).
    /// An output word in `λ(q) = w`: ε is empty, a name or number is its characters, an
    /// expression is a word, symbol or number.
    fn mval_word(&mut self, v: &MVal, out: &mut Vec<Sym>) -> R<()> {
        let chars = |t: &str, out: &mut Vec<Sym>| {
            out.extend(t.chars().map(|c| Sym::from(c.to_string().as_str())))
        };
        match v {
            MVal::Eps(_) => {}
            MVal::Name(n, _) => chars(n, out),
            MVal::Sym(n, _) => out.push(Sym::from(n.as_str())),
            MVal::Blank(_) => out.push(self.blank_sym()),
            MVal::Set(_, s) => {
                return Err(Flow::Error(e3(
                    "E309",
                    *s,
                    "λ(q) = w takes a word, not a set",
                )));
            }
            MVal::Expr(e) => match self.eval(e)? {
                Value::Word(w) => out.extend(w.iter().cloned()),
                Value::Sym(s) => out.push(s),
                Value::Int(n) => chars(&n.to_string(), out),
                other => {
                    return Err(Flow::Error(e3(
                        "E504",
                        e.span,
                        format!("λ(q) = w needs a word, found {}", other.display()),
                    )));
                }
            },
        }
        Ok(())
    }

    fn mval_syms(&mut self, v: &MVal, out: &mut Vec<(Sym, Span)>) -> R<()> {
        match v {
            MVal::Name(n, s) | MVal::Sym(n, s) => out.push((Sym::from(n.as_str()), *s)),
            MVal::Blank(s) => out.push((self.blank_sym(), *s)),
            MVal::Eps(s) => {
                return Err(Flow::Error(e3(
                    "E304",
                    *s,
                    "ε is not a symbol; an alphabet lists symbols",
                )));
            }
            MVal::Set(vs, _) => {
                for v in vs {
                    self.mval_syms(v, out)?;
                }
            }
            MVal::Expr(e) => {
                let x = self.eval(e)?;
                self.value_syms(&x, e.span, out)?;
            }
        }
        Ok(())
    }

    pub fn value_syms(&mut self, x: &Value, span: Span, out: &mut Vec<(Sym, Span)>) -> R<()> {
        match x {
            Value::Sym(s) => out.push((s.clone(), span)),
            Value::Int(n) => out.push((Sym::from(n.to_string().as_str()), span)),
            Value::Word(w) if w.len() == 1 => out.extend(w.iter().map(|s| (s.clone(), span))),
            Value::Set(items) => {
                for i in items.iter() {
                    self.value_syms(i, span, out)?;
                }
            }
            Value::List(items) => {
                for i in items.iter() {
                    self.value_syms(i, span, out)?;
                }
            }
            other => {
                return Err(Flow::Error(e3(
                    "E304",
                    span,
                    format!(
                        "expected symbols here, found {} {}",
                        other.type_name(),
                        other.display()
                    ),
                )));
            }
        }
        Ok(())
    }

    fn collect(&mut self, items: &[MItem], eff: &Model, body: &mut Body) -> R<()> {
        for it in items {
            self.tick(Span::default())?;
            match it {
                MItem::Field {
                    name,
                    name_span,
                    values,
                } => self.field(name, *name_span, values, eff, body)?,
                MItem::Edge {
                    from,
                    label: l,
                    to,
                    span,
                } => {
                    let (f, fs) = self.state_value(from)?;
                    let (t, ts) = self.state_value(to)?;
                    body.mentioned.push((f.clone(), fs));
                    body.mentioned.push((t.clone(), ts));
                    let blank = body.blank.clone().unwrap_or_else(label::blank);
                    let labels = label::parse_labels(&l.text, eff, &blank).map_err(|msg| {
                        Flow::Error(e3(
                            "E309",
                            l.span,
                            format!("bad {} label `{}`: {msg}", eff.display(), l.text.trim()),
                        ))
                    })?;
                    for lab in labels {
                        body.edges.push((f.clone(), lab, t.clone(), *span));
                    }
                }
                MItem::Delta {
                    args,
                    targets,
                    span,
                } => {
                    let mut st = Vec::new();
                    if let Some(a0) = args.first() {
                        self.mval_states(a0, &mut st)?;
                    }
                    let Some((from, fs)) = st.pop() else {
                        return Err(Flow::Error(e3(
                            "E309",
                            *span,
                            "δ(q, a) needs a state and a symbol",
                        )));
                    };
                    let sym = match args.get(1) {
                        Some(MVal::Eps(_)) | None => None,
                        Some(v) => {
                            let mut syms = Vec::new();
                            self.mval_syms(v, &mut syms)?;
                            syms.first().map(|(s, _)| s.clone())
                        }
                    };
                    if args.len() > 2 {
                        return Err(Flow::Error(e3(
                            "E309",
                            *span,
                            "the δ(…) = … form takes a state and a symbol; for storage models use arrows `-label->`",
                        )));
                    }
                    let mut ts = Vec::new();
                    for t in targets {
                        self.mval_states(t, &mut ts)?;
                    }
                    body.mentioned.push((from.clone(), fs));
                    for (t, tspan) in ts {
                        body.mentioned.push((t.clone(), tspan));
                        body.edges
                            .push((from.clone(), label::read_label(sym.clone()), t, *span));
                    }
                }
                MItem::Lambda { state, out, span } => {
                    if eff.output != crate::model::OutputMode::PerState {
                        return Err(Flow::Error(e3(
                            "E309",
                            *span,
                            "λ(q) = w gives a state's output; only per-state output models (moore) have one",
                        )));
                    }
                    let mut st = Vec::new();
                    self.mval_states(state, &mut st)?;
                    let Some((q, qs)) = st.pop() else {
                        return Err(Flow::Error(e3("E309", *span, "λ(q) needs a state")));
                    };
                    let mut word = Vec::new();
                    for v in out {
                        self.mval_word(v, &mut word)?;
                    }
                    body.mentioned.push((q.clone(), qs));
                    body.lambda.push((q, word, *span));
                }
                MItem::Table { header, rows, span } => {
                    let mut syms: Vec<Option<Sym>> = Vec::new();
                    for h in header {
                        match h {
                            MVal::Eps(_) => syms.push(None),
                            other => {
                                let mut s = Vec::new();
                                self.mval_syms(other, &mut s)?;
                                syms.push(s.first().map(|(x, _)| x.clone()));
                            }
                        }
                    }
                    for (state, cells) in rows {
                        let (from, fs) = self.state_value(state)?;
                        body.mentioned.push((from.clone(), fs));
                        for (i, cell) in cells.iter().enumerate() {
                            let Some(sym) = syms.get(i) else {
                                return Err(Flow::Error(e3(
                                    "E309",
                                    *span,
                                    "a table row has more cells than the header",
                                )));
                            };
                            let mut ts = Vec::new();
                            for v in cell {
                                self.mval_states(v, &mut ts)?;
                            }
                            for (t, tspan) in ts {
                                body.mentioned.push((t.clone(), tspan));
                                body.edges.push((
                                    from.clone(),
                                    label::read_label(sym.clone()),
                                    t,
                                    *span,
                                ));
                            }
                        }
                    }
                }
                MItem::For {
                    pat,
                    iter,
                    body: inner,
                } => {
                    let items = self.iterate(iter)?;
                    for item in items {
                        self.scopes.push(HashMap::new());
                        let r = self
                            .bind_pattern(pat, item, false)
                            .and_then(|_| self.collect(inner, eff, body));
                        self.scopes.pop();
                        r?;
                    }
                }
                MItem::If { cond, then, els } => {
                    if self.eval_bool(cond)? {
                        self.collect(then, eff, body)?;
                    } else {
                        self.collect(els, eff, body)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn field(
        &mut self,
        name: &str,
        span: Span,
        values: &[MVal],
        eff: &Model,
        body: &mut Body,
    ) -> R<()> {
        let syms = |me: &mut Self| -> R<Vec<(Sym, Span)>> {
            let mut out = Vec::new();
            for v in values {
                me.mval_syms(v, &mut out)?;
            }
            Ok(out)
        };
        let states = |me: &mut Self| -> R<Vec<(Value, Span)>> {
            let mut out = Vec::new();
            for v in values {
                me.mval_states(v, &mut out)?;
            }
            Ok(out)
        };
        match name {
            "states" | "Q" => {
                body.states.extend(states(self)?);
                body.states_declared = true;
            }
            "alphabet" | "Σ" | "input" => {
                body.sigma.get_or_insert_with(Vec::new).extend(syms(self)?)
            }
            "stack" => body.stack.get_or_insert_with(Vec::new).extend(syms(self)?),
            "tape" => body.tape.get_or_insert_with(Vec::new).extend(syms(self)?),
            "Γ" => {
                if eff.has_stack() {
                    body.stack.get_or_insert_with(Vec::new).extend(syms(self)?);
                } else {
                    body.tape.get_or_insert_with(Vec::new).extend(syms(self)?);
                }
            }
            "output" | "Δ" => body.output.get_or_insert_with(Vec::new).extend(syms(self)?),
            "start" | "s" => {
                let mut st = states(self)?;
                if st.len() != 1 {
                    return Err(Flow::Error(e3(
                        "E302",
                        span,
                        "`start` takes exactly one state",
                    )));
                }
                body.start = st.pop();
            }
            "accept" | "F" | "final" => body.accept.extend(states(self)?),
            "reject" => {
                let mut st = states(self)?;
                if st.len() != 1 {
                    return Err(Flow::Error(e3(
                        "E302",
                        span,
                        "`reject` takes exactly one state",
                    )));
                }
                body.reject = st.pop();
            }
            "blank" => {
                let s = syms(self)?;
                body.blank = s.first().map(|(x, _)| x.clone());
            }
            other => {
                return Err(Flow::Error(e3(
                    "E309",
                    span,
                    format!("unknown field `{other}`"),
                )));
            }
        }
        Ok(())
    }

    /// The effective model and initial stack after the options `(partial)`, `(missing: m)`,
    /// `(accept: by …)` and `(initial stack Z)`.
    fn apply_options(&self, model: &Model, options: &[(String, Span)]) -> R<(Model, Option<Sym>)> {
        let mut eff = model.clone();
        let mut initial = None;
        for (opt, span) in options {
            let words: Vec<&str> = opt
                .split(|c: char| c.is_whitespace() || c == ':')
                .filter(|w| !w.is_empty())
                .collect();
            match words.as_slice() {
                ["partial"] => {
                    eff.total = false;
                    eff.missing = Missing::Dead;
                }
                ["missing", m] => {
                    eff.apply(&["missing".to_string(), m.to_string()], &HashMap::new())
                        .map_err(|msg| Flow::Error(e3("E312", *span, msg)))?;
                }
                ["accept", "by", how] => {
                    eff.apply(
                        &["accept".into(), "by".into(), how.to_string()],
                        &HashMap::new(),
                    )
                    .map_err(|msg| Flow::Error(e3("E312", *span, msg)))?;
                }
                ["initial", "stack", z] => {
                    if !eff.has_stack() {
                        return Err(Flow::Error(e3(
                            "E312",
                            *span,
                            "`initial stack` needs a model with a stack",
                        )));
                    }
                    initial = Some(Sym::from(
                        z.trim_matches(|c| c == '<' || c == '>' || c == '\''),
                    ));
                }
                _ => {
                    return Err(Flow::Error(e3(
                        "E312",
                        *span,
                        format!(
                            "unknown option `{opt}`: use partial, missing: …, accept: by … or initial stack Z"
                        ),
                    )));
                }
            }
        }
        Ok((eff, initial))
    }

    pub fn machine_literal(&mut self, m: &MachineLit, span: Span) -> R<Value> {
        let Some(model) = self.model_named(&m.model) else {
            return Err(Flow::Error(e3(
                "E301",
                m.model_span,
                format!("unknown model `{}`", m.model),
            )));
        };
        let (eff, initial_stack) = self.apply_options(&model, &m.options)?;
        let mut body = Body::default();
        self.collect(&m.items, &eff, &mut body)?;
        let mut errors: Vec<Diagnostic> = Vec::new();

        // Strict mode: every field is written (01 §13).
        if self.strict {
            if !body.states_declared {
                errors.push(e3(
                    "E308",
                    m.model_span,
                    "strict mode: write the `states` field",
                ));
            }
            if body.sigma.is_none() {
                errors.push(e3(
                    "E308",
                    m.model_span,
                    "strict mode: write the `alphabet` field",
                ));
            }
        }
        let Some((start, start_span)) = body.start.clone() else {
            errors.push(e3(
                "E302",
                m.model_span,
                "the machine has no start state: add `start q0`",
            ));
            return self.fail(errors);
        };

        // States: declared, or (lenient) everything mentioned.
        let mut states: BTreeSet<Value> = body.states.iter().map(|(v, _)| v.clone()).collect();
        let mut mentioned = body.mentioned.clone();
        mentioned.push((start.clone(), start_span));
        mentioned.extend(body.accept.iter().cloned());
        if let Some(r) = &body.reject {
            mentioned.push(r.clone());
        }
        if body.states_declared {
            for (v, s) in &mentioned {
                if !states.contains(v) {
                    errors.push(e3(
                        "E303",
                        *s,
                        format!(
                            "`{}` is not one of the declared states",
                            Machine::display_state(v)
                        ),
                    ));
                }
            }
        } else {
            states.extend(mentioned.iter().map(|(v, _)| v.clone()));
        }

        let blank = body.blank.clone().unwrap_or_else(label::blank);
        // Alphabets.
        let mut read: BTreeSet<Sym> = BTreeSet::new();
        let mut written: BTreeSet<Sym> = BTreeSet::new();
        let mut stack_syms: BTreeSet<Sym> = BTreeSet::new();
        let mut emitted: BTreeSet<Sym> = BTreeSet::new();
        for (_, l, _, _) in &body.edges {
            for op in &l.ops {
                match op {
                    LOp::Read(_, s) => {
                        read.insert(s.clone());
                    }
                    LOp::Write(_, s) => {
                        written.insert(s.clone());
                    }
                    LOp::Pop(_, s) | LOp::Dequeue(s) => {
                        stack_syms.insert(s.clone());
                    }
                    LOp::Push(_, w) | LOp::Enqueue(w) => stack_syms.extend(w.iter().cloned()),
                    LOp::Emit(w) => emitted.extend(w.iter().cloned()),
                    _ => {}
                }
            }
        }
        let on_tape = eff.input == InputMode::OnTape;
        let sigma: BTreeSet<Sym> = match &body.sigma {
            Some(s) => s.iter().map(|(x, _)| x.clone()).collect(),
            None if on_tape => read
                .iter()
                .filter(|s| **s != blank && !written.contains(*s))
                .cloned()
                .collect(),
            None => read
                .iter()
                .filter(|s| eff.input != InputMode::TwoWay || (&***s != "⊢" && &***s != "⊣"))
                .cloned()
                .collect(),
        };
        if body.sigma.is_some() && !on_tape {
            for (_, l, _, span) in &body.edges {
                if let Some(s) = l.read() {
                    let endmark = eff.input == InputMode::TwoWay && (&**s == "⊢" || &**s == "⊣");
                    if !sigma.contains(s) && !endmark {
                        errors.push(e3(
                            "E304",
                            *span,
                            format!(
                                "symbol '{}' is not in the alphabet {{{}}}",
                                sym_text(s),
                                join_syms(&sigma)
                            ),
                        ));
                        break;
                    }
                }
            }
        }
        let mut tape_alpha: BTreeSet<Sym> = BTreeSet::new();
        if on_tape || eff.tapes() > 0 {
            tape_alpha.extend(sigma.iter().cloned());
            tape_alpha.extend(read.iter().cloned());
            tape_alpha.extend(written.iter().cloned());
            tape_alpha.insert(blank.clone());
            if let Some(t) = &body.tape {
                tape_alpha.extend(t.iter().map(|(x, _)| x.clone()));
            }
        }
        let mut stack_alpha: BTreeSet<Sym> = match &body.stack {
            Some(s) => s.iter().map(|(x, _)| x.clone()).collect(),
            None => stack_syms.clone(),
        };
        if let Some(z) = &initial_stack {
            if body.stack.is_some() && !stack_alpha.contains(z) {
                errors.push(e3(
                    "E313",
                    m.model_span,
                    format!("the initial stack symbol {} is not in Γ", sym_text(z)),
                ));
            }
            stack_alpha.insert(z.clone());
        }
        let mut out_alpha: BTreeSet<Sym> = body
            .output
            .as_ref()
            .map(|o| o.iter().map(|(x, _)| x.clone()).collect())
            .unwrap_or_default();
        out_alpha.extend(emitted);
        out_alpha.extend(body.lambda.iter().flat_map(|(_, w, _)| w.iter().cloned()));
        let lambda: std::collections::BTreeMap<Value, Vec<Sym>> = body
            .lambda
            .iter()
            .map(|(q, w, _)| (q.clone(), w.clone()))
            .collect();

        let mut accept: BTreeSet<Value> = body.accept.iter().map(|(v, _)| v.clone()).collect();
        if eff.accept == AcceptMode::Unstated {
            accept = states.clone();
        }

        // Model rules.
        if !eff.epsilon && eff.input == InputMode::OneWay && !eff.labels_regex {
            for (f, l, _, span) in &body.edges {
                if l.read().is_none() && l.ops.iter().all(|o| !matches!(o, LOp::Emit(_))) {
                    errors.push(e3(
                        "E307",
                        *span,
                        format!(
                            "{} doesn't allow ε-moves, but this edge from `{}` reads nothing",
                            eff.display(),
                            Machine::display_state(f)
                        ),
                    ));
                    break;
                }
            }
        }
        if eff.deterministic
            && let Some(d) = determinism_error(&body.edges, &eff)
        {
            errors.push(d);
        }
        if eff.deterministic
            && eff.total
            && eff.is_finite_automaton()
            && errors.is_empty()
            && let Some(d) = self.totality_error(m, &states, &sigma, &body.edges)
        {
            errors.push(d);
        }
        if !errors.is_empty() {
            return self.fail(errors);
        }
        let machine = Machine::assemble(Parts {
            model,
            eff,
            name: m.name.clone(),
            states,
            sigma,
            stack_alpha,
            tape_alpha,
            out_alpha,
            edges: body
                .edges
                .into_iter()
                .map(|(f, l, t, _)| (f, l, t))
                .collect(),
            start,
            accept,
            reject: body.reject.map(|(v, _)| v),
            blank,
            initial_stack,
            lambda,
            provenance: None,
        });
        let _ = span;
        Ok(Value::Obj(Rc::new(machine)))
    }

    /// Reports every error: the first stops the program, the rest are kept as diagnostics.
    fn fail<T>(&mut self, mut errors: Vec<Diagnostic>) -> R<T> {
        if errors.is_empty() {
            errors.push(Diagnostic::error(
                "X999",
                Span::default(),
                "internal: a machine failed without a reason",
            ));
        }
        let first = errors.remove(0);
        self.pending.extend(errors);
        Err(Flow::Error(first))
    }

    /// `total` for a deterministic finite automaton: every (state, symbol) has an edge. The
    /// error offers two fixes: add a dead state, or mark the machine `(partial)`.
    fn totality_error(
        &self,
        m: &MachineLit,
        states: &BTreeSet<Value>,
        sigma: &BTreeSet<Sym>,
        edges: &[(Value, Label, Value, Span)],
    ) -> Option<Diagnostic> {
        let mut have: BTreeMap<(Value, Sym), ()> = BTreeMap::new();
        for (f, l, _, _) in edges {
            if let Some(s) = l.read() {
                have.insert((f.clone(), s.clone()), ());
            }
        }
        let mut missing = Vec::new();
        for q in states {
            for a in sigma {
                if !have.contains_key(&(q.clone(), a.clone())) {
                    missing.push((q.clone(), a.clone()));
                }
            }
        }
        if missing.is_empty() {
            return None;
        }
        let listed: Vec<String> = missing
            .iter()
            .take(4)
            .map(|(q, a)| format!("({}, '{}')", Machine::display_state(q), sym_text(a)))
            .collect();
        let more = if missing.len() > 4 {
            format!(" and {} more", missing.len() - 4)
        } else {
            String::new()
        };
        let name = m.name.clone().unwrap_or_default();
        let msg = format!(
            "{} {}: no transition for {}{more}. A total deterministic model needs one for every state and symbol",
            m.model,
            name,
            listed.join(", ")
        );
        let simple_states = missing.iter().all(|(q, _)| matches!(q, Value::Atom(_)));
        let dead = (0..)
            .map(|i| {
                if i == 0 {
                    "dead".to_string()
                } else {
                    format!("dead{i}")
                }
            })
            .find(|n| !states.contains(&Value::Atom(Rc::from(n.as_str()))))
            .unwrap_or_else(|| "dead".to_string());
        let close = Span::new(m.span.to.saturating_sub(1), m.span.to.saturating_sub(1));
        let mut lines = String::new();
        for (q, a) in &missing {
            lines.push_str(&format!(
                "  {} -{}-> {dead}\n",
                Machine::display_state(q),
                sym_text(a)
            ));
        }
        let all: Vec<String> = sigma.iter().map(|s| sym_text(s)).collect();
        lines.push_str(&format!("  {dead} -[{}]-> {dead}\n", all.join(", ")));
        let src = self.src;
        let name_end = m.name.as_ref().and_then(|n| {
            let after = m.model_span.to;
            src.get(after..)
                .and_then(|rest| rest.find(n.as_str()).map(|i| after + i + n.len()))
        });
        let partial_at = name_end.unwrap_or(m.model_span.to);
        let mut d = e3("E306", m.model_span, msg);
        if simple_states {
            d = d.with_fix(format!("add a dead state `{dead}`"), close, lines);
        } else {
            d = d.with_fix("add a dead state", close, lines);
        }
        d = d.with_fix(
            "mark the machine (partial)",
            Span::new(partial_at, partial_at),
            " (partial)",
        );
        Some(d)
    }

    /// `machine μ { … }`: the canonical record form (01 §5.6).
    pub fn canonical_literal(&mut self, c: &CanonicalLit, _span: Span) -> R<Value> {
        let Some(model) = self.model_named(&c.model) else {
            return Err(Flow::Error(e3(
                "E301",
                c.model_span,
                format!("unknown model `{}`", c.model),
            )));
        };
        let mut states = BTreeSet::new();
        let mut sigma = BTreeSet::new();
        let mut start = None;
        let mut accept = BTreeSet::new();
        let mut edges = Vec::new();
        for (name, span, e) in &c.fields {
            let v = self.eval(e)?;
            match name.as_str() {
                "states" => states.extend(self.items_of(&v, *span)?),
                "alphabet" => {
                    let mut out = Vec::new();
                    self.value_syms(&v, e.span, &mut out)?;
                    sigma.extend(out.into_iter().map(|(s, _)| s));
                }
                "start" => start = Some(v),
                "accept" => accept.extend(self.items_of(&v, *span)?),
                "edges" => {
                    for item in self.items_of(&v, e.span)? {
                        let Value::Tuple(t) = &item else {
                            return Err(Flow::Error(e3(
                                "E309",
                                e.span,
                                "an edge is a tuple (from, label, to)",
                            )));
                        };
                        let (Some(f), Some(l), Some(to)) = (t.first(), t.get(1), t.get(2)) else {
                            return Err(Flow::Error(e3(
                                "E309",
                                e.span,
                                "an edge is a tuple (from, label, to)",
                            )));
                        };
                        let lab = match l {
                            Value::Word(w) if w.is_empty() => Label::eps(),
                            Value::Sym(s) => label::read_label(Some(s.clone())),
                            other => {
                                let mut out = Vec::new();
                                self.value_syms(other, e.span, &mut out)?;
                                label::read_label(out.first().map(|(s, _)| s.clone()))
                            }
                        };
                        edges.push((f.clone(), lab, to.clone()));
                    }
                }
                other => {
                    return Err(Flow::Error(e3(
                        "E309",
                        *span,
                        format!("unknown field `{other}`"),
                    )));
                }
            }
        }
        let Some(start) = start else {
            return Err(Flow::Error(e3(
                "E302",
                c.model_span,
                "the machine has no start state",
            )));
        };
        states.insert(start.clone());
        for (f, _, t) in &edges {
            states.insert(f.clone());
            states.insert(t.clone());
        }
        states.extend(accept.iter().cloned());
        let eff = (*model).clone();
        Ok(Value::Obj(Rc::new(Machine::assemble(Parts {
            model,
            eff,
            name: None,
            states,
            sigma,
            stack_alpha: BTreeSet::new(),
            tape_alpha: BTreeSet::new(),
            out_alpha: BTreeSet::new(),
            edges,
            start,
            accept,
            reject: None,
            blank: label::blank(),
            initial_stack: None,
            lambda: Default::default(),
            provenance: None,
        }))))
    }
}

fn join_syms(s: &BTreeSet<Sym>) -> String {
    s.iter().map(|x| sym_text(x)).collect::<Vec<_>>().join(", ")
}

/// A guard: what a label tests before any effect (SEMANTICS §4.2), per dimension.
fn guard(l: &Label, m: &Model) -> Vec<(String, String)> {
    let mut g = Vec::new();
    for op in &l.ops {
        match op {
            LOp::Read(t, s) => {
                let dim = if m.input == InputMode::OnTape {
                    format!("tape{t}")
                } else {
                    "input".to_string()
                };
                g.push((dim, s.to_string()));
            }
            LOp::Pop(i, s) => g.push((format!("stack{i}"), s.to_string())),
            LOp::Dequeue(s) => g.push(("queue".to_string(), s.to_string())),
            LOp::IfZero(c) => g.push((format!("c{c}"), "0".to_string())),
            LOp::IfPos(c) | LOp::Dec(c) => g.push((format!("c{c}"), "+".to_string())),
            LOp::Regex(_) => g.push(("input".to_string(), "*".to_string())),
            _ => {}
        }
    }
    g
}

fn compatible(a: &[(String, String)], b: &[(String, String)]) -> bool {
    a.iter().all(|(da, va)| {
        b.iter()
            .all(|(db, vb)| da != db || va == vb || va == "*" || vb == "*")
    })
}

fn determinism_error(edges: &[(Value, Label, Value, Span)], m: &Model) -> Option<Diagnostic> {
    for (i, (f1, l1, t1, s1)) in edges.iter().enumerate() {
        for (f2, l2, t2, _) in edges.iter().skip(i + 1) {
            if f1 != f2 || (l1 == l2 && t1 == t2) {
                continue;
            }
            let (g1, g2) = (guard(l1, m), guard(l2, m));
            if compatible(&g1, &g2) {
                return Some(e3(
                    "E305",
                    *s1,
                    format!(
                        "{} is deterministic, but from `{}` both `{}` (to {}) and `{}` (to {}) can apply",
                        m.display(),
                        Machine::display_state(f1),
                        l1.display(m),
                        Machine::display_state(t1),
                        l2.display(m),
                        Machine::display_state(t2)
                    ),
                ));
            }
        }
    }
    None
}
