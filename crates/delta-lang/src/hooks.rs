//! Engine objects in the interpreter: machine members, the library functions on machines and
//! languages (01 §12), language membership and decisions, checks, `run`, and JSON for `show`.

use crate::ast::*;
use crate::builtins::{Args, Named};
use crate::diag::{Diagnostic, Span};
use crate::grammar::{self, Grammar};
use crate::interp::{Flow, Interp, R, err};
use crate::label::{self, LOp, Label};
use crate::lang::{self, Decidability, LangOp, LangVal, Question, Src};
use crate::machine::{Machine, Parts, Provenance};
use crate::model::{self, Missing, Model, ModelVal};
use crate::rx::{self, RegexVal, Rx};
use crate::sim::Verdict;
use crate::value::{Class, Object, Sym, TAG_TRACE, Value, sym_text};
use delta_core::fa::{
    Fa, Step, decide, determinize, minimize, ops, product, regex as core_rx, to_regex,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

/// A run of a machine on a word (`M.run(w)`).
#[derive(Debug)]
pub struct TraceVal {
    pub machine: String,
    pub input: Vec<Sym>,
    pub verdict: Verdict,
    pub steps: Vec<Vec<String>>,
    pub deterministic: bool,
    pub id: u64,
}

impl Object for TraceVal {
    fn tag(&self) -> u8 {
        TAG_TRACE
    }
    fn key(&self) -> String {
        format!("{:020}", self.id)
    }
    fn type_name(&self) -> &'static str {
        "trace"
    }
    fn display(&self) -> String {
        format!(
            "trace of {} on \"{}\": {}",
            self.machine,
            word_text(&self.input),
            self.verdict.name()
        )
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

pub fn word_text(w: &[Sym]) -> String {
    w.iter().map(|s| sym_text(s)).collect()
}

fn atom(s: &str) -> Value {
    Value::Atom(Rc::from(s))
}

fn obj<T: Object + 'static>(x: T) -> Value {
    Value::Obj(Rc::new(x))
}

fn obj_rc<T: Object + 'static>(x: Rc<T>) -> Value {
    Value::Obj(x)
}

fn text_word(s: &str) -> Value {
    Value::word_from(&s.chars().map(|c| c.to_string()).collect::<Vec<_>>())
}

/// The bounded-check length offered by E701 (Q-77).
pub const SUGGESTED_BOUND: usize = 12;

