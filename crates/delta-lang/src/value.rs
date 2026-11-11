//! Values. Every value has a place in one total order (SEMANTICS.md §1.2), used to print and
//! iterate sets and maps, so output never depends on hashing or insertion order.

use crate::ast::FnDef;
use std::cmp::Ordering;
// `BTreeSet` / `BTreeMap` keep their elements sorted by `Ord`: that is the canonical order.
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// A symbol: one character, or a multi-character name like `RESET` (written `<RESET>`).
pub type Sym = Rc<str>;

/// A word: a sequence of symbols.
pub type Word = Rc<Vec<Sym>>;

/// The language classes, in order (01 §6).
// `PartialOrd`/`Ord` derived on an enum order the variants as they are written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    Finite,
    Regular,
    Dcf,
    Cf,
    Cs,
    Decidable,
    Recognizable,
    Any,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Finite => "Finite",
            Class::Regular => "Regular",
            Class::Dcf => "DCF",
            Class::Cf => "CF",
            Class::Cs => "CS",
            Class::Decidable => "Decidable",
            Class::Recognizable => "Recognizable",
            Class::Any => "Any",
        }
    }
    pub fn from_name(s: &str) -> Option<Class> {
        Some(match s {
            "Finite" => Class::Finite,
            "Regular" => Class::Regular,
            "DCF" => Class::Dcf,
            "CF" => Class::Cf,
            "CS" => Class::Cs,
            "Decidable" => Class::Decidable,
            "Recognizable" => Class::Recognizable,
            "Any" => Class::Any,
            _ => return None,
        })
    }
}

/// A function value: a user closure or a built-in.
#[derive(Debug)]
pub enum FnVal {
    Closure {
        def: Rc<FnDef>,
        /// Captured values (closures capture values, not variables: Q-24).
        captured: Vec<(String, Value)>,
        id: u64,
    },
    Builtin {
        name: &'static str,
        id: u64,
    },
    /// A member function bound to its receiver, like `M.accepts`.
    Method {
        recv: Value,
        name: String,
        id: u64,
    },
}

impl FnVal {
    pub fn id(&self) -> u64 {
        match self {
            FnVal::Closure { id, .. } | FnVal::Builtin { id, .. } | FnVal::Method { id, .. } => *id,
        }
    }
    pub fn name(&self) -> String {
        match self {
            FnVal::Closure { def, .. } => def.name.clone().unwrap_or_else(|| "fn".to_string()),
            FnVal::Builtin { name, .. } => name.to_string(),
            FnVal::Method { name, .. } => name.clone(),
        }
    }
}

/// An opaque engine object (machines, languages, regexes, …), compared by a stable key.
pub trait Object: std::fmt::Debug {
    /// The type tag position in the canonical order.
    fn tag(&self) -> u8;
    /// A key for equality and order within the tag.
    fn key(&self) -> String;
    fn type_name(&self) -> &'static str;
    fn display(&self) -> String;
    // `&dyn Any` lets callers downcast to the concrete type.
    fn as_any(&self) -> &dyn std::any::Any;
    /// The shared object as `Rc<dyn Any>`, to recover an `Rc` of the concrete type.
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any>;
}

#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Sym(Sym),
    Atom(Rc<str>),
    Word(Word),
    Tuple(Rc<Vec<Value>>),
    List(Rc<Vec<Value>>),
    Set(Rc<BTreeSet<Value>>),
    Map(Rc<BTreeMap<Value, Value>>),
    Record(Rc<BTreeMap<String, Value>>),
    Class(Class),
    Obj(Rc<dyn Object>),
    Fn(Rc<FnVal>),
}

pub const TAG_REGEX: u8 = 11;
pub const TAG_GRAMMAR: u8 = 12;
pub const TAG_MACHINE: u8 = 13;
pub const TAG_LANG: u8 = 14;
pub const TAG_MODEL: u8 = 15;
pub const TAG_TRACE: u8 = 16;

impl Value {
    /// The type tag (SEMANTICS §1.2): none < bool < int < symbol < atom < word < tuple < list <
    /// set < map < record < regex < grammar < machine < lang < model < (trace, class) < fn.
    fn tag(&self) -> u8 {
        match self {
            Value::None => 0,
            Value::Bool(_) => 1,
            Value::Int(_) => 2,
            Value::Sym(_) => 3,
            Value::Atom(_) => 4,
            Value::Word(_) => 5,
            Value::Tuple(_) => 6,
            Value::List(_) => 7,
            Value::Set(_) => 8,
            Value::Map(_) => 9,
            Value::Record(_) => 10,
            Value::Obj(o) => o.tag(),
            Value::Class(_) => 17,
            Value::Fn(_) => 18,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Sym(_) => "symbol",
            Value::Atom(_) => "atom",
            Value::Word(_) => "word",
            Value::Tuple(_) => "tuple",
            Value::List(_) => "list",
            Value::Set(_) => "set",
            Value::Map(_) => "map",
            Value::Record(_) => "record",
            Value::Class(_) => "class",
            Value::Obj(o) => o.type_name(),
            Value::Fn(_) => "fn",
        }
    }

