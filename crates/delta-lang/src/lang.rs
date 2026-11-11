//! Languages and their classes (01-LANGUAGE.md §6, SEMANTICS.md §7). A `lang` value keeps its
//! source symbolically; membership asks the source, and questions compile it to the cheapest
//! machine that can answer them. The closure table (`classes/closure`) and the decision table
//! (`classes/decision`) are data.

use crate::ast::{Expr, Pattern};
use crate::grammar::Grammar;
use crate::machine::Machine;
use crate::model::{InputMode, Model, StorageKind};
use crate::rx::RegexVal;
use crate::value::{Class, Object, Sym, TAG_LANG, Value};
use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LangOp {
    Union,
    Inter,
    Diff,
    Compl,
    Concat,
    Star,
    Plus,
    Rev,
}

impl LangOp {
    pub fn text(self) -> &'static str {
        match self {
            LangOp::Union => "∪",
            LangOp::Inter => "∩",
            LangOp::Diff => "∖",
            LangOp::Compl => "∁",
            LangOp::Concat => "++",
            LangOp::Star => "*",
            LangOp::Plus => "⁺",
            LangOp::Rev => "rev",
        }
    }
}

#[derive(Debug)]
pub enum Src {
    Finite(BTreeSet<Vec<Sym>>),
    Machine(Rc<Machine>),
    Regex(Rc<RegexVal>),
    Grammar(Rc<Grammar>),
    /// `{ w ∈ S* | cond }`
    Predicate {
        pat: Pattern,
        cond: Rc<Expr>,
        captured: Vec<(String, Value)>,
    },
    Op(LangOp, Vec<Rc<LangVal>>),
    /// `L as c by E`: the value is L; decisions use the evidence E.
    Assumed {
        inner: Rc<LangVal>,
        evidence: Rc<LangVal>,
    },
}

#[derive(Debug)]
pub struct LangVal {
    pub class: Class,
    pub src: Src,
    pub sigma: BTreeSet<Sym>,
    pub assumed: bool,
    pub id: u64,
    /// The compiled finite automaton (for Regular and Finite languages), over `sigma`.
    pub fa: OnceCell<Option<delta_core::fa::Fa>>,
}

impl LangVal {
    pub fn new(class: Class, src: Src, sigma: BTreeSet<Sym>, assumed: bool, id: u64) -> LangVal {
        LangVal {
            class,
            src,
            sigma,
            assumed,
            id,
            fa: OnceCell::new(),
        }
    }

    pub fn describe(&self) -> String {
        match &self.src {
            Src::Finite(ws) => format!("a finite language of {} words", ws.len()),
            Src::Machine(m) => format!("the language of {}", m.display()),
            Src::Regex(r) => format!("the language of {}", r.display()),
            Src::Grammar(_) => "the language of a grammar".to_string(),
            Src::Predicate { .. } => "a language given by a predicate".to_string(),
            Src::Op(op, _) => format!("a {} of languages", op.text()),
            Src::Assumed { .. } => "a language with a claimed class".to_string(),
        }
    }
}

