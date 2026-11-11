//! The checker: runs before the program and reports what it can know without running it:
//! unknown names (E201), rebinding a `let` (E202) or an undeclared name (E203), argument
//! types against annotations (E204), `Σ*` outside `lang` (E205), wrong argument counts (E206)
//! and calls of model names (E207). Types are inferred where it's easy; `Unknown` never errs.

use crate::ast::*;
use crate::diag::{Diagnostic, Span};
use crate::interp::BUILTINS;
use crate::lang::{self, Decidability, LangOp, Question};
use crate::value::Class;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Unknown,
    Int,
    Bool,
    Word,
    Sym,
    Atom,
    Set,
    List,
    Tuple,
    Map,
    Record,
    Fn,
    Machine,
    Lang,
    Regex,
    Grammar,
    Model,
    None,
}

impl Ty {
    fn name(&self) -> &'static str {
        match self {
            Ty::Unknown => "unknown",
            Ty::Int => "int",
            Ty::Bool => "bool",
            Ty::Word => "word",
            Ty::Sym => "symbol",
            Ty::Atom => "atom",
            Ty::Set => "set",
            Ty::List => "list",
            Ty::Tuple => "tuple",
            Ty::Map => "map",
            Ty::Record => "record",
            Ty::Fn => "fn",
            Ty::Machine => "machine",
            Ty::Lang => "lang",
            Ty::Regex => "regex",
            Ty::Grammar => "grammar",
            Ty::Model => "model",
            Ty::None => "none",
        }
    }

    fn from_type(t: &TypeExpr) -> Ty {
        match t.name.as_str() {
            "int" => Ty::Int,
            "bool" => Ty::Bool,
            "word" => Ty::Word,
            "symbol" => Ty::Sym,
            "atom" => Ty::Atom,
            "set" => Ty::Set,
            "list" => Ty::List,
            "tuple" => Ty::Tuple,
            "map" => Ty::Map,
            "record" => Ty::Record,
            "fn" => Ty::Fn,
            "machine" => Ty::Machine,
            "lang" => Ty::Lang,
            "regex" => Ty::Regex,
            "grammar" => Ty::Grammar,
            "model" => Ty::Model,
            "none" => Ty::None,
            _ => Ty::Unknown,
        }
    }

    fn fits(&self, actual: &Ty) -> bool {
        // A machine, regex or grammar can be used where a lang is expected (Q-27).
        matches!(self, Ty::Unknown)
            || matches!(actual, Ty::Unknown)
            || self == actual
            || (*self == Ty::Lang && matches!(actual, Ty::Machine | Ty::Regex | Ty::Grammar))
    }
}

#[derive(Clone, Debug)]
struct VarInfo {
    mutable: bool,
    ty: Ty,
    /// The language class, when it is known without running.
    class: Option<Class>,
}

/// The class of a prelude model's languages (for checking before running).
fn prelude_class(model: &str) -> Option<Class> {
    Some(match model {
        "dfa" | "nfa" | "gnfa" | "twdfa" | "mealy" | "moore" | "fst" => Class::Regular,
        "dpda" => Class::Dcf,
        "pda" => Class::Cf,
        "lba" => Class::Cs,
        "tm" | "ntm" | "mtm" => Class::Recognizable,
        _ => return None,
    })
}

/// Library functions whose result is a finite automaton.
const FA_RESULTS: [&str; 15] = [
    "to_nfa",
    "determinize",
    "minimize",
    "complement",
    "complete",
    "trim",
    "reverse",
    "remove_epsilon",
    "union",
    "intersect",
    "difference",
    "concat",
    "star",
    "product",
    "to_gnfa",
];

/// What a model name converts with (Q-06: conversions are verbs).
fn conversion_verb(model: &str) -> &'static str {
    match model {
        "dfa" => "determinize",
        "nfa" => "to_nfa",
        "gnfa" => "to_gnfa",
        "pda" => "to_pda",
        _ => "determinize",
    }
}

