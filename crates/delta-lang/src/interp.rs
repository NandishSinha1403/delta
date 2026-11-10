//! The interpreter: a tree-walker over the syntax tree (SEMANTICS.md §10). Evaluation is strict
//! and left to right. Every step counts against the budget, so no program can hang the engine.

use crate::ast::*;
use crate::builtins::{Args, Named};
use crate::diag::{Diagnostic, LineMap, Span};
use crate::value::{Class, FnVal, Sym, Value};
use serde_json::json;
// `HashMap` maps keys to values; scopes use it because their order never shows.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

/// Limits on one evaluation (01 §13).
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub steps: u64,
    pub states: u64,
    pub depth: u32,
    pub output: usize,
}

impl Default for Budget {
    fn default() -> Budget {
        Budget {
            steps: 10_000_000,
            states: 100_000,
            depth: 400,
            output: 10_000,
        }
    }
}

/// Why evaluation stopped early.
#[derive(Debug)]
pub enum Flow {
    /// `return v` unwinding to the enclosing function.
    Return(Value),
    /// A runtime error (E5xx).
    Error(Diagnostic),
    /// A budget outcome (B6xx).
    Budget(Diagnostic),
}

pub type R<T> = Result<T, Flow>;

pub fn err<T>(code: &'static str, span: Span, msg: impl Into<String>) -> R<T> {
    Err(Flow::Error(Diagnostic::error(code, span, msg)))
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub value: Value,
    pub mutable: bool,
}

pub struct Interp<'s> {
    pub src: &'s str,
    pub lines: LineMap,
    pub globals: HashMap<String, Binding>,
    /// The local scopes of the function being run, innermost last.
    pub scopes: Vec<HashMap<String, Binding>>,
    pub budget: Budget,
    pub steps: u64,
    pub depth: u32,
    next_id: u64,
    pub outputs: Vec<serde_json::Value>,
    free_cache: HashMap<usize, Rc<Vec<String>>>,
    pub strict: bool,
    pub explain: Option<String>,
    /// Models by name: the prelude, then `model` declarations.
    pub models: HashMap<String, Rc<crate::model::Model>>,
    /// Extra diagnostics found while running (warnings, further errors of one machine).
    pub pending: Vec<Diagnostic>,
    /// Notation profile: `+` in regexes means union (hmu, linz).
    pub plus_is_union: bool,
    /// Extra regex spellings of a custom notation profile (`φ` → `∅`).
    pub notation_tokens: Vec<(String, String)>,
}

/// Names of the built-in functions (01 §12). Those not available yet report E509 when called.
pub const BUILTINS: [&str; 61] = [
    "len",
    "count",
    "rev",
    "set",
    "list",
    "sort",
    "prefixes",
    "suffixes",
    "substrings",
    "power_set",
    "min",
    "max",
    "abs",
    "range",
    "steps",
    "to_nfa",
    "to_regex",
    "to_gnfa",
    "simplify",
    "determinize",
    "minimize",
    "complete",
    "trim",
    "remove_epsilon",
    "reverse",
    "complement",
    "union",
    "intersect",
    "difference",
    "concat",
    "star",
    "product",
    "equivalent",
    "subset_of",
    "is_empty",
    "is_finite",
    "is_universal",
    "shortest",
    "words",
    "myhill_nerode",
    "pump",
    "cnf",
    "gnf",
    "nullable",
    "useless",
    "parse",
    "derive",
    "to_pda",
    "to_cfg",
    "is_ambiguous",
    "to_single_tape",
    "mealy_to_moore",
    "moore_to_mealy",
    "keys",
    "values",
    "first",
    "last",
    "sum",
    "symbols",
    "word",
    "atom",
];

fn builtin_id(name: &str) -> u64 {
    BUILTINS.iter().position(|b| *b == name).unwrap_or(0) as u64
}

impl<'s> Interp<'s> {
    pub fn new(src: &'s str) -> Interp<'s> {
        Interp {
            src,
            lines: LineMap::new(src),
            globals: HashMap::new(),
            scopes: Vec::new(),
            budget: Budget::default(),
            steps: 0,
            depth: 0,
            next_id: 1_000,
            outputs: Vec::new(),
            free_cache: HashMap::new(),
            strict: false,
            explain: None,
            models: crate::prelude::models(),
            pending: Vec::new(),
            plus_is_union: false,
            notation_tokens: Vec::new(),
        }
    }

    pub fn fresh_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn line(&self, span: Span) -> usize {
        self.lines.line_col(self.src, span.from).0
    }

    pub fn text(&self, span: Span) -> String {
        self.src
            .get(span.from..span.to)
            .unwrap_or("")
            .trim()
            .to_string()
    }

    /// Counts one step against the budget.
    pub fn tick(&mut self, span: Span) -> R<()> {
        self.steps += 1;
        if self.steps > self.budget.steps {
            return Err(Flow::Budget(Diagnostic::warning(
                "B601",
                span,
                format!(
                    "stopped after {} steps (the step budget); raise it with `budget {{ steps: … }}`",
                    self.budget.steps
                ),
            )));
        }
        Ok(())
    }

    pub fn emit(&mut self, span: Span, out: serde_json::Value) -> R<()> {
        if self.outputs.len() >= self.budget.output {
            return Err(Flow::Budget(Diagnostic::warning(
                "B604",
                span,
                format!(
                    "stopped after {} outputs (the output budget)",
                    self.budget.output
                ),
            )));
        }
        self.outputs.push(out);
        Ok(())
    }

    // ----- running a program -----

    /// Runs the program. Returns the diagnostic that stopped it, if any.
    pub fn run_program(&mut self, prog: &Program) -> Option<Diagnostic> {
        // Top-level functions are visible everywhere (they can call each other).
        for s in &prog.stmts {
            if let StmtKind::Fn(def) = &s.kind
                && let Some(name) = &def.name
            {
                let id = self.fresh_id();
                let f = Value::Fn(Rc::new(FnVal::Closure {
                    def: def.clone(),
                    captured: Vec::new(),
                    id,
                }));
                self.globals.insert(
                    name.clone(),
                    Binding {
                        value: f,
                        mutable: false,
                    },
                );
            }
        }
        for s in &prog.stmts {
            match self.exec(s) {
                Ok(()) => {}
                Err(Flow::Return(_)) => {}
                Err(Flow::Error(d)) | Err(Flow::Budget(d)) => return Some(d),
            }
        }
        None
    }

    fn bind(&mut self, name: &str, value: Value, mutable: bool) {
        let b = Binding { value, mutable };
        match self.scopes.last_mut() {
            Some(scope) => {
                scope.insert(name.to_string(), b);
            }
            None => {
                self.globals.insert(name.to_string(), b);
            }
        }
    }

    pub fn bind_pattern(&mut self, pat: &Pattern, v: Value, mutable: bool) -> R<()> {
        match pat {
            Pattern::Var(n, _) => {
                self.bind(n, v, mutable);
                Ok(())
            }
            Pattern::Wild(_) => Ok(()),
            Pattern::Tuple(ps, span) => {
                let items = match &v {
                    Value::Tuple(items) | Value::List(items) => items.clone(),
                    other => {
                        return err(
                            "E506",
                            *span,
                            format!(
                                "this pattern needs a tuple, but the value is {}",
                                other.display()
                            ),
                        );
                    }
                };
                if items.len() != ps.len() {
                    return err(
                        "E506",
                        *span,
                        format!(
                            "this pattern has {} parts, but the tuple has {}",
                            ps.len(),
                            items.len()
                        ),
                    );
                }
                for (p, item) in ps.iter().zip(items.iter()) {
                    self.bind_pattern(p, item.clone(), mutable)?;
                }
                Ok(())
            }
        }
    }

    /// Does `v` match `pat`? (Used by explore; binds nothing.)
    pub fn pattern_matches(pat: &Pattern, v: &Value) -> bool {
        match pat {
            Pattern::Var(..) | Pattern::Wild(_) => true,
            Pattern::Tuple(ps, _) => match v {
                Value::Tuple(items) => {
                    items.len() == ps.len()
                        && ps
                            .iter()
                            .zip(items.iter())
                            .all(|(p, i)| Self::pattern_matches(p, i))
                }
                _ => false,
            },
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Some(b.value.clone());
            }
        }
        if let Some(b) = self.globals.get(name) {
            return Some(b.value.clone());
        }
        if BUILTINS.contains(&name) {
            let n = BUILTINS
                .iter()
                .find(|b| **b == name)
                .copied()
                .unwrap_or("len");
            return Some(Value::Fn(Rc::new(FnVal::Builtin {
                name: n,
                id: builtin_id(name),
            })));
        }
        if let Some(c) = Class::from_name(name) {
            return Some(Value::Class(c));
        }
        self.lookup_extra(name)
    }

