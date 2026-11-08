//! The syntax tree. Machine literals keep their labels as raw text: each model reads its own
//! label grammar (01-LANGUAGE.md §5.2) when the machine is built.

use crate::diag::Span;
// `Rc<T>` is a reference-counted pointer: cheap to clone, shared, immutable.
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    In,
    NotIn,
    Subset,
    Equiv,
    Union,
    Inter,
    Diff,
    EndsWith,
    StartsWith,
    Contains,
}

impl BinOp {
    pub fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Concat => "++",
            BinOp::Eq => "==",
            BinOp::Ne => "≠",
            BinOp::Lt => "<",
            BinOp::Le => "≤",
            BinOp::Gt => ">",
            BinOp::Ge => "≥",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::In => "∈",
            BinOp::NotIn => "∉",
            BinOp::Subset => "⊆",
            BinOp::Equiv => "≡",
            BinOp::Union => "∪",
            BinOp::Inter => "∩",
            BinOp::Diff => "∖",
            BinOp::EndsWith => "ends with",
            BinOp::StartsWith => "starts with",
            BinOp::Contains => "contains",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Compl,
    BigUnion,
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Var(String, Span),
    Wild(Span),
    Tuple(Vec<Pattern>, Span),
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Var(_, s) | Pattern::Wild(s) | Pattern::Tuple(_, s) => *s,
        }
    }
    /// The names this pattern binds, in order.
    pub fn names(&self, out: &mut Vec<(String, Span)>) {
        match self {
            Pattern::Var(n, s) => out.push((n.clone(), *s)),
            Pattern::Wild(_) => {}
            Pattern::Tuple(ps, _) => {
                for p in ps {
                    p.names(out);
                }
            }
        }
    }
}

/// A qualifier of a set-builder: a generator `pattern ∈ expr` or a condition.
#[derive(Clone, Debug)]
pub enum Qual {
    Gen(Pattern, Expr),
    Cond(Expr),
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(i64),
    Bool(bool),
    None,
    Sym(String),
    Word(Vec<String>),
    Atom(String),
    Eps,
    EmptySet,
    Regex {
        text: String,
        text_span: Span,
        over: Option<Box<Expr>>,
    },
    Var(String),
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    Set(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Record(Vec<(String, Expr)>),
    /// `{ head | quals }` (set) or `[ head | quals ]` (list).
    Builder {
        head: Box<Expr>,
        quals: Vec<Qual>,
        list: bool,
    },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Arg>),
    Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>),
    Member(Box<Expr>, String),
    Lambda(Rc<FnDef>),
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
        inclusive: bool,
    },
    /// `b^n`, or `b^≤n` when `upto`.
    Power {
        base: Box<Expr>,
        exp: Box<Expr>,
        upto: bool,
    },
    /// Postfix `L*`.
    Star(Box<Expr>),
    /// Postfix `L⁺`.
    Plus(Box<Expr>),
    Machine(Rc<MachineLit>),
    Canonical(Rc<CanonicalLit>),
    Explore(Rc<ExploreLit>),
    Fixpoint {
        var: String,
        init: Box<Expr>,
        body: Box<Expr>,
    },
    Cfg {
        name: Option<String>,
        body: String,
        body_span: Span,
    },
    /// The name of a class (`Regular`, `CF`, …) used as a value.
    Error,
}

#[derive(Clone, Debug)]
pub struct TypeExpr {
    pub name: String,
    pub args: Vec<TypeExpr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum FnBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Clone, Debug)]
pub struct FnDef {
    pub name: Option<String>,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub body: FnBody,
    pub span: Span,
}