pub struct Checker<'a> {
    src: &'a str,
    globals: HashMap<String, VarInfo>,
    scopes: Vec<HashMap<String, VarInfo>>,
    fns: HashMap<String, Rc<FnDef>>,
    models: HashSet<String>,
    /// Models with parameters (`mtm(k)`): calling them instantiates them.
    param_models: HashSet<String>,
    /// Inside the subject of `check … up to n`, questions are bounded and always allowed.
    bounded: bool,
    in_lang: u32,
    pub diags: Vec<Diagnostic>,
}

pub fn check(prog: &Program, src: &str, models: &[String]) -> Vec<Diagnostic> {
    let mut c = Checker {
        src,
        globals: HashMap::new(),
        scopes: Vec::new(),
        fns: HashMap::new(),
        models: models.iter().cloned().collect(),
        param_models: ["mtm", "ctr"].iter().map(|s| s.to_string()).collect(),
        bounded: false,
        in_lang: 0,
        diags: Vec::new(),
    };
    // Top-level names are visible everywhere: functions are hoisted, and a function body may
    // use a global defined later (it's looked up when the function runs).
    for s in &prog.stmts {
        match &s.kind {
            StmtKind::Fn(def) => {
                if let Some(n) = &def.name {
                    c.globals.insert(
                        n.clone(),
                        VarInfo {
                            mutable: false,
                            ty: Ty::Fn,
                            class: None,
                        },
                    );
                    c.fns.insert(n.clone(), def.clone());
                }
            }
            StmtKind::Model(m) => {
                c.models.insert(m.name.clone());
                if !m.params.is_empty() {
                    c.param_models.insert(m.name.clone());
                }
            }
            _ => {}
        }
    }
    for s in &prog.stmts {
        c.stmt(s);
    }
    c.diags
}