impl Object for LangVal {
    fn tag(&self) -> u8 {
        TAG_LANG
    }
    fn key(&self) -> String {
        format!("{:020}", self.id)
    }
    fn type_name(&self) -> &'static str {
        "lang"
    }
    fn display(&self) -> String {
        let a = if self.assumed { ", assumed" } else { "" };
        format!("lang ({}{a})", self.class.name())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

/// The class of a machine's language, from its model's properties (SEMANTICS §6.4).
pub fn model_class(m: &Model) -> Class {
    let stacks = m
        .storage
        .iter()
        .filter(|s| **s == StorageKind::Stack)
        .count();
    let counters = m.counters();
    let tapes = m.tapes();
    let queue = m.storage.contains(&StorageKind::Queue);
    if m.input == InputMode::OnTape && m.bounded {
        return Class::Cs;
    }
    if m.input == InputMode::OnTape || tapes > 0 || queue || stacks >= 2 || counters >= 2 {
        return Class::Recognizable;
    }
    if m.storage.is_empty() {
        return Class::Regular;
    }
    if stacks == 1 && counters == 0 && m.deterministic {
        return Class::Dcf;
    }
    Class::Cf
}

fn column(row: [Class; 8], c: Class) -> Class {
    let i = match c {
        Class::Finite => 0,
        Class::Regular => 1,
        Class::Dcf => 2,
        Class::Cf => 3,
        Class::Cs => 4,
        Class::Decidable => 5,
        Class::Recognizable => 6,
        Class::Any => 7,
    };
    row.get(i).copied().unwrap_or(Class::Any)
}

use Class::*;
// manual:begin closure-table
const UNION: [Class; 8] = [Finite, Regular, Cf, Cf, Cs, Decidable, Recognizable, Any];
const INTER: [Class; 8] = [Finite, Regular, Cs, Cs, Cs, Decidable, Recognizable, Any];
const INTER_R: [Class; 8] = [Finite, Regular, Dcf, Cf, Cs, Decidable, Recognizable, Any];
const INTER_F: [Class; 8] = [
    Finite,
    Finite,
    Finite,
    Finite,
    Finite,
    Finite,
    Recognizable,
    Recognizable,
];
const COMPL: [Class; 8] = [Regular, Regular, Dcf, Cs, Cs, Decidable, Any, Any];
const DIFF_F: [Class; 8] = [Finite, Finite, Finite, Finite, Finite, Finite, Any, Any];
const CONCAT: [Class; 8] = [Finite, Regular, Cf, Cf, Cs, Decidable, Recognizable, Any];
const STAR: [Class; 8] = [Regular, Regular, Cf, Cf, Cs, Decidable, Recognizable, Any];
const REV: [Class; 8] = [Finite, Regular, Cf, Cf, Cs, Decidable, Recognizable, Any];
// manual:end closure-table

/// The class of `op` applied to operands of these classes (`classes/closure`, SEMANTICS §7.5).
pub fn op_class(op: LangOp, a: Class, b: Option<Class>) -> Class {
    let b = b.unwrap_or(a);
    match op {
        LangOp::Compl => column(COMPL, a),
        LangOp::Star | LangOp::Plus => column(STAR, a),
        LangOp::Rev => column(REV, a),
        LangOp::Inter => {
            if a == Finite || b == Finite {
                let other = if a == Finite { b } else { a };
                column(INTER_F, other)
            } else if a == Regular || b == Regular {
                let other = if a == Regular { b } else { a };
                column(INTER_R, other)
            } else {
                column(INTER, a.max(b))
            }
        }
        LangOp::Diff => {
            if a == Finite {
                column(DIFF_F, b)
            } else {
                op_class(LangOp::Inter, a, Some(column(COMPL, b)))
            }
        }
        LangOp::Union => column(UNION, a.max(b)),
        LangOp::Concat => column(CONCAT, a.max(b)),
    }
}

/// The questions of 01 §6.3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Question {
    IsEmpty,
    IsFinite,
    IsUniversal,
    Equiv,
    Subset,
}

impl Question {
    pub fn text(self) -> &'static str {
        match self {
            Question::IsEmpty => "emptiness",
            Question::IsFinite => "finiteness",
            Question::IsUniversal => "universality",
            Question::Equiv => "equivalence",
            Question::Subset => "inclusion",
        }
    }
}

pub enum Decidability {
    Yes,
    /// Decidable, but the known algorithm isn't implemented.
    NotImplemented,
    No,
}

/// `classes/decision`: whether a question is decidable for a class.
// manual:begin decidable
pub fn decidable(q: Question, c: Class) -> Decidability {
    use Decidability as D;
    match (q, c) {
        (_, Finite | Regular) => D::Yes,
        (Question::IsEmpty | Question::IsFinite, Dcf | Cf) => D::Yes,
        (Question::IsUniversal, Dcf) => D::Yes,
        (Question::Equiv, Dcf) => D::NotImplemented,
        _ => D::No,
    }
}
// manual:end decidable

/// The name of a class in prose (`context-free languages`).
pub fn class_prose(c: Class) -> &'static str {
    match c {
        Finite => "finite languages",
        Regular => "regular languages",
        Dcf => "deterministic context-free languages",
        Cf => "context-free languages",
        Cs => "context-sensitive languages",
        Decidable => "decidable languages",
        Recognizable => "recognizable languages",
        Any => "arbitrary languages",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_table_rules() {
        assert_eq!(op_class(LangOp::Union, Finite, Some(Finite)), Finite);
        assert_eq!(op_class(LangOp::Compl, Finite, None), Regular);
        assert_eq!(op_class(LangOp::Union, Regular, Some(Cf)), Cf);
        assert_eq!(op_class(LangOp::Inter, Finite, Some(Decidable)), Finite);
        assert_eq!(
            op_class(LangOp::Inter, Finite, Some(Recognizable)),
            Recognizable
        );
        assert_eq!(op_class(LangOp::Diff, Finite, Some(Recognizable)), Any);
        assert_eq!(op_class(LangOp::Diff, Cf, Some(Regular)), Cf);
        assert_eq!(op_class(LangOp::Diff, Regular, Some(Cf)), Cs);
        assert_eq!(op_class(LangOp::Inter, Dcf, Some(Regular)), Dcf);
        assert_eq!(op_class(LangOp::Compl, Recognizable, None), Any);
    }
}