    fn assign(&mut self, name: &str, span: Span, v: Value) -> R<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(b) = scope.get_mut(name) {
                if !b.mutable {
                    return err(
                        "E202",
                        span,
                        format!("`{name}` was declared with `let` and can't be rebound; use `var`"),
                    );
                }
                b.value = v;
                return Ok(());
            }
        }
        if let Some(b) = self.globals.get_mut(name) {
            if !b.mutable {
                return err(
                    "E202",
                    span,
                    format!("`{name}` was declared with `let` and can't be rebound; use `var`"),
                );
            }
            b.value = v;
            return Ok(());
        }
        err(
            "E203",
            span,
            format!("`{name}` is not declared; declare it with `var {name} = …`"),
        )
    }

    fn block(&mut self, stmts: &[Stmt]) -> R<()> {
        self.scopes.push(HashMap::new());
        let mut result = Ok(());
        for s in stmts {
            result = self.exec(s);
            if result.is_err() {
                break;
            }
        }
        self.scopes.pop();
        result
    }

    pub fn exec(&mut self, s: &Stmt) -> R<()> {
        self.tick(s.span)?;
        match &s.kind {
            StmtKind::Let(pat, e) => {
                let v = self.eval(e)?;
                self.bind_pattern(pat, v, false)
            }
            StmtKind::Var(name, e) => {
                let v = self.eval(e)?;
                self.bind(name, v, true);
                Ok(())
            }
            StmtKind::Assign(name, span, e) => {
                let v = self.eval(e)?;
                self.assign(name, *span, v)
            }
            StmtKind::If(arms, els) => {
                for (cond, body) in arms {
                    if self.eval_bool(cond)? {
                        return self.block(body);
                    }
                }
                match els {
                    Some(b) => self.block(b),
                    None => Ok(()),
                }
            }
            StmtKind::For(pat, iter, body) => {
                let items = self.iterate(iter)?;
                for item in items {
                    self.tick(s.span)?;
                    self.scopes.push(HashMap::new());
                    let r = self
                        .bind_pattern(pat, item, false)
                        .and_then(|_| self.block(body));
                    self.scopes.pop();
                    r?;
                }
                Ok(())
            }
            StmtKind::While(cond, body) => {
                while self.eval_bool(cond)? {
                    self.tick(s.span)?;
                    self.block(body)?;
                }
                Ok(())
            }
            StmtKind::Return(e) => {
                let v = match e {
                    Some(e) => self.eval(e)?,
                    None => Value::None,
                };
                Err(Flow::Return(v))
            }
            StmtKind::Print(args) => {
                let mut parts = Vec::new();
                for a in args {
                    parts.push(self.eval(a)?.print_text());
                }
                let line = self.line(s.span);
                self.emit(
                    s.span,
                    json!({ "kind": "print", "line": line, "text": parts.join(" ") }),
                )
            }
            StmtKind::Show(e) => {
                let v = self.eval(e)?;
                let line = self.line(s.span);
                let value = self.value_json(&v);
                self.emit(
                    s.span,
                    json!({ "kind": "show", "line": line, "value": value }),
                )
            }
            StmtKind::Assert(e) => {
                let (pass, detail) = self.eval_test(e)?;
                let line = self.line(s.span);
                let mut out = json!({ "kind": "test", "line": line, "name": self.text(e.span), "pass": pass });
                if let (false, Some(d)) = (pass, detail) {
                    out["message"] = json!(d);
                }
                self.emit(s.span, out)
            }
            StmtKind::Save(e, p) => {
                let v = self.eval(e)?;
                let pv = self.eval(p)?;
                let Value::Word(w) = &pv else {
                    return err(
                        "E504",
                        p.span,
                        format!(
                            "`save … to` needs a file name string, not {} {}",
                            pv.type_name(),
                            pv.display()
                        ),
                    );
                };
                let path = &crate::hooks::word_text(w);
                let format = save_format(path).ok_or_else(|| {
                    Flow::Error(Diagnostic::error(
                        "E514",
                        p.span,
                        format!(
                            "can't save to \"{path}\": use .svg, .png, .pdf, .tex, .txt or .json"
                        ),
                    ))
                })?;
                let path = save_path(path).ok_or_else(|| {
                    Flow::Error(Diagnostic::error(
                        "E515",
                        p.span,
                        format!("can't save to \"{path}\": the path must be relative and stay inside this folder (no leading /, drive letter or ..)"),
                    ))
                })?;
                if matches!(format, "svg" | "png" | "pdf" | "tex")
                    && v.downcast::<crate::machine::Machine>().is_none()
                {
                    return err(
                        "E516",
                        e.span,
                        format!(
                            "only machines can be drawn; save {} {} as .txt or .json instead",
                            v.type_name(),
                            v.display()
                        ),
                    );
                }
                let line = self.line(s.span);
                let value = self.value_json(&v);
                self.emit(
                    s.span,
                    json!({ "kind": "save", "line": line, "path": path, "format": format, "value": value, "text": v.print_text() }),
                )
            }
            StmtKind::Check(subject, kind) => self.check(s, subject, kind),
            StmtKind::Run(m, w) => self.run_stmt(s, m, w),
            StmtKind::Expr(e) => {
                self.eval(e)?;
                Ok(())
            }
            StmtKind::Fn(def) => {
                if self.scopes.is_empty() {
                    return Ok(()); // already bound before the program started
                }
                let f = self.make_closure(def);
                if let Some(name) = &def.name {
                    self.bind(name, f, false);
                }
                Ok(())
            }
            StmtKind::Lang {
                name, value, claim, ..
            } => {
                let v = self.lang_decl(value, claim.as_ref())?;
                self.bind(name, v, false);
                Ok(())
            }
            StmtKind::Model(decl) => self.model_decl(decl),
            StmtKind::MachineDecl(name, _, e) => {
                let v = self.eval(e)?;
                self.bind(name, v, false);
                Ok(())
            }
            StmtKind::UseStrict => {
                self.strict = true;
                Ok(())
            }
            StmtKind::UseNotation(p) => {
                self.plus_is_union =
                    matches!(p.as_str(), "hmu" | "linz") || p.contains("union: \"+\"");
                // A custom profile: `{ union: "+", epsilon: "λ", empty: "φ" }`.
                for (key, core) in [("epsilon", "ε"), ("empty", "∅")] {
                    if let Some(rest) = p.split(&format!("{key}:")).nth(1)
                        && let Some(tok) = rest.split('"').nth(1)
                        && !tok.is_empty()
                        && tok != core
                    {
                        self.notation_tokens
                            .push((tok.to_string(), core.to_string()));
                    }
                }
                Ok(())
            }
            StmtKind::UseExplain(p) => {
                self.explain = Some(p.clone());
                Ok(())
            }
            StmtKind::Budget(pairs) => {
                for (k, span, e) in pairs {
                    let n = self.eval_int(e)?;
                    if n <= 0 {
                        return err("E504", e.span, "a budget must be a positive number");
                    }
                    let n = n as u64;
                    match k.as_str() {
                        "steps" => self.budget.steps = n,
                        "states" => self.budget.states = n,
                        "depth" => self.budget.depth = n.min(u64::from(u32::MAX)) as u32,
                        "output" => self.budget.output = n as usize,
                        other => {
                            return err(
                                "E504",
                                *span,
                                format!(
                                    "unknown budget `{other}`: use steps, states, depth or output"
                                ),
                            );
                        }
                    }
                }
                Ok(())
            }
            StmtKind::Block(b) => self.block(b),
        }
    }

    /// Evaluates a test expression. For a failing comparison, also describes both sides.
    fn eval_test(&mut self, e: &Expr) -> R<(bool, Option<String>)> {
        if let ExprKind::Binary(op, l, r) = &e.kind
            && matches!(
                op,
                BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::In
                    | BinOp::NotIn
            )
        {
            let lv = self.eval(l)?;
            let rv = self.eval(r)?;
            let v = self.binary(*op, lv.clone(), rv.clone(), e.span)?;
            let pass = self.truth(&v, e.span)?;
            let detail = format!("left: {}, right: {}", lv.display(), rv.display());
            return Ok((pass, Some(detail)));
        }
        let v = self.eval(e)?;
        Ok((self.truth(&v, e.span)?, None))
    }

    pub fn truth(&self, v: &Value, span: Span) -> R<bool> {
        match v {
            Value::Bool(b) => Ok(*b),
            other => err(
                "E504",
                span,
                format!(
                    "expected a bool (true or false), found {} {}",
                    other.type_name(),
                    other.display()
                ),
            ),
        }
    }

    pub fn eval_bool(&mut self, e: &Expr) -> R<bool> {
        let v = self.eval(e)?;
        self.truth(&v, e.span)
    }

    pub fn eval_int(&mut self, e: &Expr) -> R<i64> {
        match self.eval(e)? {
            Value::Int(n) => Ok(n),
            other => err(
                "E504",
                e.span,
                format!("expected an int, found {}", other.display()),
            ),
        }
    }

    /// The elements of a collection, in canonical (or list) order.
    pub fn iterate(&mut self, e: &Expr) -> R<Vec<Value>> {
        let v = self.eval(e)?;
        self.items_of(&v, e.span)
    }

    pub fn items_of(&mut self, v: &Value, span: Span) -> R<Vec<Value>> {
        match v {
            Value::Set(s) => Ok(s.iter().cloned().collect()),
            Value::List(l) | Value::Tuple(l) => Ok(l.to_vec()),
            Value::Word(w) => Ok(w.iter().map(|s| Value::Sym(s.clone())).collect()),
            Value::Map(m) => Ok(m
                .iter()
                .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))
                .collect()),
            other => self.items_of_extra(other, span),
        }
    }

    // ----- expressions -----

    pub fn eval(&mut self, e: &Expr) -> R<Value> {
        self.tick(e.span)?;
        match &e.kind {
            ExprKind::Int(n) => Ok(Value::Int(*n)),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::None => Ok(Value::None),
            ExprKind::Sym(s) => Ok(Value::sym(s)),
            ExprKind::Word(w) => Ok(Value::word_from(w)),
            ExprKind::Atom(a) => Ok(Value::Atom(Rc::from(a.as_str()))),
            ExprKind::Eps => Ok(Value::empty_word()),
            ExprKind::EmptySet => Ok(Value::set(BTreeSet::new())),
            ExprKind::Var(name) => match self.lookup(name) {
                Some(v) => Ok(v),
                None => err("E201", e.span, format!("unknown name `{name}`")),
            },
            ExprKind::Tuple(items) => {
                let vs = self.eval_all(items)?;
                Ok(Value::tuple(vs))
            }
            ExprKind::List(items) => {
                let vs = self.eval_all(items)?;
                Ok(Value::list(vs))
            }
            ExprKind::Set(items) => {
                let vs = self.eval_all(items)?;
                Ok(Value::set(vs.into_iter().collect()))
            }
            ExprKind::Map(pairs) => {
                let mut m = BTreeMap::new();
                for (k, v) in pairs {
                    let k = self.eval(k)?;
                    let v = self.eval(v)?;
                    m.insert(k, v);
                }
                Ok(Value::Map(Rc::new(m)))
            }
            ExprKind::Record(fields) => {
                let mut m = BTreeMap::new();
                for (k, v) in fields {
                    let v = self.eval(v)?;
                    m.insert(k.clone(), v);
                }
                Ok(Value::Record(Rc::new(m)))
            }
            ExprKind::Builder { head, quals, list } => {
                // `{ w ∈ S* | … }` is a language, never a materialized set.
                if !*list
                    && let Some(Qual::Gen(pat, src)) = quals.first()
                    && matches!(src.kind, ExprKind::Star(_) | ExprKind::Plus(_))
                {
                    return self.predicate_lang(pat, src, quals.get(1..).unwrap_or(&[]), e.span);
                }
                let mut out = Vec::new();
                self.scopes.push(HashMap::new());
                let r = self.builder(head, quals, &mut out);
                self.scopes.pop();
                r?;
                if *list {
                    Ok(Value::list(out))
                } else {
                    Ok(Value::set(out.into_iter().collect()))
                }
            }
            ExprKind::Unary(op, x) => {
                let v = self.eval(x)?;
                self.unary(*op, v, e.span)
            }
            ExprKind::Binary(BinOp::And, l, r) => {
                if !self.eval_bool(l)? {
                    return Ok(Value::Bool(false));
                }
                Ok(Value::Bool(self.eval_bool(r)?))
            }
            ExprKind::Binary(BinOp::Or, l, r) => {
                if self.eval_bool(l)? {
                    return Ok(Value::Bool(true));
                }
                Ok(Value::Bool(self.eval_bool(r)?))
            }
            ExprKind::Binary(op, l, r) => {
                let lv = self.eval(l)?;
                let rv = self.eval(r)?;
                self.binary(*op, lv, rv, e.span)
            }
            ExprKind::Call(callee, args) => self.call_expr(callee, args, e.span),
            ExprKind::Index(x, i) => {
                let v = self.eval(x)?;
                let i = self.eval(i)?;
                self.index(v, i, e.span)
            }
            ExprKind::Slice(x, lo, hi) => {
                let v = self.eval(x)?;
                let lo = match lo {
                    Some(lo) => Some(self.eval_int(lo)?),
                    None => None,
                };
                let hi = match hi {
                    Some(hi) => Some(self.eval_int(hi)?),
                    None => None,
                };
                self.slice(v, lo, hi, e.span)
            }
            ExprKind::Member(x, name) => {
                let v = self.eval(x)?;
                self.member(v, name, e.span)
            }
            ExprKind::Lambda(def) => Ok(self.make_closure(def)),
            ExprKind::Range { lo, hi, inclusive } => {
                let lo = self.eval_int(lo)?;
                let hi = self.eval_int(hi)?;
                let hi = if *inclusive { hi.saturating_add(1) } else { hi };
                let mut out = Vec::new();
                let mut i = lo;
                while i < hi {
                    self.tick(e.span)?;
                    out.push(Value::Int(i));
                    i += 1;
                }
                Ok(Value::list(out))
            }
            ExprKind::Power { base, exp, upto } => {
                let b = self.eval(base)?;
                let n = self.eval_int(exp)?;
                self.power(b, n, *upto, e.span)
            }
            ExprKind::Star(x) => {
                let v = self.eval(x)?;
                self.star(v, e.span, false)
            }
            ExprKind::Plus(x) => {
                let v = self.eval(x)?;
                self.star(v, e.span, true)
            }
            ExprKind::Fixpoint { var, init, body } => {
                let mut x = self.eval(init)?;
                loop {
                    self.tick(e.span)?;
                    self.scopes.push(HashMap::new());
                    self.bind(var, x.clone(), false);
                    let next = self.eval(body);
                    self.scopes.pop();
                    let next = next?;
                    if next == x {
                        return Ok(x);
                    }
                    x = next;
                }
            }
            ExprKind::Machine(m) => self.machine_literal(m, e.span),
            ExprKind::Canonical(c) => self.canonical_literal(c, e.span),
            ExprKind::Explore(x) => self.explore(x, e.span),
            ExprKind::Regex {
                text,
                text_span,
                over,
            } => self.regex_literal(text, *text_span, over.as_deref(), e.span),
            ExprKind::Cfg {
                name,
                body,
                body_span,
            } => self.cfg_literal(name.as_deref(), body, *body_span, e.span),
            ExprKind::Error => err(
                "X999",
                e.span,
                "internal: an expression that failed to parse was run",
            ),
        }
    }

    fn eval_all(&mut self, items: &[Expr]) -> R<Vec<Value>> {
        let mut out = Vec::with_capacity(items.len());
        for i in items {
            out.push(self.eval(i)?);
        }
        Ok(out)
    }

    fn builder(&mut self, head: &Expr, quals: &[Qual], out: &mut Vec<Value>) -> R<()> {
        let Some((first, rest)) = quals.split_first() else {
            out.push(self.eval(head)?);
            return Ok(());
        };
        match first {
            Qual::Cond(c) => {
                if self.eval_bool(c)? {
                    self.builder(head, rest, out)?;
                }
                Ok(())
            }
            Qual::Gen(pat, src) => {
                let items = self.iterate(src)?;
                for item in items {
                    self.tick(src.span)?;
                    if !Self::pattern_matches(pat, &item) {
                        continue;
                    }
                    self.scopes.push(HashMap::new());
                    let r = self
                        .bind_pattern(pat, item, false)
                        .and_then(|_| self.builder(head, rest, out));
                    self.scopes.pop();
                    r?;
                }
                Ok(())
            }
        }
    }

    fn make_closure(&mut self, def: &Rc<FnDef>) -> Value {
        let key = Rc::as_ptr(def) as usize;
        let free = match self.free_cache.get(&key) {
            Some(f) => f.clone(),
            None => {
                let mut names = BTreeSet::new();
                match &def.body {
                    FnBody::Expr(e) => free_in_expr(e, &mut names),
                    FnBody::Block(b) => free_in_block(b, &mut names),
                }
                let f = Rc::new(names.into_iter().collect::<Vec<_>>());
                self.free_cache.insert(key, f.clone());
                f
            }
        };
        let mut captured = Vec::new();
        for name in free.iter() {
            if def.params.iter().any(|p| &p.name == name) {
                continue;
            }
            for scope in self.scopes.iter().rev() {
                if let Some(b) = scope.get(name) {
                    captured.push((name.clone(), b.value.clone()));
                    break;
                }
            }
        }
        let id = self.fresh_id();
        Value::Fn(Rc::new(FnVal::Closure {
            def: def.clone(),
            captured,
            id,
        }))
    }

    fn call_expr(&mut self, callee: &Expr, args: &[Arg], span: Span) -> R<Value> {
        // `obj.method(args)`: evaluate the receiver once and dispatch.
        if let ExprKind::Member(recv, name) = &callee.kind {
            let recv_v = self.eval(recv)?;
            let (pos, named) = self.eval_args(args)?;
            return self.call_method(recv_v, name, pos, named, span);
        }
        let f = self.eval(callee)?;
        let (pos, named) = self.eval_args(args)?;
        self.call_value(&f, pos, named, span)
    }

    fn eval_args(&mut self, args: &[Arg]) -> R<(Args, Named)> {
        let mut pos = Vec::new();
        let mut named = Vec::new();
        for a in args {
            let v = self.eval(&a.value)?;
            match &a.name {
                Some(n) => named.push((n.clone(), v, a.value.span)),
                None => pos.push((v, a.value.span)),
            }
        }
        Ok((pos, named))
    }

    pub fn call_value(
        &mut self,
        f: &Value,
        args: Vec<(Value, Span)>,
        named: Vec<(String, Value, Span)>,
        span: Span,
    ) -> R<Value> {
        let Value::Fn(fv) = f else {
            return self.call_object(f, args, named, span);
        };
        match &**fv {
            FnVal::Builtin { name, .. } => self.call_builtin(name, args, named, span),
            FnVal::Method { recv, name, .. } => {
                self.call_method(recv.clone(), name, args, named, span)
            }
            FnVal::Closure { def, captured, .. } => {
                if let Some((n, _, s)) = named.first() {
                    return err(
                        "E507",
                        *s,
                        format!("`{}` has no named parameter `{n}`", fv.name()),
                    );
                }
                if args.len() != def.params.len() {
                    return err(
                        "E507",
                        span,
                        format!(
                            "`{}` takes {} argument{}, but {} {} given",
                            fv.name(),
                            def.params.len(),
                            if def.params.len() == 1 { "" } else { "s" },
                            args.len(),
                            if args.len() == 1 { "was" } else { "were" }
                        ),
                    );
                }
                self.depth += 1;
                if self.depth > self.budget.depth {
                    self.depth -= 1;
                    return Err(Flow::Budget(Diagnostic::warning(
                        "B603",
                        span,
                        format!(
                            "stopped at recursion depth {} (the depth budget)",
                            self.budget.depth
                        ),
                    )));
                }
                let mut scope = HashMap::new();
                for (n, v) in captured {
                    scope.insert(
                        n.clone(),
                        Binding {
                            value: v.clone(),
                            mutable: false,
                        },
                    );
                }
                if let Some(n) = &def.name {
                    scope.entry(n.clone()).or_insert(Binding {
                        value: f.clone(),
                        mutable: false,
                    });
                }
                for (p, (v, _)) in def.params.iter().zip(args) {
                    scope.insert(
                        p.name.clone(),
                        Binding {
                            value: v,
                            mutable: true,
                        },
                    );
                }
                // `std::mem::replace` swaps in the callee's scopes and hands back the caller's.
                let saved = std::mem::replace(&mut self.scopes, vec![scope]);
                let result = match &def.body {
                    FnBody::Expr(e) => self.eval(e),
                    FnBody::Block(b) => {
                        let mut r = Ok(Value::None);
                        for s in b {
                            if let Err(flow) = self.exec(s) {
                                r = Err(flow);
                                break;
                            }
                        }
                        r
                    }
                };
                self.scopes = saved;
                self.depth -= 1;
                match result {
                    Ok(v) => Ok(v),
                    Err(Flow::Return(v)) => Ok(v),
                    Err(other) => Err(other),
                }
            }
        }
    }

    // ----- operators -----

    fn unary(&mut self, op: UnOp, v: Value, span: Span) -> R<Value> {
        match (op, v) {
            (UnOp::Neg, Value::Int(n)) => match n.checked_neg() {
                Some(m) => Ok(Value::Int(m)),
                None => err("E501", span, "integer overflow (int is 64-bit)"),
            },
            (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnOp::BigUnion, v) => {
                let items = self.items_of(&v, span)?;
                let mut acc: Option<Value> = None;
                for item in items {
                    acc = Some(match acc {
                        None => item,
                        Some(a) => self.binary(BinOp::Union, a, item, span)?,
                    });
                }
                Ok(acc.unwrap_or_else(|| Value::set(BTreeSet::new())))
            }
            (UnOp::Compl, v) => self.complement(v, span),
            (op, v) => err(
                "E504",
                span,
                format!(
                    "`{}` doesn't apply to {} {}",
                    match op {
                        UnOp::Neg => "-",
                        UnOp::Not => "not",
                        UnOp::Compl => "∁",
                        UnOp::BigUnion => "⋃",
                    },
                    v.type_name(),
                    v.display()
                ),
            ),
        }
    }

    pub fn binary(&mut self, op: BinOp, l: Value, r: Value, span: Span) -> R<Value> {
        use Value as V;
        let overflow = || err("E501", span, "integer overflow (int is 64-bit)");
        match (op, &l, &r) {
            (BinOp::Add, V::Int(a), V::Int(b)) => {
                a.checked_add(*b).map(V::Int).map_or_else(overflow, Ok)
            }
            (BinOp::Sub, V::Int(a), V::Int(b)) => {
                a.checked_sub(*b).map(V::Int).map_or_else(overflow, Ok)
            }
            (BinOp::Mul, V::Int(a), V::Int(b)) => {
                a.checked_mul(*b).map(V::Int).map_or_else(overflow, Ok)
            }
            (BinOp::Div | BinOp::Mod, V::Int(_), V::Int(0)) => {
                err("E502", span, "division by zero")
            }
            (BinOp::Div, V::Int(a), V::Int(b)) => a
                .checked_div_euclid(*b)
                .map(V::Int)
                .map_or_else(overflow, Ok),
            (BinOp::Mod, V::Int(a), V::Int(b)) => a
                .checked_rem_euclid(*b)
                .map(V::Int)
                .map_or_else(overflow, Ok),
            (BinOp::Eq, _, _) => Ok(V::Bool(self.values_equal(&l, &r, span)?)),
            (BinOp::Ne, _, _) => Ok(V::Bool(!self.values_equal(&l, &r, span)?)),
            (BinOp::Lt, V::Int(a), V::Int(b)) => Ok(V::Bool(a < b)),
            (BinOp::Le, V::Int(a), V::Int(b)) => Ok(V::Bool(a <= b)),
            (BinOp::Gt, V::Int(a), V::Int(b)) => Ok(V::Bool(a > b)),
            (BinOp::Ge, V::Int(a), V::Int(b)) => Ok(V::Bool(a >= b)),
            (BinOp::Concat, V::Word(a), V::Word(b)) => {
                let mut w = a.to_vec();
                w.extend(b.iter().cloned());
                Ok(V::Word(Rc::new(w)))
            }
            (BinOp::Concat, V::Word(a), V::Sym(s)) => {
                let mut w = a.to_vec();
                w.push(s.clone());
                Ok(V::Word(Rc::new(w)))
            }
            (BinOp::Concat, V::Sym(s), V::Word(b)) => {
                let mut w = vec![s.clone()];
                w.extend(b.iter().cloned());
                Ok(V::Word(Rc::new(w)))
            }
            (BinOp::Concat, V::Sym(a), V::Sym(b)) => {
                Ok(V::Word(Rc::new(vec![a.clone(), b.clone()])))
            }
            (BinOp::Concat, V::List(a), V::List(b)) => {
                let mut v = a.to_vec();
                v.extend(b.iter().cloned());
                Ok(V::list(v))
            }
            (BinOp::In | BinOp::NotIn, _, _) => {
                let m = self.member_of(&l, &r, span)?;
                Ok(V::Bool(if op == BinOp::In { m } else { !m }))
            }
            (BinOp::Subset, V::Set(a), V::Set(b)) => Ok(V::Bool(a.is_subset(b))),
            (BinOp::Union, V::Set(a), V::Set(b)) => {
                let mut s = (**a).clone();
                s.extend(b.iter().cloned());
                Ok(V::set(s))
            }
            (BinOp::Inter, V::Set(a), V::Set(b)) => {
                Ok(V::set(a.intersection(b).cloned().collect()))
            }
            (BinOp::Diff, V::Set(a), V::Set(b)) => Ok(V::set(a.difference(b).cloned().collect())),
            (BinOp::EndsWith, V::Word(a), V::Word(b)) => Ok(V::Bool(a.ends_with(b))),
            (BinOp::StartsWith, V::Word(a), V::Word(b)) => Ok(V::Bool(a.starts_with(b))),
            (BinOp::Contains, V::Word(a), V::Word(b)) => Ok(V::Bool(
                b.is_empty() || a.windows(b.len()).any(|win| win == &b[..]),
            )),
            _ => self.binary_extra(op, l, r, span),
        }
    }

    fn member_of(&mut self, x: &Value, coll: &Value, span: Span) -> R<bool> {
        match coll {
            Value::Set(s) => Ok(s.contains(x)),
            Value::List(l) => Ok(l.contains(x)),
            Value::Map(m) => Ok(m.contains_key(x)),
            other => self.member_of_extra(x, other, span),
        }
    }

    fn index(&mut self, v: Value, i: Value, span: Span) -> R<Value> {
        match (&v, &i) {
            (Value::Map(m), _) => match m.get(&i) {
                Some(x) => Ok(x.clone()),
                None => err("E503", span, format!("the map has no key {}", i.display())),
            },
            (Value::List(items) | Value::Tuple(items), Value::Int(n)) => {
                let len = items.len();
                usize::try_from(*n)
                    .ok()
                    .and_then(|k| items.get(k))
                    .cloned()
                    .map_or_else(
                        || {
                            err(
                                "E503",
                                span,
                                format!(
                                    "index {n} is out of range (length {len}; indexes start at 0)"
                                ),
                            )
                        },
                        Ok,
                    )
            }
            (Value::Word(w), Value::Int(n)) => {
                let len = w.len();
                usize::try_from(*n)
                    .ok()
                    .and_then(|k| w.get(k))
                    .map(|s| Value::Sym(s.clone()))
                    .map_or_else(
                        || {
                            err(
                                "E503",
                                span,
                                format!(
                                    "index {n} is out of range (length {len}; indexes start at 0)"
                                ),
                            )
                        },
                        Ok,
                    )
            }
            _ => err(
                "E504",
                span,
                format!("can't index {} with {}", v.type_name(), i.display()),
            ),
        }
    }

    fn slice(&mut self, v: Value, lo: Option<i64>, hi: Option<i64>, span: Span) -> R<Value> {
        let len = match &v {
            Value::List(l) => l.len(),
            Value::Word(w) => w.len(),
            other => return err("E504", span, format!("can't slice {}", other.type_name())),
        };
        let clamp = |x: i64| -> usize { usize::try_from(x.max(0)).unwrap_or(0).min(len) };
        let a = lo.map(clamp).unwrap_or(0);
        let b = hi.map(clamp).unwrap_or(len).max(a);
        match v {
            Value::List(l) => Ok(Value::list(l.get(a..b).unwrap_or(&[]).to_vec())),
            Value::Word(w) => Ok(Value::Word(Rc::new(w.get(a..b).unwrap_or(&[]).to_vec()))),
            _ => err("E504", span, "can't slice this value"),
        }
    }

    fn power(&mut self, b: Value, n: i64, upto: bool, span: Span) -> R<Value> {
        if n < 0 {
            return err("E504", span, "the exponent must not be negative");
        }
        match b {
            Value::Int(x) if !upto => {
                let e = u32::try_from(n).unwrap_or(u32::MAX);
                x.checked_pow(e)
                    .map(Value::Int)
                    .map_or_else(|| err("E501", span, "integer overflow (int is 64-bit)"), Ok)
            }
            Value::Word(w) if !upto => {
                let mut out = Vec::new();
                for _ in 0..n {
                    self.tick(span)?;
                    out.extend(w.iter().cloned());
                }
                Ok(Value::Word(Rc::new(out)))
            }
            Value::Sym(s) if !upto => {
                let mut out = Vec::new();
                for _ in 0..n {
                    self.tick(span)?;
                    out.push(s.clone());
                }
                Ok(Value::Word(Rc::new(out)))
            }
            Value::Set(alpha) => {
                // Σ^n: all words of length n over the symbols (or words) of the set.
                let parts: Vec<Vec<Sym>> = alpha
                    .iter()
                    .map(|v| match v {
                        Value::Sym(s) => Ok(vec![s.clone()]),
                        Value::Word(w) => Ok(w.to_vec()),
                        other => Err(other.display()),
                    })
                    .collect::<Result<_, _>>()
                    .map_err(|d| {
                        Flow::Error(Diagnostic::error(
                            "E504",
                            span,
                            format!("Σ^n needs a set of symbols or words, found {d}"),
                        ))
                    })?;
                let mut all = BTreeSet::new();
                let mut layer: Vec<Vec<Sym>> = vec![Vec::new()];
                if upto || n == 0 {
                    all.insert(Value::empty_word());
                }
                for k in 1..=n {
                    let mut next = Vec::new();
                    for w in &layer {
                        for p in &parts {
                            self.tick(span)?;
                            let mut x = w.clone();
                            x.extend(p.iter().cloned());
                            next.push(x);
                        }
                    }
                    layer = next;
                    if upto || k == n {
                        for w in &layer {
                            all.insert(Value::Word(Rc::new(w.clone())));
                        }
                    }
                }
                Ok(Value::set(all))
            }
            other => self.power_extra(other, n, upto, span),
        }
    }

    // ----- members -----

    fn member(&mut self, v: Value, name: &str, span: Span) -> R<Value> {
        match &v {
            Value::Record(r) => match r.get(name) {
                Some(x) => Ok(x.clone()),
                None => err("E505", span, format!("the record has no field `{name}`")),
            },
            _ => self.member_extra(v, name, span),
        }
    }

    pub fn value_json(&self, v: &Value) -> serde_json::Value {
        match v {
            Value::Int(n) => json!({ "type": "int", "value": n }),
            Value::Bool(b) => json!({ "type": "bool", "value": b }),
            Value::Set(s) => {
                json!({ "type": "set", "items": s.iter().map(|x| self.value_json(x)).collect::<Vec<_>>(), "text": v.display() })
            }
            Value::List(s) => {
                json!({ "type": "list", "items": s.iter().map(|x| self.value_json(x)).collect::<Vec<_>>(), "text": v.display() })
            }
            Value::Obj(_) => self.object_json(v),
            other => json!({ "type": other.type_name(), "text": other.display() }),
        }
    }
}