impl Checker<'_> {
    fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Some(v);
            }
        }
        self.globals.get(name)
    }

    fn known(&self, name: &str) -> bool {
        self.lookup(name).is_some()
            || BUILTINS.contains(&name)
            || Class::from_name(name).is_some()
            || self.models.contains(name)
    }

    fn bind(&mut self, name: &str, mutable: bool, ty: Ty) {
        self.bind_class(name, mutable, ty, None);
    }

    fn bind_class(&mut self, name: &str, mutable: bool, ty: Ty, class: Option<Class>) {
        let info = VarInfo { mutable, ty, class };
        match self.scopes.last_mut() {
            Some(s) => {
                s.insert(name.to_string(), info);
            }
            None => {
                self.globals.insert(name.to_string(), info);
            }
        }
    }

    fn bind_pattern(&mut self, p: &Pattern, ty: Ty) {
        match p {
            Pattern::Var(n, _) => self.bind(n, false, ty),
            _ => {
                let mut names = Vec::new();
                p.names(&mut names);
                for (n, _) in names {
                    self.bind(&n, false, Ty::Unknown);
                }
            }
        }
    }

    fn block(&mut self, b: &[Stmt]) {
        self.scopes.push(HashMap::new());
        for s in b {
            self.stmt(s);
        }
        self.scopes.pop();
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let(p, e) => {
                self.expr(e);
                let ty = self.infer(e);
                let class = self.class_of(e);
                match p {
                    Pattern::Var(n, _) => self.bind_class(n, false, ty, class),
                    _ => self.bind_pattern(p, ty),
                }
            }
            StmtKind::Var(n, e) => {
                self.expr(e);
                let ty = self.infer(e);
                self.bind(n, true, ty);
            }
            StmtKind::Assign(n, span, e) => {
                self.expr(e);
                match self.lookup(n) {
                    Some(v) if !v.mutable => self.diags.push(Diagnostic::error(
                        "E202",
                        *span,
                        format!("`{n}` was declared with `let` and can't be rebound; declare it with `var` instead"),
                    )),
                    Some(_) => {}
                    None => self.diags.push(Diagnostic::error(
                        "E203",
                        *span,
                        format!("`{n}` is not declared; declare it first with `var {n} = …`"),
                    )),
                }
            }
            StmtKind::If(arms, els) => {
                for (c, b) in arms {
                    self.expr(c);
                    self.block(b);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            StmtKind::For(p, e, b) => {
                self.expr(e);
                self.scopes.push(HashMap::new());
                self.bind_pattern(p, Ty::Unknown);
                self.block(b);
                self.scopes.pop();
            }
            StmtKind::While(c, b) => {
                self.expr(c);
                self.block(b);
            }
            StmtKind::Return(Some(e))
            | StmtKind::Show(e)
            | StmtKind::Assert(e)
            | StmtKind::Expr(e) => self.expr(e),
            StmtKind::Return(None) => {}
            StmtKind::Print(es) => es.iter().for_each(|e| self.expr(e)),
            StmtKind::Check(e, k) => {
                self.bounded = matches!(k, CheckKind::Holds(Some(_)));
                self.expr(e);
                self.bounded = false;
                match k {
                    CheckKind::Accepts(ws) | CheckKind::Rejects(ws) => {
                        ws.iter().for_each(|w| self.expr(w))
                    }
                    CheckKind::Holds(Some(n)) => self.expr(n),
                    CheckKind::Holds(None) => {}
                }
            }
            StmtKind::Run(m, w) | StmtKind::Save(m, w, _) => {
                self.expr(m);
                self.expr(w);
            }
            StmtKind::Fn(def) => {
                // `if a && let Some(x) = …` (a let chain) tests both at once.
                if !self.scopes.is_empty()
                    && let Some(n) = &def.name
                {
                    self.bind(n, false, Ty::Fn);
                }
                self.fn_body(def);
            }
            StmtKind::Lang {
                name, value, claim, ..
            } => {
                self.in_lang += 1;
                self.expr(value);
                if let Some((_, _, e)) = claim {
                    self.expr(e);
                }
                self.in_lang -= 1;
                let class = match claim {
                    Some((c, _, _)) => Class::from_name(c),
                    None => self.class_of(value),
                };
                self.bind_class(name, false, Ty::Lang, class);
            }
            StmtKind::MachineDecl(name, _, e) => {
                self.expr(e);
                let ty = self.infer(e);
                let class = self.class_of(e);
                self.bind_class(name, false, ty, class);
            }
            StmtKind::Budget(pairs) => pairs.iter().for_each(|(_, _, e)| self.expr(e)),
            StmtKind::Block(b) => self.block(b),
            StmtKind::Model(_)
            | StmtKind::UseStrict
            | StmtKind::UseNotation(_)
            | StmtKind::UseExplain(_) => {}
        }
    }

    fn fn_body(&mut self, def: &FnDef) {
        self.scopes.push(HashMap::new());
        for p in &def.params {
            let ty = p.ty.as_ref().map(Ty::from_type).unwrap_or(Ty::Unknown);
            self.bind(&p.name, true, ty);
        }
        match &def.body {
            FnBody::Expr(e) => self.expr(e),
            FnBody::Block(b) => {
                for s in b {
                    self.stmt(s);
                }
            }
        }
        self.scopes.pop();
    }

    fn unknown_name(&mut self, name: &str, span: Span) {
        let mut candidates: Vec<String> = BUILTINS.iter().map(|s| s.to_string()).collect();
        candidates.extend(self.globals.keys().cloned());
        for s in &self.scopes {
            candidates.extend(s.keys().cloned());
        }
        let best = candidates
            .iter()
            .map(|c| (edit_distance(name, c), c))
            .filter(|(d, c)| *d <= 2 && *d < name.chars().count().max(2) && c.as_str() != name)
            .min_by_key(|(d, c)| (*d, (*c).clone()));
        let mut d = Diagnostic::error("E201", span, format!("unknown name `{name}`"));
        if let Some((_, c)) = best {
            d.message = format!("unknown name `{name}`; did you mean `{c}`?");
            d = d.with_fix(format!("change to `{c}`"), span, c.clone());
        }
        self.diags.push(d);
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Var(n) => {
                if !self.known(n) {
                    self.unknown_name(n, e.span);
                }
            }
            ExprKind::Call(callee, args) => {
                if let ExprKind::Var(n) = &callee.kind {
                    if self.models.contains(n)
                        && self.lookup(n).is_none()
                        && !self.param_models.contains(n)
                    {
                        let verb = conversion_verb(n);
                        self.diags.push(
                            Diagnostic::error(
                                "E207",
                                callee.span,
                                format!("a model name can't be called: `{n}(…)` is not a conversion; use `{verb}(…)`"),
                            )
                            .with_fix(format!("change to `{verb}`"), callee.span, verb),
                        );
                    } else {
                        self.expr(callee);
                        self.check_call(n, args, e.span);
                    }
                } else {
                    self.expr(callee);
                    if let ExprKind::Member(x, m) = &callee.kind {
                        let q = match m.as_str() {
                            "is_empty" => Some(Question::IsEmpty),
                            "is_finite" => Some(Question::IsFinite),
                            "is_universal" => Some(Question::IsUniversal),
                            _ => None,
                        };
                        if let (Some(q), Some(c)) = (q, self.class_of(x)) {
                            self.question(q, c, None, e.span);
                        }
                    }
                }
                args.iter().for_each(|a| self.expr(&a.value));
            }
            ExprKind::Star(x) | ExprKind::Plus(x) => {
                self.expr(x);
                if self.in_lang == 0 && self.infer(x) == Ty::Set {
                    let what = self
                        .src
                        .get(e.span.from..e.span.to)
                        .unwrap_or("this")
                        .trim()
                        .to_string();
                    self.diags.push(Diagnostic::error(
                        "E205",
                        e.span,
                        format!("`{what}` is an infinite set of words; it's allowed only inside `lang` (use Σ^n or Σ^≤n for a finite set)"),
                    ));
                }
            }
            ExprKind::Lambda(def) => self.fn_body(def),
            ExprKind::Builder { head, quals, .. } => {
                self.scopes.push(HashMap::new());
                for q in quals {
                    match q {
                        Qual::Gen(p, src) => {
                            // `{ w ∈ {a,b}* | … }`: bare names in the alphabet are symbols.
                            let set_items = match &src.kind {
                                ExprKind::Star(inner) | ExprKind::Plus(inner) => {
                                    match &inner.kind {
                                        ExprKind::Set(items) => Some(items),
                                        _ => None,
                                    }
                                }
                                _ => None,
                            };
                            match set_items {
                                Some(items) => {
                                    for i in items {
                                        match &i.kind {
                                            ExprKind::Var(n) if !self.known(n) => {}
                                            _ => self.expr(i),
                                        }
                                    }
                                }
                                None => self.expr(src),
                            }
                            self.bind_pattern(p, Ty::Unknown);
                        }
                        Qual::Cond(c) => self.expr(c),
                    }
                }
                self.expr(head);
                self.scopes.pop();
            }
            ExprKind::Fixpoint { var, init, body } => {
                self.expr(init);
                self.scopes.push(HashMap::new());
                self.bind(var, false, Ty::Unknown);
                self.expr(body);
                self.scopes.pop();
            }
            ExprKind::Machine(m) => self.mitems(&m.items),
            ExprKind::Explore(x) => {
                for c in &x.clauses {
                    match c {
                        ExploreClause::Alphabet(e) | ExploreClause::Start(e) => self.expr(e),
                        ExploreClause::Step {
                            pat, on, target, ..
                        } => {
                            self.scopes.push(HashMap::new());
                            self.bind_pattern(pat, Ty::Unknown);
                            if let StepOn::Sym(n, _) = on {
                                self.bind(n, false, Ty::Sym);
                            }
                            self.expr(target);
                            self.scopes.pop();
                        }
                        ExploreClause::Accept { pat, cond } => {
                            self.scopes.push(HashMap::new());
                            self.bind_pattern(pat, Ty::Unknown);
                            self.expr(cond);
                            self.scopes.pop();
                        }
                        ExploreClause::Edges { pat, target } => {
                            self.scopes.push(HashMap::new());
                            self.bind_pattern(pat, Ty::Unknown);
                            self.expr(target);
                            self.scopes.pop();
                        }
                    }
                }
            }
            ExprKind::Member(x, _) => self.expr(x),
            // manual:begin checker-equiv
            ExprKind::Binary(op @ (BinOp::Equiv | BinOp::Subset), a, b) => {
                self.expr(a);
                self.expr(b);
                if !self.bounded
                    && let (Some(ca), Some(cb)) = (self.class_of(a), self.class_of(b))
                {
                    let q = if *op == BinOp::Equiv {
                        Question::Equiv
                    } else {
                        Question::Subset
                    };
                    self.question(q, ca, Some(cb), e.span);
                }
            }
            // manual:end checker-equiv
            _ => {
                // Every other expression: check its direct sub-expressions.
                for_each_child(e, &mut |c| self.expr(c));
            }
        }
    }

    fn mitems(&mut self, items: &[MItem]) {
        for it in items {
            match it {
                MItem::Field { values, .. } => values.iter().for_each(|v| self.mval(v)),
                MItem::Edge { from, to, .. } => {
                    self.mstate(from);
                    self.mstate(to);
                }
                MItem::Delta { args, targets, .. } => {
                    args.iter().chain(targets).for_each(|v| self.mval(v))
                }
                MItem::Lambda { state, out, .. } => {
                    std::iter::once(state).chain(out).for_each(|v| self.mval(v))
                }
                MItem::Table { header, rows, .. } => {
                    header.iter().for_each(|v| self.mval(v));
                    for (s, cells) in rows {
                        self.mstate(s);
                        cells.iter().flatten().for_each(|v| self.mval(v));
                    }
                }
                MItem::For { pat, iter, body } => {
                    self.expr(iter);
                    self.scopes.push(HashMap::new());
                    self.bind_pattern(pat, Ty::Unknown);
                    self.mitems(body);
                    self.scopes.pop();
                }
                MItem::If { cond, then, els } => {
                    self.expr(cond);
                    self.mitems(then);
                    self.mitems(els);
                }
            }
        }
    }

    fn mstate(&mut self, s: &MState) {
        if let MState::Expr(e) = s {
            self.expr(e);
        }
    }

    fn mval(&mut self, v: &MVal) {
        match v {
            MVal::Expr(e) => self.expr(e),
            MVal::Set(vs, _) => vs.iter().for_each(|v| self.mval(v)),
            _ => {}
        }
    }

    /// Arity and annotated parameter types of a direct call to a top-level function.
    fn check_call(&mut self, name: &str, args: &[Arg], span: Span) {
        if self.scopes.iter().any(|s| s.contains_key(name)) {
            return; // shadowed by a local
        }
        let Some(def) = self.fns.get(name).cloned() else {
            return;
        };
        if args.iter().any(|a| a.name.is_some()) {
            return;
        }
        if args.len() != def.params.len() {
            self.diags.push(Diagnostic::error(
                "E206",
                span,
                format!(
                    "`{name}` takes {} argument{}, but {} {} given",
                    def.params.len(),
                    if def.params.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                ),
            ));
            return;
        }
        for (p, a) in def.params.iter().zip(args) {
            let Some(t) = &p.ty else { continue };
            let want = Ty::from_type(t);
            let got = self.infer(&a.value);
            if !want.fits(&got) {
                self.diags.push(Diagnostic::error(
                    "E204",
                    a.value.span,
                    format!(
                        "`{name}` expects {} `{}` to be {}, but this is {}",
                        "parameter",
                        p.name,
                        article(want.name()),
                        article(got.name())
                    ),
                ));
            }
        }
    }

    // manual:begin checker-question
    /// Reports a question that is undecidable (E701) or not implemented (E702) for its class.
    fn question(&mut self, q: Question, a: Class, b: Option<Class>, span: Span) {
        let joined = b.map(|b| a.max(b)).unwrap_or(a);
        match lang::decidable(q, joined) {
            Decidability::Yes => {}
            Decidability::No => self
                .diags
                .push(crate::hooks::undecidable(q, a, b, span, None)),
            Decidability::NotImplemented => self
                .diags
                .push(crate::hooks::not_implemented(q, joined, span)),
        }
    }
    // manual:end checker-question

    /// The language class of an expression, when it is known without running.
    fn class_of(&self, e: &Expr) -> Option<Class> {
        match &e.kind {
            ExprKind::Var(n) => self.lookup(n).and_then(|v| v.class),
            ExprKind::Machine(m) => prelude_class(&m.model),
            ExprKind::Canonical(m) => prelude_class(&m.model),
            ExprKind::Explore(x) => match &x.model {
                None => Some(Class::Regular),
                Some((m, _)) => prelude_class(m),
            },
            ExprKind::Regex { .. } => Some(Class::Regular),
            ExprKind::Cfg { .. } => Some(Class::Cf),
            ExprKind::Builder {
                quals, list: false, ..
            } if matches!(quals.first(), Some(Qual::Gen(_, s)) if matches!(s.kind, ExprKind::Star(_) | ExprKind::Plus(_))) => {
                Some(Class::Decidable)
            }
            ExprKind::Set(items)
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| matches!(i.kind, ExprKind::Word(_) | ExprKind::Eps)) =>
            {
                Some(Class::Finite)
            }
            ExprKind::Binary(op, a, b) => {
                let op = match op {
                    BinOp::Union => LangOp::Union,
                    BinOp::Inter => LangOp::Inter,
                    BinOp::Diff => LangOp::Diff,
                    BinOp::Concat => LangOp::Concat,
                    _ => return None,
                };
                Some(lang::op_class(
                    op,
                    self.class_of(a)?,
                    Some(self.class_of(b)?),
                ))
            }
            ExprKind::Unary(UnOp::Compl, x) => {
                Some(lang::op_class(LangOp::Compl, self.class_of(x)?, None))
            }
            ExprKind::Star(x) => Some(lang::op_class(LangOp::Star, self.class_of(x)?, None)),
            ExprKind::Plus(x) => Some(lang::op_class(LangOp::Plus, self.class_of(x)?, None)),
            ExprKind::Member(x, m) if m == "lang" => self.class_of(x),
            ExprKind::Call(f, args) => {
                match &f.kind {
                    ExprKind::Var(n)
                        if self.lookup(n).is_none() && FA_RESULTS.contains(&n.as_str()) =>
                    {
                        Some(Class::Regular)
                    }
                    ExprKind::Var(n) if n == "rev" && self.lookup(n).is_none() => Some(
                        lang::op_class(LangOp::Rev, self.class_of(&args.first()?.value)?, None),
                    ),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn infer(&self, e: &Expr) -> Ty {
        match &e.kind {
            ExprKind::Int(_) => Ty::Int,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::None => Ty::None,
            ExprKind::Word(_) | ExprKind::Eps => Ty::Word,
            ExprKind::Sym(_) => Ty::Sym,
            ExprKind::Atom(_) => Ty::Atom,
            ExprKind::Set(_) | ExprKind::EmptySet => Ty::Set,
            ExprKind::Builder { list, .. } => {
                if *list {
                    Ty::List
                } else {
                    Ty::Set
                }
            }
            ExprKind::List(_) | ExprKind::Range { .. } => Ty::List,
            ExprKind::Tuple(_) => Ty::Tuple,
            ExprKind::Map(_) => Ty::Map,
            ExprKind::Record(_) => Ty::Record,
            ExprKind::Lambda(_) => Ty::Fn,
            ExprKind::Machine(_) | ExprKind::Canonical(_) | ExprKind::Explore(_) => Ty::Machine,
            ExprKind::Regex { .. } => Ty::Regex,
            ExprKind::Cfg { .. } => Ty::Grammar,
            ExprKind::Var(n) => self.lookup(n).map(|v| v.ty.clone()).unwrap_or(Ty::Unknown),
            ExprKind::Binary(op, l, r) => {
                let (lt, rt) = (self.infer(l), self.infer(r));
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                        if lt == Ty::Int && rt == Ty::Int =>
                    {
                        Ty::Int
                    }
                    BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::And
                    | BinOp::Or
                    | BinOp::In
                    | BinOp::NotIn
                    | BinOp::Subset
                    | BinOp::Equiv
                    | BinOp::EndsWith
                    | BinOp::StartsWith
                    | BinOp::Contains => Ty::Bool,
                    BinOp::Union | BinOp::Inter | BinOp::Diff if lt == Ty::Set && rt == Ty::Set => {
                        Ty::Set
                    }
                    BinOp::Concat if lt == Ty::Word && rt == Ty::Word => Ty::Word,
                    BinOp::Concat if lt == Ty::List && rt == Ty::List => Ty::List,
                    _ => Ty::Unknown,
                }
            }
            ExprKind::Unary(UnOp::Not, _) => Ty::Bool,
            ExprKind::Call(callee, _) => match &callee.kind {
                ExprKind::Var(n) if !self.scopes.iter().any(|s| s.contains_key(n)) => self
                    .fns
                    .get(n)
                    .and_then(|d| d.ret.as_ref())
                    .map(Ty::from_type)
                    .unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            },
            _ => Ty::Unknown,
        }
    }
}

fn article(name: &str) -> String {
    match name.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => format!("an {name}"),
        _ => format!("a {name}"),
    }
}

