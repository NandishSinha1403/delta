//! The syntax tree the parser produces. Every node keeps its span so the checker can point at
//! exactly the right characters.

use crate::diag::Span;

/// A name with its position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub text: String,
    pub span: Span,
}

/// What kind of element a set or label item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElemKind {
    /// A word: a state name or a single-character symbol.
    Word,
    /// `<NAME>`
    Angle,
    /// ε
    Eps,
    /// ∅ (only meaningful as a whole set)
    Empty,
}

/// An item inside `{ … }` or `[ … ]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elem {
    pub text: String,
    pub kind: ElemKind,
    pub span: Span,
}

/// `{ a, b, c }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetLit {
    pub items: Vec<Elem>,
    pub span: Span,
    /// Where the closing `}` is (fixes insert new items just before it).
    pub close: Option<Span>,
}

/// `dfa` or `nfa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Dfa,
    Nfa,
}

/// One written δ entry, whichever of the three forms it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaEntry {
    pub from: Ident,
    pub label: Elem,
    /// Target states. `set` is true when written as a set (`{q1, q2}` or `∅`).
    pub to: Vec<Ident>,
    pub set: bool,
    pub span: Span,
}

/// Where δ was written, for fixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaForm {
    Table,
    /// Arrow block; holds the position of its closing `}`.
    Edges(Span),
    Function,
}

/// A field of a machine block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    Q(SetLit),
    Sigma(SetLit),
    Start(Ident),
    Accept(SetLit),
    /// δ entries plus how they were written.
    Delta(Vec<DeltaEntry>, DeltaForm),
}

/// `dfa M1 { … }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineDecl {
    pub kind: Kind,
    pub kind_span: Span,
    pub name: Ident,
    pub partial: bool,
    /// (field, span of its key word) in source order.
    pub fields: Vec<(Field, Span)>,
    pub span: Span,
}

/// A regular expression as written (shorthands kept, so messages can quote them).
#[derive(Debug, Clone, PartialEq, Eq)]
// `Box<Rx>` inside `Rx` makes the type recursive: a Box has a fixed size, so Rust can lay it out.
pub enum Rx {
    Sym(String, Span),
    Eps(Span),
    Empty(Span),
    /// Σ shorthand (Example 1.53)
    Sigma(Span),
    Union(Vec<Rx>),
    Concat(Vec<Rx>),
    Star(Box<Rx>),
    /// R⁺ = RR*
    Plus(Box<Rx>),
    /// Rᵏ
    Pow(Box<Rx>, u32),
}

/// `regex R over {a,b} = …`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexDecl {
    pub name: Ident,
    pub over: Option<SetLit>,
    pub body: Rx,
    pub span: Span,
}

/// A value in a call: `N`, `true`, `3`, `[q2, q1]`, or a nested call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Name(Ident),
    Bool(bool, Span),
    Int(u32, Span),
    List(Vec<Ident>, Span),
    Call(Box<Call>),
}

// `impl` adds methods to a type; `&self` means the method only reads the value.
impl Value {
    /// Where the value is written.
    pub fn span(&self) -> Span {
        match self {
            Value::Name(i) => i.span,
            Value::Bool(_, s) | Value::Int(_, s) | Value::List(_, s) => *s,
            Value::Call(c) => c.span,
        }
    }
}

/// `dfa(N, full: true)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub func: Ident,
    pub args: Vec<Value>,
    pub options: Vec<(Ident, Value)>,
    pub span: Span,
}

/// A string literal with its span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrLit {
    pub text: String,
    pub span: Span,
}

/// One top-level statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Machine(MachineDecl),
    Regex(RegexDecl),
    Let {
        name: Ident,
        call: Call,
        span: Span,
    },
    Check {
        target: Ident,
        accepts: bool,
        inputs: Vec<StrLit>,
        span: Span,
    },
    Run {
        target: Ident,
        input: StrLit,
        span: Span,
    },
}

/// `#mode strict` or `#mode lenient` (docs/plan/08-ERRORS.md §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Strict,
    Lenient,
}

/// A whole `.delta` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
    pub mode: Mode,
}