// ----- free variables (for closure capture) -----

fn free_in_block(b: &[Stmt], out: &mut BTreeSet<String>) {
    for s in b {
        free_in_stmt(s, out);
    }
}

fn free_in_stmt(s: &Stmt, out: &mut BTreeSet<String>) {
    match &s.kind {
        StmtKind::Let(_, e)
        | StmtKind::Var(_, e)
        | StmtKind::Show(e)
        | StmtKind::Assert(e)
        | StmtKind::Expr(e) => free_in_expr(e, out),
        StmtKind::Assign(n, _, e) => {
            out.insert(n.clone());
            free_in_expr(e, out);
        }
        StmtKind::If(arms, els) => {
            for (c, b) in arms {
                free_in_expr(c, out);
                free_in_block(b, out);
            }
            if let Some(b) = els {
                free_in_block(b, out);
            }
        }
        StmtKind::For(_, e, b) | StmtKind::While(e, b) => {
            free_in_expr(e, out);
            free_in_block(b, out);
        }
        StmtKind::Return(e) => {
            if let Some(e) = e {
                free_in_expr(e, out);
            }
        }
        StmtKind::Print(es) => es.iter().for_each(|e| free_in_expr(e, out)),
        StmtKind::Check(e, k) => {
            free_in_expr(e, out);
            match k {
                CheckKind::Accepts(ws) | CheckKind::Rejects(ws) => {
                    ws.iter().for_each(|e| free_in_expr(e, out))
                }
                CheckKind::Holds(Some(n)) => free_in_expr(n, out),
                CheckKind::Holds(None) => {}
            }
        }
        StmtKind::Run(a, b) | StmtKind::Save(a, b) => {
            free_in_expr(a, out);
            free_in_expr(b, out);
        }
        StmtKind::Fn(def) => match &def.body {
            FnBody::Expr(e) => free_in_expr(e, out),
            FnBody::Block(b) => free_in_block(b, out),
        },
        StmtKind::Lang { value, claim, .. } => {
            free_in_expr(value, out);
            if let Some((_, _, e)) = claim {
                free_in_expr(e, out);
            }
        }
        StmtKind::MachineDecl(_, _, e) => free_in_expr(e, out),
        StmtKind::Block(b) => free_in_block(b, out),
        StmtKind::Budget(pairs) => pairs.iter().for_each(|(_, _, e)| free_in_expr(e, out)),
        StmtKind::Model(_)
        | StmtKind::UseStrict
        | StmtKind::UseNotation(_)
        | StmtKind::UseExplain(_) => {}
    }
}