impl Interp<'_> {
    pub fn lookup_extra(&self, name: &str) -> Option<Value> {
        self.models.get(name).map(|m| obj(ModelVal(m.clone())))
    }

    /// Calling a non-function value: instantiating a parameterized model (`mtm(2)`).
    pub fn call_object(&mut self, f: &Value, args: Args, _named: Named, span: Span) -> R<Value> {
        if let Some(m) = f.rc::<ModelVal>() {
            let mut ints = Vec::new();
            for (a, s) in &args {
                match a {
                    Value::Int(n) => ints.push(*n),
                    other => {
                        return err(
                            "E504",
                            *s,
                            format!("a model takes int arguments, found {}", other.display()),
                        );
                    }
                }
            }
            let inst = model::instantiate(&m.0, &ints, span).map_err(Flow::Error)?;
            return Ok(obj(ModelVal(Rc::new(inst))));
        }
        err("E508", span, format!("{} is not a function", f.display()))
    }

    pub fn word_of(&self, v: &Value, span: Span) -> R<Vec<Sym>> {
        match v {
            Value::Word(w) => Ok(w.to_vec()),
            Value::Sym(s) => Ok(vec![s.clone()]),
            other => err(
                "E504",
                span,
                format!(
                    "expected a word, found {} {}",
                    other.type_name(),
                    other.display()
                ),
            ),
        }
    }

    pub fn fuel(&self) -> u64 {
        self.budget.steps
    }

    // ----- languages -----

    pub fn is_langlike(v: &Value) -> bool {
        v.downcast::<Machine>().is_some()
            || v.downcast::<RegexVal>().is_some()
            || v.downcast::<LangVal>().is_some()
            || v.downcast::<Grammar>().is_some()
    }

    pub fn new_lang(
        &mut self,
        class: Class,
        src: Src,
        sigma: BTreeSet<Sym>,
        assumed: bool,
    ) -> Rc<LangVal> {
        let id = self.fresh_id();
        Rc::new(LangVal::new(class, src, sigma, assumed, id))
    }

    /// Coerces machines, regexes, grammars and sets of words to languages (Q-27).
    pub fn to_lang(&mut self, v: &Value, span: Span) -> R<Rc<LangVal>> {
        if let Some(l) = v.rc::<LangVal>() {
            return Ok(l);
        }
        if let Some(m) = v.rc::<Machine>() {
            let class = lang::model_class(&m.eff);
            let sigma = m.sigma.clone();
            return Ok(self.new_lang(class, Src::Machine(m), sigma, false));
        }
        if let Some(r) = v.rc::<RegexVal>() {
            let sigma = r.alphabet.clone();
            return Ok(self.new_lang(Class::Regular, Src::Regex(r), sigma, false));
        }
        if let Some(g) = v.rc::<Grammar>() {
            let sigma = g.terminals.clone();
            return Ok(self.new_lang(Class::Cf, Src::Grammar(g), sigma, false));
        }
        if let Value::Set(items) = v {
            let mut words = BTreeSet::new();
            for i in items.iter() {
                words.insert(self.word_of(i, span)?);
            }
            let sigma = words.iter().flatten().cloned().collect();
            return Ok(self.new_lang(Class::Finite, Src::Finite(words), sigma, false));
        }
        err(
            "E504",
            span,
            format!(
                "expected a language (a machine, regex, grammar or lang), found {} {}",
                v.type_name(),
                v.display()
            ),
        )
    }

    fn lang_op(&mut self, op: LangOp, a: &Value, b: Option<&Value>, span: Span) -> R<Value> {
        let la = self.to_lang(a, span)?;
        let lb = match b {
            Some(b) => Some(self.to_lang(b, span)?),
            None => None,
        };
        let class = lang::op_class(op, la.class, lb.as_ref().map(|l| l.class));
        let mut sigma = la.sigma.clone();
        if let Some(lb) = &lb {
            sigma.extend(lb.sigma.iter().cloned());
        }
        let assumed = la.assumed || lb.as_ref().is_some_and(|l| l.assumed);
        let mut parts = vec![la];
        parts.extend(lb);
        Ok(obj_rc(self.new_lang(
            class,
            Src::Op(op, parts),
            sigma,
            assumed,
        )))
    }

    /// `w ∈ L`: `Some(bool)` when decided, `None` when the budget ran out.
    pub fn lang_member(&mut self, l: &LangVal, w: &[Sym], span: Span) -> R<Option<bool>> {
        self.tick(span)?;
        if w.iter().any(|s| !l.sigma.contains(s)) {
            return Ok(Some(false));
        }
        let compilable = matches!(
            l.src,
            Src::Regex(_) | Src::Op(..) | Src::Assumed { .. } | Src::Finite(_)
        ) || matches!(&l.src, Src::Machine(m) if m.eff.labels_regex);
        if compilable && let Some(f) = self.compile_lang(l, span)? {
            let syms: Vec<Sym> = l.sigma.iter().cloned().collect();
            let ix: Option<Vec<u32>> = w
                .iter()
                .map(|s| syms.binary_search(s).ok().map(|i| i as u32))
                .collect();
            return Ok(Some(ix.is_some_and(|ix| f.accepts(&ix))));
        }
        match &l.src {
            Src::Finite(ws) => Ok(Some(ws.contains(w))),
            Src::Machine(m) => {
                let fuel = self.fuel();
                match m.run(w, fuel, false) {
                    Ok(r) => Ok(match r.verdict {
                        Verdict::Accept => Some(true),
                        Verdict::Reject | Verdict::Loops => Some(false),
                        Verdict::Unknown => None,
                    }),
                    Err(crate::sim::Stuck(msg)) => err("E512", span, msg),
                }
            }
            Src::Regex(_) => Ok(Some(false)),
            Src::Grammar(g) => Ok(g.accepts(w, self.fuel())),
            Src::Predicate {
                pat,
                cond,
                captured,
            } => {
                let mut scope = HashMap::new();
                for (n, v) in captured {
                    scope.insert(
                        n.clone(),
                        crate::interp::Binding {
                            value: v.clone(),
                            mutable: false,
                        },
                    );
                }
                let saved = std::mem::replace(&mut self.scopes, vec![scope]);
                let word = Value::Word(Rc::new(w.to_vec()));
                let r = self
                    .bind_pattern(pat, word, false)
                    .and_then(|_| self.eval_bool(cond));
                self.scopes = saved;
                Ok(Some(r?))
            }
            Src::Assumed { inner, .. } => {
                let inner = inner.clone();
                self.lang_member(&inner, w, span)
            }
            Src::Op(op, parts) => {
                let op = *op;
                let a = match parts.first() {
                    Some(a) => a.clone(),
                    None => return Ok(Some(false)),
                };
                let b = parts.get(1).cloned();
                self.op_member(op, l, &a, b.as_ref(), w, span)
            }
        }
    }

    fn op_member(
        &mut self,
        op: LangOp,
        l: &LangVal,
        a: &Rc<LangVal>,
        b: Option<&Rc<LangVal>>,
        w: &[Sym],
        span: Span,
    ) -> R<Option<bool>> {
        let x = |me: &mut Self, l: &Rc<LangVal>, w: &[Sym]| me.lang_member(l, w, span);
        Ok(match op {
            LangOp::Union => {
                let p = x(self, a, w)?;
                if p == Some(true) {
                    return Ok(Some(true));
                }
                let q = match b {
                    Some(b) => x(self, b, w)?,
                    None => Some(false),
                };
                match (p, q) {
                    (_, Some(true)) => Some(true),
                    (Some(false), Some(false)) => Some(false),
                    _ => None,
                }
            }
            LangOp::Inter | LangOp::Diff => {
                let p = x(self, a, w)?;
                if p == Some(false) {
                    return Ok(Some(false));
                }
                let q = match b {
                    Some(b) => x(self, b, w)?,
                    None => Some(op == LangOp::Inter),
                };
                let q = if op == LangOp::Diff { q.map(|v| !v) } else { q };
                match (p, q) {
                    (_, Some(false)) => Some(false),
                    (Some(true), Some(true)) => Some(true),
                    _ => None,
                }
            }
            LangOp::Compl => {
                if w.iter().any(|s| !l.sigma.contains(s)) {
                    Some(false)
                } else {
                    x(self, a, w)?.map(|v| !v)
                }
            }
            LangOp::Rev => {
                let r: Vec<Sym> = w.iter().rev().cloned().collect();
                x(self, a, &r)?
            }
            LangOp::Concat => {
                let Some(b) = b else { return Ok(Some(false)) };
                let mut unknown = false;
                for i in 0..=w.len() {
                    let (u, v) = w.split_at(i);
                    match x(self, a, u)? {
                        Some(true) => match x(self, b, v)? {
                            Some(true) => return Ok(Some(true)),
                            Some(false) => {}
                            None => unknown = true,
                        },
                        Some(false) => {}
                        None => unknown = true,
                    }
                }
                if unknown { None } else { Some(false) }
            }
            LangOp::Star | LangOp::Plus => {
                let n = w.len();
                if n == 0 {
                    return Ok(if op == LangOp::Star {
                        Some(true)
                    } else {
                        x(self, a, &[])?
                    });
                }
                let mut reach = vec![false; n + 1];
                reach[0] = true;
                let mut unknown = false;
                for j in 1..=n {
                    for i in 0..j {
                        if !reach[i] {
                            continue;
                        }
                        match x(self, a, w.get(i..j).unwrap_or(&[]))? {
                            Some(true) => {
                                reach[j] = true;
                                break;
                            }
                            Some(false) => {}
                            None => unknown = true,
                        }
                    }
                }
                if reach[n] {
                    Some(true)
                } else if unknown {
                    None
                } else {
                    Some(false)
                }
            }
        })
    }

    /// Compiles a language to a finite automaton over its own alphabet, if it has one.
    pub fn compile_lang(&mut self, l: &LangVal, span: Span) -> R<Option<Fa>> {
        if let Some(f) = l.fa.get() {
            return Ok(f.clone());
        }
        let alpha: Vec<Sym> = l.sigma.iter().cloned().collect();
        let f = self.compile_fa(l, &alpha, span)?;
        let _ = l.fa.set(f.clone());
        Ok(f)
    }

    /// `l` as a finite automaton over `alpha` (which contains `l`'s alphabet).
    fn compile_fa(&mut self, l: &LangVal, alpha: &[Sym], span: Span) -> R<Option<Fa>> {
        self.tick(span)?;
        let k = alpha.len();
        let max = self.budget.states as usize;
        let det = |f: &Fa| -> R<Fa> {
            let (d, _, _) = determinize::subset(f, max).map_err(|_| budget_states(span, max))?;
            Ok(d)
        };
        let f = match &l.src {
            Src::Finite(ws) => {
                let mut f = Fa::new(k, 1, 0);
                for w in ws {
                    let mut q = 0;
                    for s in w {
                        let Ok(a) = alpha.binary_search(s) else {
                            continue;
                        };
                        q = match f.next(q, a as u32) {
                            Some(t) => t,
                            None => {
                                let t = f.add_state(false);
                                f.add_edge(q, Some(a as u32), t);
                                t
                            }
                        };
                    }
                    if let Some(x) = f.accept.get_mut(q) {
                        *x = true;
                    }
                }
                Some(f)
            }
            Src::Machine(m) => {
                if m.eff.labels_regex {
                    gnfa_fa(m, alpha)
                } else {
                    m.to_fa(alpha)
                }
            }
            Src::Regex(r) => Some(core_rx::to_nfa_textbook(&r.rx.lower(alpha), k)),
            Src::Grammar(_) | Src::Predicate { .. } => None,
            Src::Op(..) if l.class == Class::Finite && !self.fa_compilable(l) => {
                // A finite language from F ∩ L or F ∖ L: list its words (L is decidable).
                let words = self.finite_words(l, span)?;
                let fin = self.new_lang(
                    Class::Finite,
                    Src::Finite(words),
                    l.sigma.clone(),
                    l.assumed,
                );
                self.compile_fa(&fin, alpha, span)?
            }
            Src::Assumed { evidence, .. } => {
                let e = evidence.clone();
                self.compile_fa(&e, alpha, span)?
            }
            Src::Op(op, parts) => {
                let op = *op;
                let parts = parts.clone();
                let mut fas = Vec::new();
                for p in &parts {
                    match self.compile_fa(p, alpha, span)? {
                        Some(f) => fas.push(f),
                        None => return Ok(None),
                    }
                }
                let a = fas.first().cloned();
                let b = fas.get(1).cloned();
                match (op, a, b) {
                    (LangOp::Union, Some(a), Some(b)) => Some(ops::union(&a, &b)),
                    (LangOp::Concat, Some(a), Some(b)) => Some(ops::concat(&a, &b)),
                    (LangOp::Star, Some(a), _) => Some(ops::star(&a)),
                    (LangOp::Plus, Some(a), _) => Some(ops::concat(&a, &ops::star(&a))),
                    (LangOp::Rev, Some(a), _) => Some(ops::reverse(&a)),
                    (LangOp::Inter | LangOp::Diff, Some(a), Some(b)) => {
                        let (da, _) = ops::complete(&det(&a)?);
                        let (db, _) = ops::complete(&det(&b)?);
                        let keep: &dyn Fn(bool, bool) -> bool = if op == LangOp::Inter {
                            &|x, y| x && y
                        } else {
                            &|x, y| x && !y
                        };
                        let (p, _) = product::product(&da, &db, keep, max)
                            .map_err(|_| budget_states(span, max))?;
                        Some(p)
                    }
                    (LangOp::Compl, Some(a), _) => {
                        let (d, _) = ops::complete(&det(&a)?);
                        let c = ops::complement_dfa(&d);
                        // Complement relative to this language's alphabet (Q-71).
                        let mut own = Fa::new(k, 1, 0);
                        own.accept[0] = true;
                        for s in &l.sigma {
                            if let Ok(i) = alpha.binary_search(s) {
                                own.add_edge(0, Some(i as u32), 0);
                            }
                        }
                        let (own, _) = ops::complete(&own);
                        let (p, _) = product::product(&c, &own, &|x, y| x && y, max)
                            .map_err(|_| budget_states(span, max))?;
                        Some(p)
                    }
                    _ => None,
                }
            }
        };
        Ok(f)
    }

    fn fa_compilable(&self, l: &LangVal) -> bool {
        match &l.src {
            Src::Finite(_) | Src::Regex(_) => true,
            Src::Machine(m) => m.is_fa() || m.eff.labels_regex,
            Src::Grammar(_) | Src::Predicate { .. } => false,
            Src::Assumed { evidence, .. } => self.fa_compilable(evidence),
            Src::Op(_, parts) => parts.iter().all(|p| self.fa_compilable(p)),
        }
    }

    /// The words of a language whose class is Finite.
    fn finite_words(&mut self, l: &LangVal, span: Span) -> R<BTreeSet<Vec<Sym>>> {
        match &l.src {
            Src::Finite(ws) => Ok(ws.clone()),
            Src::Op(op, parts) => {
                let op = *op;
                let (Some(a), b) = (parts.first().cloned(), parts.get(1).cloned()) else {
                    return Ok(BTreeSet::new());
                };
                let (fin, other) = if a.class == Class::Finite {
                    (a, b)
                } else {
                    (b.unwrap_or_else(|| a.clone()), Some(a))
                };
                let base = self.finite_words(&fin, span)?;
                let mut out = BTreeSet::new();
                for w in base {
                    let keep = match (op, &other) {
                        (LangOp::Inter, Some(o)) => self.lang_member(o, &w, span)? == Some(true),
                        (LangOp::Diff, Some(o)) => self.lang_member(o, &w, span)? == Some(false),
                        _ => self.lang_member(l, &w, span)? == Some(true),
                    };
                    if keep {
                        out.insert(w);
                    }
                }
                Ok(out)
            }
            Src::Assumed { inner, .. } => {
                let i = inner.clone();
                self.finite_words(&i, span)
            }
            _ => err(
                "E702",
                span,
                "listing the words of this language is not implemented",
            ),
        }
    }

    /// Answers a question about languages, with a witness word where it has one.
    pub fn decide(
        &mut self,
        q: Question,
        a: &LangVal,
        b: Option<&LangVal>,
        span: Span,
    ) -> R<(bool, Option<Vec<Sym>>)> {
        let class = b.map(|b| a.class.max(b.class)).unwrap_or(a.class);
        match lang::decidable(q, class) {
            Decidability::No => {
                return Err(Flow::Error(undecidable(
                    q,
                    a.class,
                    b.map(|b| b.class),
                    span,
                    None,
                )));
            }
            Decidability::NotImplemented => {
                return Err(Flow::Error(not_implemented(q, class, span)));
            }
            Decidability::Yes => {}
        }
        let mut sigma = a.sigma.clone();
        if let Some(b) = b {
            sigma.extend(b.sigma.iter().cloned());
        }
        let alpha: Vec<Sym> = sigma.iter().cloned().collect();
        let max = self.budget.states as usize;
        let fa_a = self.compile_fa(a, &alpha, span)?;
        let fa_b = match b {
            Some(b) => self.compile_fa(b, &alpha, span)?,
            None => None,
        };
        let to_word = |w: Vec<u32>| -> Vec<Sym> {
            w.iter()
                .filter_map(|i| alpha.get(*i as usize).cloned())
                .collect()
        };
        let exhausted = |_| budget_states(span, max);
        if let (Some(fa), None) = (&fa_a, b) {
            return match q {
                Question::IsEmpty => Ok((decide::is_empty(fa), None)),
                Question::IsFinite => Ok((decide::is_finite(fa, max).map_err(exhausted)?, None)),
                _ => {
                    let w = decide::universality(fa, max).map_err(exhausted)?;
                    Ok((w.is_none(), w.map(to_word)))
                }
            };
        }
        if let (Some(x), Some(y)) = (&fa_a, &fa_b) {
            let w = match q {
                Question::Equiv => decide::equivalence(x, y, max).map_err(exhausted)?,
                _ => decide::inclusion(x, y, max).map_err(exhausted)?,
            };
            return Ok((w.is_none(), w.map(to_word)));
        }
        if b.is_none() {
            if let Src::Grammar(g) = &a.src {
                match q {
                    Question::IsEmpty => return Ok((g.is_empty(), None)),
                    Question::IsFinite => return Ok((g.is_finite(), None)),
                    _ => {}
                }
            }
            // Pushdown machines: a word found settles emptiness and universality.
            if let Some(answer) = self.search_word(a, q, span)? {
                return Ok(answer);
            }
        }
        Err(Flow::Error(not_implemented(q, class, span)))
    }

    /// Looks for a word that settles the question (a member for emptiness, a non-member for
    /// universality), up to length 10.
    fn search_word(
        &mut self,
        a: &LangVal,
        q: Question,
        span: Span,
    ) -> R<Option<(bool, Option<Vec<Sym>>)>> {
        if !matches!(q, Question::IsEmpty | Question::IsUniversal) {
            return Ok(None);
        }
        let syms: Vec<Sym> = a.sigma.iter().cloned().collect();
        let mut layer: Vec<Vec<Sym>> = vec![Vec::new()];
        for _ in 0..=10 {
            for w in &layer {
                let m = self.lang_member(a, w, span)?;
                match (q, m) {
                    (Question::IsEmpty, Some(true)) | (Question::IsUniversal, Some(false)) => {
                        return Ok(Some((false, Some(w.clone()))));
                    }
                    _ => {}
                }
            }
            let mut next = Vec::new();
            for w in &layer {
                for s in &syms {
                    let mut w2 = w.clone();
                    w2.push(s.clone());
                    next.push(w2);
                }
            }
            if next.len() > 4096 {
                return Ok(None);
            }
            layer = next;
        }
        Ok(None)
    }

    /// Bounded comparison: every word over the union alphabet up to length `n`.
    pub fn bounded(
        &mut self,
        q: Question,
        a: &LangVal,
        b: &LangVal,
        n: usize,
        span: Span,
    ) -> R<(bool, Option<Vec<Sym>>)> {
        let mut sigma = a.sigma.clone();
        sigma.extend(b.sigma.iter().cloned());
        let syms: Vec<Sym> = sigma.into_iter().collect();
        let mut layer: Vec<Vec<Sym>> = vec![Vec::new()];
        for len in 0..=n {
            for w in &layer {
                let x = self.lang_member(a, w, span)?;
                let y = self.lang_member(b, w, span)?;
                let (Some(x), Some(y)) = (x, y) else {
                    return Err(Flow::Budget(Diagnostic::warning(
                        "B605",
                        span,
                        format!(
                            "membership of \"{}\" was not decided within the budget",
                            word_text(w)
                        ),
                    )));
                };
                let bad = match q {
                    Question::Subset => x && !y,
                    _ => x != y,
                };
                if bad {
                    return Ok((false, Some(w.clone())));
                }
            }
            if len == n {
                break;
            }
            let mut next = Vec::with_capacity(layer.len() * syms.len().max(1));
            for w in &layer {
                for s in &syms {
                    self.tick(span)?;
                    let mut w2 = w.clone();
                    w2.push(s.clone());
                    next.push(w2);
                }
            }
            layer = next;
        }
        Ok((true, None))
    }

    // ----- operators on engine objects -----

    pub fn items_of_extra(&mut self, v: &Value, span: Span) -> R<Vec<Value>> {
        err(
            "E504",
            span,
            format!("can't iterate over {} {}", v.type_name(), v.display()),
        )
    }

    pub fn binary_extra(&mut self, op: BinOp, l: Value, r: Value, span: Span) -> R<Value> {
        let langs = Self::is_langlike(&l) || Self::is_langlike(&r);
        if langs {
            match op {
                BinOp::Union => return self.lang_op(LangOp::Union, &l, Some(&r), span),
                BinOp::Inter => return self.lang_op(LangOp::Inter, &l, Some(&r), span),
                BinOp::Diff => return self.lang_op(LangOp::Diff, &l, Some(&r), span),
                BinOp::Concat => return self.lang_op(LangOp::Concat, &l, Some(&r), span),
                _ => {}
            }
        }
        if (langs || matches!(op, BinOp::Equiv))
            && let BinOp::Equiv | BinOp::Subset = op
        {
            let a = self.to_lang(&l, span)?;
            let b = self.to_lang(&r, span)?;
            let q = if op == BinOp::Equiv {
                Question::Equiv
            } else {
                Question::Subset
            };
            let (ok, _) = self.decide(q, &a, Some(&b), span)?;
            return Ok(Value::Bool(ok));
        }
        err(
            "E504",
            span,
            format!(
                "`{}` doesn't apply to {} and {} ({} and {})",
                op.text(),
                l.type_name(),
                r.type_name(),
                l.display(),
                r.display()
            ),
        )
    }

    pub fn member_of_extra(&mut self, x: &Value, coll: &Value, span: Span) -> R<bool> {
        if Self::is_langlike(coll) {
            let w = self.word_of(x, span)?;
            let l = self.to_lang(coll, span)?;
            return match self.lang_member(&l, &w, span)? {
                Some(b) => Ok(b),
                None => Err(Flow::Budget(Diagnostic::warning(
                    "B605",
                    span,
                    format!(
                        "membership of \"{}\" was not decided within the budget (the verdict is unknown)",
                        word_text(&w)
                    ),
                ))),
            };
        }
        err(
            "E504",
            span,
            format!(
                "can't test membership of {} in {} {}",
                x.display(),
                coll.type_name(),
                coll.display()
            ),
        )
    }

    pub fn power_extra(&mut self, v: Value, _n: i64, _upto: bool, span: Span) -> R<Value> {
        err(
            "E504",
            span,
            format!("`^` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn rev_extra(&mut self, v: Value, span: Span) -> R<Value> {
        if Self::is_langlike(&v) || matches!(v, Value::Set(_)) {
            return self.lang_op(LangOp::Rev, &v, None, span);
        }
        err(
            "E504",
            span,
            format!("`rev` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn complement(&mut self, v: Value, span: Span) -> R<Value> {
        if Self::is_langlike(&v) {
            return self.lang_op(LangOp::Compl, &v, None, span);
        }
        err(
            "E504",
            span,
            format!("`∁` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn star(&mut self, v: Value, span: Span, plus: bool) -> R<Value> {
        if Self::is_langlike(&v) {
            return self.lang_op(
                if plus { LangOp::Plus } else { LangOp::Star },
                &v,
                None,
                span,
            );
        }
        match v {
            Value::Set(_) => err(
                "E205",
                span,
                "this set is infinite (Σ* has every word); it's allowed only inside `lang`",
            ),
            other => err(
                "E504",
                span,
                format!(
                    "`*` doesn't apply to {} {}",
                    other.type_name(),
                    other.display()
                ),
            ),
        }
    }

    pub fn values_equal(&mut self, l: &Value, r: &Value, _span: Span) -> R<bool> {
        Ok(l == r)
    }

    // ----- members -----

    pub fn member_extra(&mut self, v: Value, name: &str, span: Span) -> R<Value> {
        if let Some(m) = v.rc::<Machine>() {
            return self.machine_member(&m, name, span);
        }
        if let Some(m) = v.downcast::<ModelVal>() {
            return match name {
                "name" => Ok(text_word(&m.0.name)),
                _ => err("E505", span, format!("a model has no member `{name}`")),
            };
        }
        if let Some(t) = v.downcast::<TraceVal>() {
            return match name {
                "verdict" => Ok(atom(t.verdict.name())),
                "accepted" => Ok(Value::Bool(t.verdict == Verdict::Accept)),
                "input" => Ok(Value::Word(Rc::new(t.input.clone()))),
                "steps" => Ok(Value::list(
                    t.steps
                        .iter()
                        .map(|layer| Value::list(layer.iter().map(|c| text_word(c)).collect()))
                        .collect(),
                )),
                _ => err("E505", span, format!("a trace has no member `{name}`")),
            };
        }
        if let Some(g) = v.downcast::<Grammar>() {
            let var = |i: usize| Value::sym(g.vars.get(i).map(|s| s.as_str()).unwrap_or(""));
            let gsym = |x: &grammar::GSym| match x {
                grammar::GSym::V(i) => var(*i),
                grammar::GSym::T(t) => Value::Sym(t.clone()),
            };
            match name {
                "rules" => {
                    return Ok(Value::set(
                        g.rules
                            .iter()
                            .map(|(l, rhs)| {
                                Value::tuple(vec![
                                    var(*l),
                                    Value::list(rhs.iter().map(gsym).collect()),
                                ])
                            })
                            .collect(),
                    ));
                }
                "variables" => return Ok(Value::set((0..g.vars.len()).map(var).collect())),
                "terminals" => {
                    return Ok(Value::set(
                        g.terminals.iter().map(|t| Value::Sym(t.clone())).collect(),
                    ));
                }
                "start" => return Ok(var(g.start)),
                _ => {}
            }
        }
        if Self::is_langlike(&v) {
            let l = self.to_lang(&v, span)?;
            return match name {
                "class" => Ok(Value::Class(l.class)),
                "alphabet" => Ok(Value::set(
                    l.sigma.iter().map(|s| Value::Sym(s.clone())).collect(),
                )),
                "assumed" => Ok(Value::Bool(l.assumed)),
                _ => err("E505", span, format!("a language has no member `{name}`")),
            };
        }
        err(
            "E505",
            span,
            format!("{} has no member `{name}`", v.type_name()),
        )
    }

    fn machine_member(&mut self, m: &Rc<Machine>, name: &str, span: Span) -> R<Value> {
        match name {
            "model" => Ok(obj(ModelVal(m.model.clone()))),
            "states" => Ok(Value::set(m.states.iter().cloned().collect())),
            "alphabet" => Ok(Value::set(
                m.sigma.iter().map(|s| Value::Sym(s.clone())).collect(),
            )),
            "start" => Ok(m.state(m.start)),
            "accept" => Ok(Value::set(m.accept.iter().map(|q| m.state(*q)).collect())),
            "reject" => Ok(m.reject.map(|q| m.state(q)).unwrap_or(Value::None)),
            "size" => Ok(Value::Int(m.states.len() as i64)),
            "edges" => Ok(Value::list(
                m.edges
                    .iter()
                    .map(|e| {
                        let mut r = BTreeMap::new();
                        r.insert("from".to_string(), m.state(e.from));
                        r.insert("label".to_string(), m.label_value(&e.label));
                        r.insert("to".to_string(), m.state(e.to));
                        Value::Record(Rc::new(r))
                    })
                    .collect(),
            )),
            "lang" => {
                let l = self.to_lang(&obj_rc(m.clone()), span)?;
                Ok(obj_rc(l))
            }
            "stack" => Ok(Value::set(
                m.stack_alpha
                    .iter()
                    .map(|s| Value::Sym(s.clone()))
                    .collect(),
            )),
            "tape" => Ok(Value::set(
                m.tape_alpha.iter().map(|s| Value::Sym(s.clone())).collect(),
            )),
            "blank" => Ok(Value::Sym(m.blank.clone())),
            "class" => Ok(Value::Class(lang::model_class(&m.eff))),
            _ => err(
                "E505",
                span,
                format!(
                    "a machine has no member `{name}` (see M.states, M.alphabet, M.start, M.accept, M.edges, M.δ, …)"
                ),
            ),
        }
    }

    pub fn call_method(
        &mut self,
        recv: Value,
        name: &str,
        args: Args,
        named: Named,
        span: Span,
    ) -> R<Value> {
        if let Value::Record(r) = &recv
            && let Some(f) = r.get(name).cloned()
        {
            return self.call_value(&f, args, named, span);
        }
        if let Some(m) = recv.rc::<Machine>() {
            return self.machine_method(&m, name, args, named, span);
        }
        if Self::is_langlike(&recv) {
            let l = self.to_lang(&recv, span)?;
            let q = match name {
                "is_empty" => Some(Question::IsEmpty),
                "is_finite" => Some(Question::IsFinite),
                "is_universal" => Some(Question::IsUniversal),
                _ => None,
            };
            if let Some(q) = q {
                let (b, _) = self.decide(q, &l, None, span)?;
                return Ok(Value::Bool(b));
            }
            if name == "accepts" {
                let w = self.word_of(
                    &args.first().map(|a| a.0.clone()).unwrap_or(Value::None),
                    span,
                )?;
                return match self.lang_member(&l, &w, span)? {
                    Some(b) => Ok(Value::Bool(b)),
                    None => Err(Flow::Budget(Diagnostic::warning(
                        "B605",
                        span,
                        "membership was not decided within the budget",
                    ))),
                };
            }
        }
        let f = self.member_extra(recv, name, span)?;
        self.call_value(&f, args, named, span)
    }

    fn machine_method(
        &mut self,
        m: &Rc<Machine>,
        name: &str,
        args: Args,
        named: Named,
        span: Span,
    ) -> R<Value> {
        let arg = |i: usize| args.get(i).map(|(v, _)| v.clone()).unwrap_or(Value::None);
        let aspan = |i: usize| args.get(i).map(|(_, s)| *s).unwrap_or(span);
        match name {
            "δ" => {
                let q = arg(0);
                let Some(qi) = m.index_of(&q) else {
                    return err(
                        "E503",
                        aspan(0),
                        format!("{} is not a state of this machine", q.display()),
                    );
                };
                let sym: Option<Sym> = match arg(1) {
                    Value::Word(w) if w.is_empty() => None,
                    Value::Sym(s) => Some(s),
                    Value::Word(w) if w.len() == 1 => w.first().cloned(),
                    other => {
                        return err(
                            "E504",
                            aspan(1),
                            format!("δ(q, a) needs a symbol, found {}", other.display()),
                        );
                    }
                };
                let targets: Vec<usize> = m
                    .out_edges(qi)
                    .iter()
                    .filter_map(|ei| m.edges.get(*ei))
                    .filter(|e| match &sym {
                        Some(s) => e.label.read() == Some(s),
                        None => e.label.ops.is_empty(),
                    })
                    .map(|e| e.to)
                    .collect();
                let det = m.eff.deterministic
                    && m.eff.storage.is_empty()
                    && m.eff.input == crate::model::InputMode::OneWay;
                if det && sym.is_some() {
                    return Ok(match targets.first() {
                        Some(t) => m.state(*t),
                        None if m.eff.missing == Missing::Dead => atom("⊥"),
                        None => Value::None,
                    });
                }
                Ok(Value::set(
                    targets.into_iter().map(|t| m.state(t)).collect(),
                ))
            }
            "eclose" => {
                let items = self.items_of(&arg(0), aspan(0))?;
                let mut set: Vec<usize> = items.iter().filter_map(|v| m.index_of(v)).collect();
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
                Ok(Value::set(set.into_iter().map(|q| m.state(q)).collect()))
            }
            "accepts" => {
                let w = self.word_of(&arg(0), aspan(0))?;
                if w.iter().any(|s| !m.sigma.contains(s)) {
                    return Ok(Value::Bool(false));
                }
                let fuel = self.fuel();
                match m.run(&w, fuel, false) {
                    Ok(r) => match r.verdict {
                        Verdict::Accept => Ok(Value::Bool(true)),
                        Verdict::Reject | Verdict::Loops => Ok(Value::Bool(false)),
                        Verdict::Unknown => Err(Flow::Budget(Diagnostic::warning(
                            "B605",
                            span,
                            format!(
                                "the run on \"{}\" didn't finish within the budget (the verdict is unknown)",
                                word_text(&w)
                            ),
                        ))),
                    },
                    Err(crate::sim::Stuck(msg)) => err("E512", span, msg),
                }
            }
            "run" => {
                let w = self.word_of(&arg(0), aspan(0))?;
                self.trace_value(m, &w, span)
            }
            "output" => {
                let w = self.word_of(&arg(0), aspan(0))?;
                let fuel = self.fuel();
                match m.run(&w, fuel, true) {
                    Ok(r) => Ok(r
                        .output
                        .map(|o| Value::Word(Rc::new(o)))
                        .unwrap_or(Value::None)),
                    Err(crate::sim::Stuck(msg)) => err("E512", span, msg),
                }
            }
            "rename" => {
                let f = arg(0);
                let mut map: BTreeMap<Value, Value> = BTreeMap::new();
                for v in &m.states {
                    let nv = self.call_value(&f, vec![(v.clone(), span)], Vec::new(), span)?;
                    if map.values().any(|x| *x == nv) {
                        return err(
                            "E513",
                            span,
                            format!("rename maps two states to {}", nv.display()),
                        );
                    }
                    map.insert(v.clone(), nv);
                }
                let mut p = m.parts();
                let get = |v: &Value| map.get(v).cloned().unwrap_or(Value::None);
                p.states = p.states.iter().map(get).collect();
                p.edges = p
                    .edges
                    .iter()
                    .map(|(a, l, b)| (get(a), l.clone(), get(b)))
                    .collect();
                p.start = get(&p.start);
                p.accept = p.accept.iter().map(get).collect();
                p.reject = p.reject.as_ref().map(get);
                Ok(obj(Machine::assemble(p)))
            }
            "with" => {
                let mut p = m.parts();
                for (k, v, s) in named {
                    match k.as_str() {
                        "accept" => p.accept = self.items_of(&v, s)?.into_iter().collect(),
                        "start" => p.start = v,
                        "states" => p.states = self.items_of(&v, s)?.into_iter().collect(),
                        "alphabet" => {
                            let mut out = Vec::new();
                            self.value_syms(&v, s, &mut out)?;
                            p.sigma = out.into_iter().map(|(x, _)| x).collect();
                        }
                        other => {
                            return err(
                                "E507",
                                s,
                                format!(
                                    "M.with takes accept:, start:, states: or alphabet:, not `{other}`"
                                ),
                            );
                        }
                    }
                }
                p.states.insert(p.start.clone());
                p.states.extend(p.accept.iter().cloned());
                Ok(obj(Machine::assemble(p)))
            }
            _ => {
                let v = self.machine_member(m, name, span)?;
                self.call_value(&v, args, named, span)
            }
        }
    }

    fn trace_value(&mut self, m: &Rc<Machine>, w: &[Sym], span: Span) -> R<Value> {
        let fuel = self.fuel();
        let r = m
            .run(w, fuel, true)
            .map_err(|crate::sim::Stuck(msg)| Flow::Error(Diagnostic::error("E512", span, msg)))?;
        let id = self.fresh_id();
        Ok(obj(TraceVal {
            machine: m.name.clone().unwrap_or_else(|| m.model.display()),
            input: w.to_vec(),
            verdict: r.verdict,
            steps: r.steps,
            deterministic: m.eff.deterministic,
            id,
        }))
    }

    // ----- library functions (01 §12) -----

    fn method_of(named: &Named) -> Option<String> {
        named
            .iter()
            .find(|(n, _, _)| n == "method")
            .and_then(|(_, v, _)| match v {
                Value::Word(w) => Some(word_text(w)),
                _ => None,
            })
    }

    fn arg_machine(&mut self, args: &Args, i: usize, span: Span) -> R<Rc<Machine>> {
        let (v, s) = args.get(i).cloned().unwrap_or((Value::None, span));
        if let Some(m) = v.rc::<Machine>() {
            return Ok(m);
        }
        if let Some(r) = v.rc::<RegexVal>() {
            return self.regex_to_nfa(&r, None, s);
        }
        err(
            "E504",
            s,
            format!(
                "expected a machine, found {} {}",
                v.type_name(),
                v.display()
            ),
        )
    }

    fn regex_to_nfa(&mut self, r: &RegexVal, method: Option<&str>, span: Span) -> R<Rc<Machine>> {
        let alpha = r.alpha_vec();
        let re = r.rx.lower(&alpha);
        let (f, id) = match method.unwrap_or("textbook") {
            "textbook" => (
                core_rx::to_nfa_textbook(&re, alpha.len()),
                core_rx::TEXTBOOK,
            ),
            "thompson" => (
                core_rx::to_nfa_thompson(&re, alpha.len()),
                core_rx::THOMPSON,
            ),
            "glushkov" => (
                core_rx::to_nfa_glushkov(&re, alpha.len()),
                core_rx::GLUSHKOV,
            ),
            other => {
                return err(
                    "E507",
                    span,
                    format!(
                        "unknown method \"{other}\": use \"textbook\", \"thompson\" or \"glushkov\""
                    ),
                );
            }
        };
        let values: Vec<Value> = (0..f.n()).map(|i| Value::Int(i as i64)).collect();
        let prov = Provenance {
            algorithm: id.to_string(),
            steps: Vec::new(),
        };
        Ok(Rc::new(self.fa_machine(
            &f,
            &alpha,
            &values,
            "nfa",
            Some(prov),
        )))
    }

    /// A machine of the prelude model `model` from an index automaton.
    pub fn fa_machine(
        &self,
        f: &Fa,
        alpha: &[Sym],
        values: &[Value],
        model: &str,
        prov: Option<Provenance>,
    ) -> Machine {
        let m = self
            .models
            .get(model)
            .cloned()
            .unwrap_or_else(|| Rc::new(Model::blank(model)));
        let v = |i: usize| values.get(i).cloned().unwrap_or(Value::Int(i as i64));
        let mut edges = Vec::new();
        for (p, list) in f.edges.iter().enumerate() {
            for (a, q) in list {
                let l = label::read_label(a.and_then(|a| alpha.get(a as usize).cloned()));
                edges.push((v(p), l, v(*q)));
            }
        }
        let mut eff = (*m).clone();
        if model == "dfa" && !f.is_dfa() {
            eff.total = false;
            eff.missing = Missing::Dead;
        }
        Machine::assemble(Parts {
            model: m,
            eff,
            name: None,
            states: (0..f.n()).map(v).collect(),
            sigma: alpha.iter().cloned().collect(),
            stack_alpha: BTreeSet::new(),
            tape_alpha: BTreeSet::new(),
            out_alpha: BTreeSet::new(),
            edges,
            start: v(f.start),
            accept: (0..f.n()).filter(|q| f.is_accept(*q)).map(v).collect(),
            reject: None,
            blank: label::blank(),
            initial_stack: None,
            provenance: prov,
        })
    }

    /// A finite automaton machine as an index automaton over its alphabet.
    fn machine_fa(&self, m: &Machine, span: Span) -> R<(Fa, Vec<Sym>)> {
        let alpha: Vec<Sym> = m.sigma.iter().cloned().collect();
        let f = if m.eff.labels_regex {
            gnfa_fa(m, &alpha)
        } else {
            m.to_fa(&alpha)
        };
        match f {
            Some(f) => Ok((f, alpha)),
            None => err(
                "E504",
                span,
                format!(
                    "this needs a finite automaton (dfa or nfa), not a {}",
                    m.model.display()
                ),
            ),
        }
    }

    fn determinize_machine(&mut self, m: &Machine, span: Span) -> R<Machine> {
        let (f, alpha) = self.machine_fa(m, span)?;
        let max = self.budget.states as usize;
        let (d, sets, steps) =
            determinize::subset(&f, max).map_err(|_| budget_states(span, max))?;
        let values: Vec<Value> = sets
            .iter()
            .map(|s| Value::set(s.iter().map(|q| m.state(*q)).collect()))
            .collect();
        let prov = Provenance {
            algorithm: determinize::ID.to_string(),
            steps,
        };
        Ok(self.fa_machine(&d, &alpha, &values, "dfa", Some(prov)))
    }

    /// A deterministic, total version of `m` (determinize if needed, complete with `:dead`).
    fn dfa_of(&mut self, m: &Machine, span: Span) -> R<(Fa, Vec<Sym>, Vec<Value>)> {
        let (f, alpha) = self.machine_fa(m, span)?;
        if f.is_dfa() {
            return Ok((f, alpha, m.states.clone()));
        }
        let deterministic = !f.has_epsilon()
            && f.edges.iter().all(|l| {
                let mut s: Vec<Option<u32>> = l.iter().map(|(a, _)| *a).collect();
                s.sort();
                s.windows(2).all(|w| w.first() != w.get(1))
            });
        let (base, values) = if deterministic {
            (f, m.states.clone())
        } else {
            let d = self.determinize_machine(m, span)?;
            let (f2, _) = self.machine_fa(&d, span)?;
            (f2, d.states.clone())
        };
        let (c, dead) = ops::complete(&base);
        let mut values = values;
        if dead.is_some() {
            let fresh = fresh_atom("dead", &values);
            values.push(fresh);
        }
        Ok((c, alpha, values))
    }

    pub fn call_builtin_extra(
        &mut self,
        name: &str,
        args: &Args,
        named: &Named,
        span: Span,
    ) -> R<Option<Value>> {
        let method = Self::method_of(named);
        let max = self.budget.states as usize;
        let v = match name {
            "to_nfa" => {
                let (x, s) = args.first().cloned().unwrap_or((Value::None, span));
                let Some(r) = x.rc::<RegexVal>() else {
                    return err(
                        "E504",
                        s,
                        format!("to_nfa needs a regex, found {}", x.display()),
                    );
                };
                obj_rc(self.regex_to_nfa(&r, method.as_deref(), span)?)
            }
            "determinize" => {
                let m = self.arg_machine(args, 0, span)?;
                obj(self.determinize_machine(&m, span)?)
            }
            "minimize" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha, values) = self.dfa_of(&m, span)?;
                let which = method.clone().unwrap_or_else(|| "hopcroft".to_string());
                let (mf, blocks, steps) = match which.as_str() {
                    "hopcroft" => minimize::hopcroft(&f),
                    "moore" => minimize::moore(&f),
                    "table" => minimize::table(&f),
                    "brzozowski" => {
                        minimize::brzozowski(&f, max).map_err(|_| budget_states(span, max))?
                    }
                    other => {
                        return err(
                            "E507",
                            span,
                            format!(
                                "unknown method \"{other}\": use \"hopcroft\", \"moore\", \"brzozowski\" or \"table\""
                            ),
                        );
                    }
                };
                let id = match which.as_str() {
                    "moore" => minimize::MOORE,
                    "table" => minimize::TABLE,
                    "brzozowski" => minimize::BRZOZOWSKI,
                    _ => minimize::HOPCROFT,
                };
                let vals: Vec<Value> = blocks
                    .iter()
                    .map(|b| match b.as_slice() {
                        [one] => values.get(*one).cloned().unwrap_or(Value::None),
                        many => Value::set(
                            many.iter()
                                .filter_map(|q| values.get(*q).cloned())
                                .collect(),
                        ),
                    })
                    .collect();
                let prov = Provenance {
                    algorithm: id.to_string(),
                    steps,
                };
                obj(self.fa_machine(&mf, &alpha, &vals, "dfa", Some(prov)))
            }
            "complete" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                let (c, dead) = ops::complete(&f);
                let mut values = m.states.clone();
                if dead.is_some() {
                    values.push(fresh_atom("dead", &m.states));
                }
                let model = if c.is_dfa() { "dfa" } else { "nfa" };
                obj(self.fa_machine(&c, &alpha, &values, model, None))
            }
            "trim" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                let (t, old) = ops::trim(&f);
                let values: Vec<Value> = old.iter().map(|q| m.state(*q)).collect();
                let model = if m.model.name == "dfa" { "dfa" } else { "nfa" };
                obj(self.fa_machine(&t, &alpha, &values, model, None))
            }
            "remove_epsilon" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                obj(self.fa_machine(&ops::remove_epsilon(&f), &alpha, &m.states, "nfa", None))
            }
            "reverse" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                let mut values = m.states.clone();
                values.push(fresh_atom("start", &m.states));
                obj(self.fa_machine(&ops::reverse(&f), &alpha, &values, "nfa", None))
            }
            "complement" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha, values) = self.dfa_of(&m, span)?;
                obj(self.fa_machine(&ops::complement_dfa(&f), &alpha, &values, "dfa", None))
            }
            "union" | "intersect" | "difference" | "product" if args.len() >= 2 => {
                let a = self.arg_machine(args, 0, span)?;
                let b = self.arg_machine(args, 1, span)?;
                let mut sigma: BTreeSet<Sym> = a.sigma.clone();
                sigma.extend(b.sigma.iter().cloned());
                let alpha: Vec<Sym> = sigma.into_iter().collect();
                let ma = widen(&a, &alpha);
                let mb = widen(&b, &alpha);
                let (fa_, _, va) = self.dfa_of(&ma, span)?;
                let (fb, _, vb) = self.dfa_of(&mb, span)?;
                let keep: &dyn Fn(bool, bool) -> bool = match name {
                    "union" => &|x, y| x || y,
                    "intersect" => &|x, y| x && y,
                    "difference" => &|x, y| x && !y,
                    _ => &|_, _| false,
                };
                let (mut p, pairs) =
                    product::product(&fa_, &fb, keep, max).map_err(|_| budget_states(span, max))?;
                if name == "product" {
                    let keepfn = args.get(2).map(|(v, _)| v.clone()).unwrap_or(Value::None);
                    for (i, (x, y)) in pairs.iter().enumerate() {
                        let r = self.call_value(
                            &keepfn,
                            vec![
                                (Value::Bool(fa_.is_accept(*x)), span),
                                (Value::Bool(fb.is_accept(*y)), span),
                            ],
                            Vec::new(),
                            span,
                        )?;
                        let t = self.truth(&r, span)?;
                        if let Some(acc) = p.accept.get_mut(i) {
                            *acc = t;
                        }
                    }
                }
                let values: Vec<Value> = pairs
                    .iter()
                    .map(|(x, y)| {
                        Value::tuple(vec![
                            va.get(*x).cloned().unwrap_or(Value::None),
                            vb.get(*y).cloned().unwrap_or(Value::None),
                        ])
                    })
                    .collect();
                let prov = Provenance {
                    algorithm: product::ID.to_string(),
                    steps: Vec::new(),
                };
                obj(self.fa_machine(&p, &alpha, &values, "dfa", Some(prov)))
            }
            "star"
                if args.len() == 1
                    && args
                        .first()
                        .is_some_and(|(v, _)| v.downcast::<Machine>().is_some()) =>
            {
                let a = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&a, span)?;
                let mut values = vec![fresh_atom("start", &a.states)];
                values.extend(a.states.iter().cloned());
                obj(self.fa_machine(&ops::star(&f), &alpha, &values, "nfa", None))
            }
            "concat" if args.len() == 2 => {
                let a = self.arg_machine(args, 0, span)?;
                let b = self.arg_machine(args, 1, span)?;
                let mut sigma: BTreeSet<Sym> = a.sigma.clone();
                sigma.extend(b.sigma.iter().cloned());
                let alpha: Vec<Sym> = sigma.into_iter().collect();
                let (Some(fa_), Some(fb)) = (a.to_fa(&alpha), b.to_fa(&alpha)) else {
                    return err("E504", span, "concat needs two finite automata");
                };
                let mut values: Vec<Value> = a
                    .states
                    .iter()
                    .map(|q| Value::tuple(vec![Value::Int(1), q.clone()]))
                    .collect();
                values.extend(
                    b.states
                        .iter()
                        .map(|q| Value::tuple(vec![Value::Int(2), q.clone()])),
                );
                obj(self.fa_machine(&ops::concat(&fa_, &fb), &alpha, &values, "nfa", None))
            }
            "to_regex" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                let order: Option<Vec<usize>> = match named.iter().find(|(n, _, _)| n == "order") {
                    Some((_, v, s)) => Some(
                        self.items_of(v, *s)?
                            .iter()
                            .filter_map(|x| m.index_of(x))
                            .collect(),
                    ),
                    None => None,
                };
                let mut steps = Vec::new();
                let re = match method.as_deref().unwrap_or("state-elimination") {
                    "state-elimination" => {
                        to_regex::state_elimination(&f, order.as_deref(), &mut steps)
                    }
                    "arden" => to_regex::arden(&f, &mut steps),
                    "brzozowski-mccluskey" => to_regex::brzozowski_mccluskey(&f, &mut steps),
                    other => {
                        return err(
                            "E507",
                            span,
                            format!(
                                "unknown method \"{other}\": use \"state-elimination\", \"arden\" or \"brzozowski-mccluskey\""
                            ),
                        );
                    }
                };
                obj_rc(rx::regex_value(
                    Rx::raise(&re.simplify(), &alpha),
                    Some(m.sigma.clone()),
                ))
            }
            "simplify" => {
                let (x, s) = args.first().cloned().unwrap_or((Value::None, span));
                let Some(r) = x.rc::<RegexVal>() else {
                    return err("E504", s, "simplify needs a regex");
                };
                let alpha = r.alpha_vec();
                obj_rc(rx::regex_value(
                    Rx::raise(&r.rx.lower(&alpha).simplify(), &alpha),
                    Some(r.alphabet.clone()),
                ))
            }
            "to_gnfa" => {
                let m = self.arg_machine(args, 0, span)?;
                let (f, alpha) = self.machine_fa(&m, span)?;
                obj(self.gnfa_machine(&f, &alpha, &m))
            }
            "equivalent" | "subset_of" => {
                let a = self.lang_arg(args, 0, span)?;
                let b = self.lang_arg(args, 1, span)?;
                let q = if name == "equivalent" {
                    Question::Equiv
                } else {
                    Question::Subset
                };
                Value::Bool(self.decide(q, &a, Some(&b), span)?.0)
            }
            "is_empty" | "is_finite" | "is_universal" => {
                let a = self.lang_arg(args, 0, span)?;
                let q = match name {
                    "is_empty" => Question::IsEmpty,
                    "is_finite" => Question::IsFinite,
                    _ => Question::IsUniversal,
                };
                Value::Bool(self.decide(q, &a, None, span)?.0)
            }
            "shortest" | "words" => {
                let a = self.lang_arg(args, 0, span)?;
                let alpha: Vec<Sym> = a.sigma.iter().cloned().collect();
                let Some(f) = self.compile_lang(&a, span)? else {
                    return err(
                        "E702",
                        span,
                        format!("`{name}` is implemented for regular languages only"),
                    );
                };
                let wv = |w: &[u32]| {
                    Value::Word(Rc::new(
                        w.iter()
                            .filter_map(|i| alpha.get(*i as usize).cloned())
                            .collect(),
                    ))
                };
                if name == "shortest" {
                    match decide::shortest(&f, max).map_err(|_| budget_states(span, max))? {
                        Some(w) => wv(&w),
                        None => Value::None,
                    }
                } else {
                    let upto = named
                        .iter()
                        .find(|(n, _, _)| n == "upto")
                        .map(|(_, v, _)| v.clone())
                        .or_else(|| args.get(1).map(|a| a.0.clone()));
                    let Some(Value::Int(n)) = upto else {
                        return err("E507", span, "words(L, upto: n) needs `upto:`");
                    };
                    let limit = self.budget.output.max(1000);
                    let ws = decide::words(&f, n.max(0) as usize, limit)
                        .map_err(|_| budget_states(span, max))?;
                    Value::set(ws.iter().map(|w| wv(w)).collect())
                }
            }
            "count" if named.iter().any(|(n, _, _)| n == "length") => {
                let a = self.lang_arg(args, 0, span)?;
                let Some((_, Value::Int(n), _)) =
                    named.iter().find(|(n, _, _)| n == "length").cloned()
                else {
                    return err("E507", span, "count(L, length: n) needs an int");
                };
                let Some(f) = self.compile_lang(&a, span)? else {
                    return err(
                        "E702",
                        span,
                        "`count` is implemented for regular languages only",
                    );
                };
                let c = decide::count(&f, n.max(0) as usize, max)
                    .map_err(|_| budget_states(span, max))?;
                Value::Int(i64::try_from(c).unwrap_or(i64::MAX))
            }
            "steps" => {
                let (x, _) = args.first().cloned().unwrap_or((Value::None, span));
                match x.rc::<Machine>().and_then(|m| m.provenance.clone()) {
                    Some(p) => Value::list(p.steps.iter().map(|s| text_word(&s.text)).collect()),
                    None => Value::list(Vec::new()),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    fn lang_arg(&mut self, args: &Args, i: usize, span: Span) -> R<Rc<LangVal>> {
        let (v, s) = args.get(i).cloned().unwrap_or((Value::None, span));
        self.to_lang(&v, s)
    }

    /// `to_gnfa`: a new start and accept state, one regex edge between every pair (∅ allowed).
    fn gnfa_machine(&self, f: &Fa, alpha: &[Sym], m: &Machine) -> Machine {
        let n = f.n();
        let s = fresh_atom("start", &m.states);
        let a = fresh_atom("accept", &m.states);
        let v = |i: usize| -> Value {
            if i == n {
                s.clone()
            } else if i == n + 1 {
                a.clone()
            } else {
                m.state(i)
            }
        };
        let mut lab: BTreeMap<(usize, usize), core_rx::Re> = BTreeMap::new();
        for (p, list) in f.edges.iter().enumerate() {
            for (sym, q) in list {
                let r = sym.map(core_rx::Re::Sym).unwrap_or(core_rx::Re::Eps);
                let cur = lab.remove(&(p, *q)).unwrap_or(core_rx::Re::Empty);
                lab.insert((p, *q), core_rx::Re::alt(cur, r));
            }
        }
        lab.insert((n, f.start), core_rx::Re::Eps);
        for q in 0..n {
            if f.is_accept(q) {
                lab.insert((q, n + 1), core_rx::Re::Eps);
            }
        }
        let gm = self
            .models
            .get("gnfa")
            .cloned()
            .unwrap_or_else(|| Rc::new(Model::blank("gnfa")));
        let mut edges = Vec::new();
        for i in (0..n).chain([n]) {
            for j in (0..n).chain([n + 1]) {
                let r = lab.get(&(i, j)).cloned().unwrap_or(core_rx::Re::Empty);
                let text = Rx::raise(&r, alpha).text();
                edges.push((v(i), Label::new(vec![LOp::Regex(text)]), v(j)));
            }
        }
        let eff = (*gm).clone();
        let mut states: BTreeSet<Value> = m.states.iter().cloned().collect();
        states.insert(s.clone());
        states.insert(a.clone());
        Machine::assemble(Parts {
            model: gm,
            eff,
            name: None,
            states,
            sigma: alpha.iter().cloned().collect(),
            stack_alpha: BTreeSet::new(),
            tape_alpha: BTreeSet::new(),
            out_alpha: BTreeSet::new(),
            edges,
            start: s,
            accept: [a].into_iter().collect(),
            reject: None,
            blank: label::blank(),
            initial_stack: None,
            provenance: Some(Provenance {
                algorithm: "to_gnfa/special-form".to_string(),
                steps: Vec::new(),
            }),
        })
    }

    // ----- statements -----

    pub fn check(&mut self, s: &Stmt, subject: &Expr, kind: &CheckKind) -> R<()> {
        let line = self.line(s.span);
        match kind {
            CheckKind::Accepts(words) | CheckKind::Rejects(words) => {
                let want = matches!(kind, CheckKind::Accepts(_));
                let v = self.eval(subject)?;
                let l = self.to_lang(&v, subject.span)?;
                let who = self.text(subject.span);
                for we in words {
                    let wv = self.eval(we)?;
                    let w = self.word_of(&wv, we.span)?;
                    let got = self.lang_member(&l, &w, we.span)?;
                    let verb = if want { "accepts" } else { "rejects" };
                    let name = format!("{who} {verb} \"{}\"", word_text(&w));
                    let mut out = json!({ "kind": "test", "line": line, "name": name });
                    match got {
                        Some(b) => {
                            out["pass"] = json!(b == want);
                            if b != want {
                                out["message"] = json!(if b {
                                    "it accepts this word"
                                } else {
                                    "it rejects this word"
                                });
                            }
                        }
                        None => {
                            out["pass"] = json!(false);
                            out["verdict"] = json!("unknown");
                            out["message"] = json!("the budget ran out before a verdict (unknown)");
                        }
                    }
                    self.emit(s.span, out)?;
                }
                Ok(())
            }
            CheckKind::Holds(bound) => {
                let name = self.text(subject.span);
                if let ExprKind::Binary(op @ (BinOp::Equiv | BinOp::Subset), a, b) = &subject.kind {
                    let av = self.eval(a)?;
                    let bv = self.eval(b)?;
                    let la = self.to_lang(&av, a.span)?;
                    let lb = self.to_lang(&bv, b.span)?;
                    let q = if *op == BinOp::Equiv {
                        Question::Equiv
                    } else {
                        Question::Subset
                    };
                    let (pass, witness, bounded) = match bound {
                        Some(n) => {
                            let n = self.eval_int(n)?.max(0) as usize;
                            let (p, w) = self.bounded(q, &la, &lb, n, subject.span)?;
                            (p, w, Some(n))
                        }
                        None => {
                            let (p, w) = self.decide(q, &la, Some(&lb), subject.span)?;
                            (p, w, None)
                        }
                    };
                    let mut out =
                        json!({ "kind": "test", "line": line, "name": name, "pass": pass });
                    if let Some(n) = bounded {
                        out["bounded"] = json!(n);
                        out["name"] = json!(format!("{name} up to {n}"));
                    }
                    if let Some(w) = witness {
                        out["witness"] = json!(word_text(&w));
                        out["message"] = json!(format!("they differ on \"{}\"", word_text(&w)));
                    }
                    if la.assumed || lb.assumed {
                        out["assumed"] = json!(true);
                    }
                    return self.emit(s.span, out);
                }
                if bound.is_some() {
                    return err(
                        "E504",
                        subject.span,
                        "`up to n` applies to `≡` and `⊆` checks",
                    );
                }
                let v = self.eval(subject)?;
                let pass = self.truth(&v, subject.span)?;
                self.emit(
                    s.span,
                    json!({ "kind": "test", "line": line, "name": name, "pass": pass }),
                )
            }
        }
    }

    pub fn run_stmt(&mut self, s: &Stmt, m: &Expr, w: &Expr) -> R<()> {
        let mv = self.eval(m)?;
        let Some(machine) = mv.rc::<Machine>() else {
            return err(
                "E504",
                m.span,
                format!("`run` needs a machine, found {}", mv.display()),
            );
        };
        let wv = self.eval(w)?;
        let word = self.word_of(&wv, w.span)?;
        let t = self.trace_value(&machine, &word, s.span)?;
        let line = self.line(s.span);
        let mut out = self.object_json(&t);
        out["kind"] = json!("trace");
        out["line"] = json!(line);
        self.emit(s.span, out)
    }

    pub fn lang_decl(&mut self, value: &Expr, claim: Option<&(String, Span, Expr)>) -> R<Value> {
        let v = self.eval(value)?;
        let l = self.to_lang(&v, value.span)?;
        let Some((class_name, cspan, evidence)) = claim else {
            return Ok(obj_rc(l));
        };
        let Some(class) = Class::from_name(class_name) else {
            return err(
                "E201",
                *cspan,
                format!(
                    "unknown class `{class_name}`: use Finite, Regular, DCF, CF, CS, Decidable or Recognizable"
                ),
            );
        };
        let ev = self.eval(evidence)?;
        let le = self.to_lang(&ev, evidence.span)?;
        if le.class > class {
            return err(
                "E703",
                evidence.span,
                format!(
                    "the evidence is {}, so it can't show that the language is {}",
                    le.class.name(),
                    class.name()
                ),
            );
        }
        // Check the claim up to length 8 (Q-76).
        let (same, witness) = self.bounded(Question::Equiv, &l, &le, 8, value.span)?;
        if !same {
            let w = witness.map(|w| word_text(&w)).unwrap_or_default();
            return err(
                "E703",
                *cspan,
                format!(
                    "the claim `as {class_name}` is false: the language and the evidence differ on \"{w}\""
                ),
            );
        }
        let sigma = l.sigma.clone();
        let src = Src::Assumed {
            inner: l,
            evidence: le,
        };
        Ok(obj_rc(self.new_lang(class, src, sigma, true)))
    }

    pub fn model_decl(&mut self, decl: &ModelDecl) -> R<()> {
        let m = model::from_decl(decl, &self.models).map_err(Flow::Error)?;
        self.models.insert(decl.name.clone(), Rc::new(m));
        Ok(())
    }

    pub fn regex_literal(
        &mut self,
        text: &str,
        text_span: Span,
        over: Option<&Expr>,
        span: Span,
    ) -> R<Value> {
        let mut warnings = Vec::new();
        let plus_union = self.plus_is_union;
        // Notation profiles map their spellings to the core tokens (01 §10).
        let mut text = text.to_string();
        for (from, to) in self.notation_tokens.clone() {
            text = text.replace(&from, &to);
        }
        let rx =
            rx::parse(&text, text_span.from, plus_union, &mut warnings).map_err(Flow::Error)?;
        for w in warnings {
            if !self
                .pending
                .iter()
                .any(|d| d.span == w.span && d.code == w.code)
            {
                self.pending.push(w);
            }
        }
        let over_set = match over {
            Some(e) => {
                let v = self.eval(e)?;
                let mut out = Vec::new();
                self.value_syms(&v, e.span, &mut out)?;
                Some(out.into_iter().map(|(s, _)| s).collect::<BTreeSet<Sym>>())
            }
            None => None,
        };
        if rx.uses_any() && over_set.is_none() {
            let mut own = BTreeSet::new();
            rx.symbols(&mut own);
            if own.is_empty() {
                return err(
                    "E402",
                    span,
                    "Σ in a regex needs an alphabet: add `over {…}`",
                );
            }
        }
        Ok(obj_rc(rx::regex_value(rx, over_set)))
    }

    pub fn cfg_literal(
        &mut self,
        name: Option<&str>,
        body: &str,
        body_span: Span,
        _span: Span,
    ) -> R<Value> {
        match grammar::parse(name, body) {
            Ok(g) => Ok(obj(g)),
            Err((off, msg)) => err(
                "E403",
                Span::new(body_span.from + off, body_span.from + off + 1),
                msg,
            ),
        }
    }

    /// `{ w ∈ S* | cond }`: a predicate language (never materialized).
    pub fn predicate_lang(
        &mut self,
        pat: &Pattern,
        domain: &Expr,
        conds: &[Qual],
        span: Span,
    ) -> R<Value> {
        let base = match &domain.kind {
            ExprKind::Star(inner) | ExprKind::Plus(inner) => inner,
            _ => return err("X999", span, "internal: predicate language without Σ*"),
        };
        // Bare names in the alphabet `{a, b}` are symbols when they aren't variables.
        let sv = match &base.kind {
            ExprKind::Set(items) => {
                let mut set = BTreeSet::new();
                for i in items {
                    match &i.kind {
                        ExprKind::Var(n) if self.lookup(n).is_none() => {
                            set.insert(Value::sym(n));
                        }
                        _ => {
                            set.insert(self.eval(i)?);
                        }
                    }
                }
                Value::set(set)
            }
            _ => self.eval(base)?,
        };
        let mut out = Vec::new();
        self.value_syms(&sv, base.span, &mut out)?;
        let sigma: BTreeSet<Sym> = out.into_iter().map(|(s, _)| s).collect();
        let mut cond: Option<Expr> = None;
        for q in conds {
            let Qual::Cond(c) = q else {
                return err(
                    "E504",
                    span,
                    "a language's set-builder has one generator `w ∈ Σ*` and conditions",
                );
            };
            cond = Some(match cond {
                None => c.clone(),
                Some(prev) => Expr {
                    span: prev.span.join(c.span),
                    kind: ExprKind::Binary(BinOp::And, Box::new(prev), Box::new(c.clone())),
                },
            });
        }
        let mut cond = cond.unwrap_or(Expr {
            kind: ExprKind::Bool(true),
            span,
        });
        if matches!(domain.kind, ExprKind::Plus(_))
            && let Pattern::Var(n, _) = pat
        {
            let nonempty = Expr {
                span,
                kind: ExprKind::Binary(
                    BinOp::Ne,
                    Box::new(Expr {
                        kind: ExprKind::Var(n.clone()),
                        span,
                    }),
                    Box::new(Expr {
                        kind: ExprKind::Eps,
                        span,
                    }),
                ),
            };
            cond = Expr {
                span,
                kind: ExprKind::Binary(BinOp::And, Box::new(nonempty), Box::new(cond)),
            };
        }
        let mut names = BTreeSet::new();
        crate::interp::free_in_expr(&cond, &mut names);
        let mut captured = Vec::new();
        for n in names {
            for scope in self.scopes.iter().rev() {
                if let Some(b) = scope.get(&n) {
                    captured.push((n.clone(), b.value.clone()));
                    break;
                }
            }
        }
        let src = Src::Predicate {
            pat: pat.clone(),
            cond: Rc::new(cond),
            captured,
        };
        Ok(obj_rc(self.new_lang(Class::Decidable, src, sigma, false)))
    }

    // ----- explore (SEMANTICS §8) -----

    fn bind_and_eval(
        &mut self,
        pat: &Pattern,
        v: &Value,
        sym: Option<(&str, &Sym)>,
        e: &Expr,
    ) -> R<Value> {
        self.scopes.push(HashMap::new());
        let r = (|| -> R<Value> {
            self.bind_pattern(pat, v.clone(), false)?;
            if let Some((n, a)) = sym {
                self.bind_pattern(
                    &Pattern::Var(n.to_string(), Span::default()),
                    Value::Sym(a.clone()),
                    false,
                )?;
            }
            self.eval(e)
        })();
        self.scopes.pop();
        r
    }

    pub fn explore(&mut self, x: &ExploreLit, span: Span) -> R<Value> {
        let mut alpha: Option<BTreeSet<Sym>> = None;
        let mut start: Option<Value> = None;
        for c in &x.clauses {
            match c {
                ExploreClause::Alphabet(e) => {
                    let v = self.eval(e)?;
                    let mut out = Vec::new();
                    self.value_syms(&v, e.span, &mut out)?;
                    alpha = Some(out.into_iter().map(|(s, _)| s).collect());
                }
                ExploreClause::Start(e) => start = Some(self.eval(e)?),
                _ => {}
            }
        }
        let Some(alpha) = alpha else {
            return err("E107", span, "`explore` needs an `alphabet` clause");
        };
        let Some(start) = start else {
            return err("E108", span, "`explore` needs a `start` clause");
        };
        let only_det = x.clauses.iter().all(|c| match c {
            ExploreClause::Step { any, on, .. } => !*any && matches!(on, StepOn::Sym(..)),
            ExploreClause::Edges { .. } => false,
            _ => true,
        });
        let model_name = x.model.as_ref().map(|(n, _)| n.clone()).unwrap_or_else(|| {
            if only_det {
                "dfa".to_string()
            } else {
                "nfa".to_string()
            }
        });
        let Some(model) = self.model_named(&model_name) else {
            let s = x.model.as_ref().map(|(_, s)| *s).unwrap_or(span);
            return err("E301", s, format!("unknown model `{model_name}`"));
        };
        let syms: Vec<Sym> = alpha.iter().cloned().collect();
        let mut states: Vec<Value> = vec![start.clone()];
        let mut index: BTreeSet<Value> = BTreeSet::new();
        index.insert(start.clone());
        let mut edges: Vec<(Value, Label, Value)> = Vec::new();
        let mut accept = BTreeSet::new();
        let mut steps = vec![Step::new("AddState", format!("start {}", start.display()))];
        let mut missing = false;
        let mut i = 0;
        let max_states = self.budget.states as usize;
        while let Some(v) = states.get(i).cloned() {
            i += 1;
            for c in &x.clauses {
                self.tick(span)?;
                let mut targets: Vec<(Label, Value)> = Vec::new();
                match c {
                    ExploreClause::Step {
                        pat,
                        on,
                        any,
                        target,
                        ..
                    } => {
                        if !Self::pattern_matches(pat, &v) {
                            continue;
                        }
                        match on {
                            StepOn::Eps => {
                                let t = self.bind_and_eval(pat, &v, None, target)?;
                                for u in self.items_of(&t, target.span)? {
                                    targets.push((Label::eps(), u));
                                }
                            }
                            StepOn::Sym(n, _) => {
                                for a in &syms {
                                    let t = self.bind_and_eval(pat, &v, Some((n, a)), target)?;
                                    let lab = label::read_label(Some(a.clone()));
                                    if *any {
                                        for u in self.items_of(&t, target.span)? {
                                            targets.push((lab.clone(), u));
                                        }
                                    } else if t == Value::None {
                                        missing = true;
                                    } else {
                                        targets.push((lab, t));
                                    }
                                }
                            }
                        }
                    }
                    ExploreClause::Edges { pat, target } => {
                        if !Self::pattern_matches(pat, &v) {
                            continue;
                        }
                        let list = self.bind_and_eval(pat, &v, None, target)?;
                        for item in self.items_of(&list, target.span)? {
                            let pair = match &item {
                                Value::Tuple(t) => (t.first().cloned(), t.get(1).cloned()),
                                _ => (None, None),
                            };
                            let (Some(lv), Some(tv)) = pair else {
                                return err(
                                    "E504",
                                    target.span,
                                    "edges gives a list of (label, target) pairs",
                                );
                            };
                            let text = match &lv {
                                Value::Word(w) if w.is_empty() => "ε".to_string(),
                                Value::Word(w) => word_text(w),
                                Value::Sym(s) => sym_text(s),
                                other => other.print_text(),
                            };
                            let labs = label::parse_labels(&text, &model, &label::blank())
                                .map_err(|m| {
                                    Flow::Error(Diagnostic::error("E309", target.span, m))
                                })?;
                            for l in labs {
                                targets.push((l, tv.clone()));
                            }
                        }
                    }
                    _ => continue,
                }
                for (l, t) in targets {
                    if !index.contains(&t) {
                        if states.len() >= max_states {
                            return Err(budget_states(span, max_states));
                        }
                        index.insert(t.clone());
                        steps.push(Step::new("AddState", format!("new state {}", t.display())));
                        states.push(t.clone());
                    }
                    steps.push(Step::new(
                        "AddEdge",
                        format!("{} → {}", v.display(), t.display()),
                    ));
                    edges.push((v.clone(), l, t));
                }
            }
        }
        for v in &states {
            for c in &x.clauses {
                if let ExploreClause::Accept { pat, cond } = c
                    && Self::pattern_matches(pat, v)
                {
                    let r = self.bind_and_eval(pat, v, None, cond)?;
                    if self.truth(&r, cond.span)? {
                        accept.insert(v.clone());
                    }
                }
            }
        }
        let mut eff = (*model).clone();
        if eff.total && eff.deterministic {
            // Missing successors (`none`) leave the result partial (SEMANTICS §8.2).
            let have: BTreeSet<(Value, Sym)> = edges
                .iter()
                .filter_map(|(f, l, _)| l.read().map(|a| (f.clone(), a.clone())))
                .collect();
            let incomplete = missing
                || states
                    .iter()
                    .any(|q| syms.iter().any(|a| !have.contains(&(q.clone(), a.clone()))));
            if incomplete {
                eff.total = false;
                eff.missing = Missing::Dead;
            }
        }
        Ok(obj(Machine::assemble(Parts {
            model,
            eff,
            name: None,
            states: states.into_iter().collect(),
            sigma: alpha,
            stack_alpha: BTreeSet::new(),
            tape_alpha: BTreeSet::new(),
            out_alpha: BTreeSet::new(),
            edges,
            start,
            accept,
            reject: None,
            blank: label::blank(),
            initial_stack: None,
            provenance: Some(Provenance {
                algorithm: "explore".to_string(),
                steps,
            }),
        })))
    }

    // ----- JSON for show and traces -----

    pub fn object_json(&self, v: &Value) -> serde_json::Value {
        if let Some(m) = v.downcast::<Machine>() {
            let st = |q: usize| Machine::display_state(&m.state(q));
            let mut out = json!({
                "type": "machine",
                "model": m.model.display(),
                "states": m.states.iter().map(Machine::display_state).collect::<Vec<_>>(),
                "alphabet": m.sigma.iter().map(|s| sym_text(s)).collect::<Vec<_>>(),
                "start": st(m.start),
                "accept": m.accept.iter().map(|q| st(*q)).collect::<Vec<_>>(),
                "edges": m.edges.iter().map(|e| json!([st(e.from), m.label_text(&e.label), st(e.to)])).collect::<Vec<_>>(),
                "text": m.text(),
            });
            if let Some(n) = &m.name {
                out["name"] = json!(n);
            }
            if let Some(r) = m.reject {
                out["reject"] = json!(st(r));
            }
            if m.eff.missing == Missing::Dead && !m.eff.total {
                out["partial"] = json!(true);
            }
            if let Some(p) = &m.provenance {
                out["algorithm"] = json!(p.algorithm);
                out["steps"] = json!(
                    p.steps
                        .iter()
                        .map(|s| json!({ "kind": s.kind, "text": s.text }))
                        .collect::<Vec<_>>()
                );
                if let Some(cite) = self.citation(&p.algorithm) {
                    out["cite"] = json!(cite);
                }
            }
            return out;
        }
        if let Some(t) = v.downcast::<TraceVal>() {
            let steps: Vec<serde_json::Value> = t
                .steps
                .iter()
                .map(|layer| {
                    if t.deterministic && layer.len() == 1 {
                        json!({ "config": layer.first() })
                    } else {
                        json!({ "configs": layer })
                    }
                })
                .collect();
            return json!({ "type": "trace", "machine": t.machine, "input": word_text(&t.input), "verdict": t.verdict.name(), "steps": steps });
        }
        if let Some(l) = v.downcast::<LangVal>() {
            return json!({
                "type": "lang",
                "class": l.class.name(),
                "assumed": l.assumed,
                "alphabet": l.sigma.iter().map(|s| sym_text(s)).collect::<Vec<_>>(),
                "text": l.describe()
            });
        }
        json!({ "type": v.type_name(), "text": v.display() })
    }
}

/// A gnfa's language as an NFA: each regex edge becomes an NFA fragment.
fn gnfa_fa(m: &Machine, alpha: &[Sym]) -> Option<Fa> {
    let k = alpha.len();
    let mut f = Fa::new(k, m.states.len(), m.start);
    for q in &m.accept {
        if let Some(a) = f.accept.get_mut(*q) {
            *a = true;
        }
    }
    for e in &m.edges {
        let text = match e.label.ops.as_slice() {
            [LOp::Regex(t)] => t.clone(),
            _ => return None,
        };
        let mut warn = Vec::new();
        let r = rx::parse(&text, 0, false, &mut warn).ok()?;
        let frag = core_rx::to_nfa_thompson(&r.lower(alpha), k);
        let off = f.n();
        for _ in 0..frag.n() {
            f.add_state(false);
        }
        for (p, list) in frag.edges.iter().enumerate() {
            for (a, q) in list {
                f.add_edge(p + off, *a, q + off);
            }
        }
        f.add_edge(e.from, None, frag.start + off);
        for q in 0..frag.n() {
            if frag.is_accept(q) {
                f.add_edge(q + off, None, e.to);
            }
        }
    }
    Some(f)
}

/// A fresh atom `:base`, `:base1`, … not among `states`.
fn fresh_atom(base: &str, states: &[Value]) -> Value {
    (0..)
        .map(|i| {
            if i == 0 {
                base.to_string()
            } else {
                format!("{base}{i}")
            }
        })
        .map(|n| atom(&n))
        .find(|a| !states.contains(a))
        .unwrap_or_else(|| atom(base))
}

/// The same finite automaton over a larger alphabet (missing moves now go to a dead state).
fn widen(m: &Machine, alpha: &[Sym]) -> Machine {
    let mut p = m.parts();
    p.sigma = alpha.iter().cloned().collect();
    if p.sigma.len() > m.sigma.len() {
        p.eff.total = false;
        p.eff.missing = Missing::Dead;
    }
    Machine::assemble(p)
}

pub fn budget_states(span: Span, max: usize) -> Flow {
    Flow::Budget(Diagnostic::warning(
        "B602",
        span,
        format!(
            "stopped after creating {max} states (the states budget); raise it with `budget {{ states: … }}`"
        ),
    ))
}

/// E701: an undecidable question, with the bounded alternative as a quick fix.
pub fn undecidable(
    q: Question,
    a: Class,
    b: Option<Class>,
    span: Span,
    insert_at: Option<usize>,
) -> Diagnostic {
    let joined = b.map(|b| a.max(b)).unwrap_or(a);
    let classes = match b {
        Some(b) => format!("the operands are {} and {}", a.name(), b.name()),
        None => format!("the language is {}", a.name()),
    };
    let msg = format!(
        "{} of {} is undecidable ({classes})",
        q.text(),
        lang::class_prose(joined)
    );
    let mut d = Diagnostic::error("E701", span, msg);
    if matches!(q, Question::Equiv | Question::Subset) {
        let at = insert_at.unwrap_or(span.to);
        d = d.with_fix(
            format!("use a bounded check: up to {SUGGESTED_BOUND}"),
            Span::new(at, at),
            format!(" up to {SUGGESTED_BOUND}"),
        );
    }
    d
}

pub fn not_implemented(q: Question, c: Class, span: Span) -> Diagnostic {
    let mut d = Diagnostic::error(
        "E702",
        span,
        format!(
            "{} of {} is decidable, but Delta doesn't implement the algorithm",
            q.text(),
            lang::class_prose(c)
        ),
    );
    if matches!(q, Question::Equiv | Question::Subset) {
        d = d.with_fix(
            format!("use a bounded check: up to {SUGGESTED_BOUND}"),
            Span::new(span.to, span.to),
            format!(" up to {SUGGESTED_BOUND}"),
        );
    }
    d
}