pub type Block = Vec<Stmt>;

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum CheckKind {
    Accepts(Vec<Expr>),
    Rejects(Vec<Expr>),
    /// `check e` or `check e up to n`.
    Holds(Option<Expr>),
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Let(Pattern, Expr),
    Var(String, Expr),
    Assign(String, Span, Expr),
    If(Vec<(Expr, Block)>, Option<Block>),
    For(Pattern, Expr, Block),
    While(Expr, Block),
    Return(Option<Expr>),
    Print(Vec<Expr>),
    Show(Expr),
    Assert(Expr),
    Check(Expr, CheckKind),
    Run(Expr, Expr),
    Expr(Expr),
    Fn(Rc<FnDef>),
    Lang {
        name: String,
        name_span: Span,
        value: Expr,
        claim: Option<(String, Span, Expr)>,
    },
    Model(Rc<ModelDecl>),
    /// `dfa M { … }` at statement level: binds M.
    MachineDecl(String, Span, Expr),
    UseStrict,
    UseNotation(String),
    UseExplain(String),
    Budget(Vec<(String, Span, Expr)>),
    Block(Block),
}

/// A state in a machine body: a bare name (an atom) or `[expr]`.
#[derive(Clone, Debug)]
pub enum MState {
    Name(String, Span),
    Expr(Expr),
}

impl MState {
    pub fn span(&self) -> Span {
        match self {
            MState::Name(_, s) => *s,
            MState::Expr(e) => e.span,
        }
    }
}

/// A value in a field of a machine body (`alphabet 0, 1`, `accept q0, q3`).
#[derive(Clone, Debug)]
pub enum MVal {
    Name(String, Span),
    Sym(String, Span),
    Eps(Span),
    Blank(Span),
    Expr(Expr),
    /// `{ v, … }` inside a field or a δ cell.
    Set(Vec<MVal>, Span),
}

impl MVal {
    pub fn span(&self) -> Span {
        match self {
            MVal::Name(_, s)
            | MVal::Sym(_, s)
            | MVal::Eps(s)
            | MVal::Blank(s)
            | MVal::Set(_, s) => *s,
            MVal::Expr(e) => e.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LabelSrc {
    pub text: String,
    pub bracketed: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum MItem {
    Field {
        name: String,
        name_span: Span,
        values: Vec<MVal>,
    },
    Edge {
        from: MState,
        label: LabelSrc,
        to: MState,
        span: Span,
    },
    /// `δ(q, a) = p` (or a set of targets).
    Delta {
        args: Vec<MVal>,
        targets: Vec<MVal>,
        span: Span,
    },
    /// `δ = table { … }`: header labels, then rows (state, cells).
    Table {
        header: Vec<MVal>,
        rows: Vec<(MState, Vec<Vec<MVal>>)>,
        span: Span,
    },
    For {
        pat: Pattern,
        iter: Expr,
        body: Vec<MItem>,
    },
    If {
        cond: Expr,
        then: Vec<MItem>,
        els: Vec<MItem>,
    },
}

#[derive(Clone, Debug)]
pub struct MachineLit {
    pub model: String,
    pub model_span: Span,
    pub name: Option<String>,
    pub options: Vec<(String, Span)>,
    pub items: Vec<MItem>,
    pub span: Span,
}

/// `machine μ { field expr … }`: the canonical record form (01 §5.6).
#[derive(Clone, Debug)]
pub struct CanonicalLit {
    pub model: String,
    pub model_span: Span,
    pub fields: Vec<(String, Span, Expr)>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum StepOn {
    Sym(String, Span),
    Eps,
}

#[derive(Clone, Debug)]
pub enum ExploreClause {
    Alphabet(Expr),
    Start(Expr),
    Step {
        pat: Pattern,
        on: StepOn,
        any: bool,
        target: Expr,
        span: Span,
    },
    Accept {
        pat: Pattern,
        cond: Expr,
    },
    Edges {
        pat: Pattern,
        target: Expr,
    },
}

#[derive(Clone, Debug)]
pub struct ExploreLit {
    pub model: Option<(String, Span)>,
    pub clauses: Vec<ExploreClause>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModelDecl {
    pub name: String,
    pub params: Vec<String>,
    pub extends: Option<String>,
    /// Property phrases, one per `;` or line, as words (`["accept", "by", "final"]`).
    pub props: Vec<(Vec<String>, Span)>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