pub fn free_in_expr(e: &Expr, out: &mut BTreeSet<String>) {
    // `walk_expr` calls the closure on every sub-expression.
    walk_expr(e, &mut |x| {
        if let ExprKind::Var(n) = &x.kind {
            out.insert(n.clone());
        }
    });
}

/// Calls `f` on `e` and every expression inside it (including machine bodies).
pub fn walk_expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    match &e.kind {
        ExprKind::Tuple(xs) | ExprKind::List(xs) | ExprKind::Set(xs) => {
            xs.iter().for_each(|x| walk_expr(x, f))
        }
        ExprKind::Map(ps) => ps.iter().for_each(|(k, v)| {
            walk_expr(k, f);
            walk_expr(v, f);
        }),
        ExprKind::Record(fs) => fs.iter().for_each(|(_, v)| walk_expr(v, f)),
        ExprKind::Builder { head, quals, .. } => {
            walk_expr(head, f);
            for q in quals {
                match q {
                    Qual::Gen(_, s) => walk_expr(s, f),
                    Qual::Cond(c) => walk_expr(c, f),
                }
            }
        }
        ExprKind::Unary(_, x) | ExprKind::Star(x) | ExprKind::Plus(x) | ExprKind::Member(x, _) => {
            walk_expr(x, f)
        }
        ExprKind::Binary(_, a, b)
        | ExprKind::Index(a, b)
        | ExprKind::Range { lo: a, hi: b, .. } => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        ExprKind::Power { base, exp, .. } => {
            walk_expr(base, f);
            walk_expr(exp, f);
        }
        ExprKind::Call(c, args) => {
            walk_expr(c, f);
            args.iter().for_each(|a| walk_expr(&a.value, f));
        }
        ExprKind::Slice(x, a, b) => {
            walk_expr(x, f);
            if let Some(a) = a {
                walk_expr(a, f);
            }
            if let Some(b) = b {
                walk_expr(b, f);
            }
        }
        ExprKind::Lambda(def) => match &def.body {
            FnBody::Expr(x) => walk_expr(x, f),
            FnBody::Block(b) => walk_block(b, f),
        },
        ExprKind::Fixpoint { init, body, .. } => {
            walk_expr(init, f);
            walk_expr(body, f);
        }
        ExprKind::Regex { over: Some(o), .. } => walk_expr(o, f),
        ExprKind::Machine(m) => walk_mitems(&m.items, f),
        ExprKind::Canonical(c) => c.fields.iter().for_each(|(_, _, x)| walk_expr(x, f)),
        ExprKind::Explore(x) => {
            for c in &x.clauses {
                match c {
                    ExploreClause::Alphabet(e) | ExploreClause::Start(e) => walk_expr(e, f),
                    ExploreClause::Step { target, .. } | ExploreClause::Edges { target, .. } => {
                        walk_expr(target, f)
                    }
                    ExploreClause::Accept { cond, .. } => walk_expr(cond, f),
                }
            }
        }
        _ => {}
    }
}