    pub fn word_from(syms: &[String]) -> Value {
        Value::Word(Rc::new(
            syms.iter().map(|s| Sym::from(s.as_str())).collect(),
        ))
    }

    pub fn empty_word() -> Value {
        Value::Word(Rc::new(Vec::new()))
    }

    pub fn sym(s: &str) -> Value {
        Value::Sym(Sym::from(s))
    }

    pub fn set(items: BTreeSet<Value>) -> Value {
        Value::Set(Rc::new(items))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(items))
    }

    pub fn tuple(items: Vec<Value>) -> Value {
        Value::Tuple(Rc::new(items))
    }

    /// The shared object behind `Obj`, if it has type `T`.
    pub fn rc<T: 'static>(&self) -> Option<Rc<T>> {
        match self {
            Value::Obj(o) => o.clone().into_any().downcast::<T>().ok(),
            _ => None,
        }
    }

    /// The object behind `Obj`, if it has type `T`.
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        // Generic `<T: 'static>`: works for any type that holds no borrowed references.
        match self {
            Value::Obj(o) => o.as_any().downcast_ref::<T>(),
            _ => None,
        }
    }

    /// The text `print` writes: words and symbols bare, everything else as in a set.
    pub fn print_text(&self) -> String {
        match self {
            Value::Word(w) if w.is_empty() => crate::notation::eps(),
            Value::Word(w) => w.iter().map(|s| sym_text(s)).collect(),
            Value::Sym(s) => sym_text(s),
            other => other.display(),
        }
    }

    /// The canonical display: words in quotes, symbols in single quotes, atoms with `:`.
    pub fn display(&self) -> String {
        match self {
            Value::None => "none".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Sym(s) => format!("'{}'", sym_text(s)),
            Value::Atom(a) => format!(":{a}"),
            Value::Word(w) if w.is_empty() => crate::notation::eps(),
            Value::Word(w) => format!("\"{}\"", w.iter().map(|s| sym_text(s)).collect::<String>()),
            Value::Tuple(items) => format!("({})", join(items.iter())),
            Value::List(items) => format!("[{}]", join(items.iter())),
            Value::Set(items) if items.is_empty() => "∅".to_string(),
            Value::Set(items) => format!("{{{}}}", join(items.iter())),
            Value::Map(m) => {
                let parts: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.display(), v.display()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            Value::Record(r) => {
                let parts: Vec<String> = r
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.display()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            Value::Class(c) => c.name().to_string(),
            Value::Obj(o) => o.display(),
            Value::Fn(f) => format!("fn {}", f.name()),
        }
    }
}

/// A symbol as written in a word: multi-character names in angle brackets.
pub fn sym_text(s: &str) -> String {
    // The blank is printed in the profile's spelling (⊔, B or □).
    if s == "⊔" {
        return crate::notation::blank();
    }
    if s.chars().count() > 1 {
        format!("<{s}>")
    } else {
        s.to_string()
    }
}

fn join<'a>(items: impl Iterator<Item = &'a Value>) -> String {
    items.map(|v| v.display()).collect::<Vec<_>>().join(", ")
}

/// Shortlex order on words: shorter first, then symbol by symbol.
pub fn cmp_words(a: &[Sym], b: &[Sym]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

// Implementing `Ord` (and the traits it needs) gives Value the canonical total order.
impl Ord for Value {
    fn cmp(&self, other: &Value) -> Ordering {
        let t = self.tag().cmp(&other.tag());
        if t != Ordering::Equal {
            return t;
        }
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Sym(a), Value::Sym(b)) => a.cmp(b),
            (Value::Atom(a), Value::Atom(b)) => a.cmp(b),
            (Value::Word(a), Value::Word(b)) => cmp_words(a, b),
            (Value::Tuple(a), Value::Tuple(b)) | (Value::List(a), Value::List(b)) => a.cmp(b),
            (Value::Set(a), Value::Set(b)) => a.cmp(b),
            (Value::Map(a), Value::Map(b)) => a.cmp(b),
            (Value::Record(a), Value::Record(b)) => a.cmp(b),
            (Value::Class(a), Value::Class(b)) => a.cmp(b),
            (Value::Obj(a), Value::Obj(b)) => a.key().cmp(&b.key()),
            (Value::Fn(a), Value::Fn(b)) => a.id().cmp(&b.id()),
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Value {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_order_across_types_and_shortlex_words() {
        let mut s = BTreeSet::new();
        s.insert(Value::word_from(&["b".into()]));
        s.insert(Value::word_from(&["a".into(), "a".into()]));
        s.insert(Value::Int(3));
        s.insert(Value::sym("a"));
        s.insert(Value::empty_word());
        let shown: Vec<String> = s.iter().map(|v| v.display()).collect();
        assert_eq!(shown, vec!["3", "'a'", "ε", "\"b\"", "\"aa\""]);
    }

    #[test]
    fn atoms_differ_from_symbols_and_words() {
        assert_ne!(Value::Atom("a".into()), Value::sym("a"));
        assert_ne!(
            Value::Atom("q0".into()),
            Value::word_from(&["q".into(), "0".into()])
        );
    }
}