/// Calls `f` on each direct sub-expression of `e` (not machine bodies or explore clauses).
fn for_each_child(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    match &e.kind {
        ExprKind::Tuple(xs) | ExprKind::List(xs) | ExprKind::Set(xs) => xs.iter().for_each(&mut *f),
        ExprKind::Map(ps) => ps.iter().for_each(|(k, v)| {
            f(k);
            f(v);
        }),
        ExprKind::Record(fs) => fs.iter().for_each(|(_, v)| f(v)),
        ExprKind::Unary(_, x) => f(x),
        ExprKind::Binary(_, a, b)
        | ExprKind::Index(a, b)
        | ExprKind::Range { lo: a, hi: b, .. } => {
            f(a);
            f(b);
        }
        ExprKind::Power { base, exp, .. } => {
            f(base);
            f(exp);
        }
        ExprKind::Slice(x, a, b) => {
            f(x);
            if let Some(a) = a {
                f(a);
            }
            if let Some(b) = b {
                f(b);
            }
        }
        ExprKind::Regex { over: Some(o), .. } => f(o),
        ExprKind::Canonical(c) => c.fields.iter().for_each(|(_, _, x)| f(x)),
        _ => {}
    }
}

/// Levenshtein distance, for "did you mean".
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev.get(j).copied().unwrap_or(0) + usize::from(ca != cb);
            let del = prev.get(j + 1).copied().unwrap_or(0) + 1;
            let ins = cur.get(j).copied().unwrap_or(0) + 1;
            cur.push(sub.min(del).min(ins));
        }
        prev = cur;
    }
    prev.last().copied().unwrap_or(0)
}