fn walk_block(b: &[Stmt], f: &mut dyn FnMut(&Expr)) {
    for s in b {
        walk_stmt(s, f);
    }
}

fn walk_stmt(s: &Stmt, f: &mut dyn FnMut(&Expr)) {
    match &s.kind {
        StmtKind::Let(_, e)
        | StmtKind::Var(_, e)
        | StmtKind::Show(e)
        | StmtKind::Assert(e)
        | StmtKind::Expr(e) => walk_expr(e, f),
        StmtKind::Assign(n, span, e) => {
            f(&Expr {
                kind: ExprKind::Var(n.clone()),
                span: *span,
            });
            walk_expr(e, f);
        }
        StmtKind::If(arms, els) => {
            for (c, b) in arms {
                walk_expr(c, f);
                walk_block(b, f);
            }
            if let Some(b) = els {
                walk_block(b, f);
            }
        }
        StmtKind::For(_, e, b) | StmtKind::While(e, b) => {
            walk_expr(e, f);
            walk_block(b, f);
        }
        StmtKind::Return(Some(e)) => walk_expr(e, f),
        StmtKind::Print(es) => es.iter().for_each(|e| walk_expr(e, f)),
        StmtKind::Check(e, k) => {
            walk_expr(e, f);
            match k {
                CheckKind::Accepts(ws) | CheckKind::Rejects(ws) => {
                    ws.iter().for_each(|e| walk_expr(e, f))
                }
                CheckKind::Holds(Some(n)) => walk_expr(n, f),
                CheckKind::Holds(None) => {}
            }
        }
        StmtKind::Run(a, b) | StmtKind::Save(a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        StmtKind::Fn(def) => match &def.body {
            FnBody::Expr(e) => walk_expr(e, f),
            FnBody::Block(b) => walk_block(b, f),
        },
        StmtKind::Lang { value, claim, .. } => {
            walk_expr(value, f);
            if let Some((_, _, e)) = claim {
                walk_expr(e, f);
            }
        }
        StmtKind::MachineDecl(_, _, e) => walk_expr(e, f),
        StmtKind::Block(b) => walk_block(b, f),
        StmtKind::Budget(pairs) => pairs.iter().for_each(|(_, _, e)| walk_expr(e, f)),
        _ => {}
    }
}

fn walk_mitems(items: &[MItem], f: &mut dyn FnMut(&Expr)) {
    let state = |s: &MState, f: &mut dyn FnMut(&Expr)| {
        if let MState::Expr(e) = s {
            walk_expr(e, f);
        }
    };
    fn mval(v: &MVal, f: &mut dyn FnMut(&Expr)) {
        match v {
            MVal::Expr(e) => walk_expr(e, f),
            MVal::Set(vs, _) => vs.iter().for_each(|v| mval(v, f)),
            _ => {}
        }
    }
    for it in items {
        match it {
            MItem::Field { values, .. } => values.iter().for_each(|v| mval(v, f)),
            MItem::Edge { from, to, .. } => {
                state(from, f);
                state(to, f);
            }
            MItem::Delta { args, targets, .. } => {
                args.iter().chain(targets.iter()).for_each(|v| mval(v, f));
            }
            MItem::Lambda { state, out, .. } => {
                std::iter::once(state)
                    .chain(out.iter())
                    .for_each(|v| mval(v, f));
            }
            MItem::Table { header, rows, .. } => {
                header.iter().for_each(|v| mval(v, f));
                for (s, cells) in rows {
                    state(s, f);
                    cells.iter().flatten().for_each(|v| mval(v, f));
                }
            }
            MItem::For { iter, body, .. } => {
                walk_expr(iter, f);
                walk_mitems(body, f);
            }
            MItem::If { cond, then, els } => {
                walk_expr(cond, f);
                walk_mitems(then, f);
                walk_mitems(els, f);
            }
        }
    }
}

/// The format a `save` path asks for: its extension, lower-cased (SEMANTICS §10.4).
pub fn save_format(path: &str) -> Option<&'static str> {
    // `rsplit_once` splits at the last `.`; `?` returns None early if there is none.
    let (_, ext) = path.rsplit_once('.')?;
    ["svg", "png", "pdf", "tex", "txt", "json"]
        .into_iter()
        .find(|f| f.eq_ignore_ascii_case(ext))
}

/// A `save` path with `\\` turned into `/`, or None if it is absolute or leaves the folder.
pub fn save_path(path: &str) -> Option<String> {
    let p = path.replace('\\', "/");
    let bytes = p.as_bytes();
    let drive = bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic();
    if p.is_empty() || p.starts_with('/') || drive {
        return None;
    }
    if p.split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(p)
}
