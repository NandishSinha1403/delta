//! Models: what a machine is made of (01-LANGUAGE.md §4, SEMANTICS.md §5). A model is a record of
//! properties built from a few orthogonal building blocks.

use crate::ast::ModelDecl;
use crate::diag::{Diagnostic, Span};
use crate::value::{Object, TAG_MODEL};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputMode {
    OneWay,
    TwoWay,
    OnTape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StorageKind {
    Stack,
    Tape,
    Queue,
    Counter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AcceptMode {
    Final,
    EmptyStack,
    Halting,
    FinalAndEmpty,
    /// No `accept` property (mealy, moore): treated as `by final` with every state accepting.
    Unstated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputMode {
    None,
    PerTransition,
    PerState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Missing {
    Reject,
    Dead,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub name: String,
    pub params: Vec<String>,
    /// Arguments of an instantiated parameterized model (`mtm(2)`).
    pub args: Vec<i64>,
    pub input: InputMode,
    pub storage: Vec<StorageKind>,
    pub accept: AcceptMode,
    pub output: OutputMode,
    pub deterministic: bool,
    pub total: bool,
    pub epsilon: bool,
    pub labels_regex: bool,
    pub special_form: bool,
    pub bounded: bool,
    pub missing: Missing,
    /// The property phrases as written (kept to instantiate parameters).
    pub props: Vec<Vec<String>>,
    pub extends: Option<String>,
}

impl Model {
    pub fn blank(name: &str) -> Model {
        Model {
            name: name.to_string(),
            params: Vec::new(),
            args: Vec::new(),
            input: InputMode::OneWay,
            storage: Vec::new(),
            accept: AcceptMode::Unstated,
            output: OutputMode::None,
            deterministic: false,
            total: false,
            epsilon: false,
            labels_regex: false,
            special_form: false,
            bounded: false,
            missing: Missing::Reject,
            props: Vec::new(),
            extends: None,
        }
    }

    pub fn display(&self) -> String {
        if self.args.is_empty() {
            if self.params.is_empty() {
                self.name.clone()
            } else {
                format!("{}({})", self.name, self.params.join(", "))
            }
        } else {
            let a: Vec<String> = self.args.iter().map(|n| n.to_string()).collect();
            format!("{}({})", self.name, a.join(", "))
        }
    }

    pub fn has_stack(&self) -> bool {
        self.storage.contains(&StorageKind::Stack)
    }

    pub fn tapes(&self) -> usize {
        self.storage
            .iter()
            .filter(|s| **s == StorageKind::Tape)
            .count()
    }

    pub fn counters(&self) -> usize {
        self.storage
            .iter()
            .filter(|s| **s == StorageKind::Counter)
            .count()
    }

    /// A plain finite automaton: one-way input, no storage, no output, no regex labels.
    pub fn is_finite_automaton(&self) -> bool {
        self.input == InputMode::OneWay
            && self.storage.is_empty()
            && self.output == OutputMode::None
            && !self.labels_regex
    }

    /// Applies one property phrase (`["accept", "by", "final"]`).
    pub fn apply(&mut self, words: &[String], env: &HashMap<String, i64>) -> Result<(), String> {
        let w: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let subst = |s: &str| -> Result<i64, String> {
            // `tapes(k)` → the value of k, or a number.
            let inner = s
                .split_once('(')
                .and_then(|(_, r)| r.strip_suffix(')'))
                .ok_or_else(|| format!("expected a count in parentheses in `{s}`"))?;
            if let Ok(n) = inner.parse::<i64>() {
                return Ok(n);
            }
            env.get(inner)
                .copied()
                .ok_or_else(|| format!("`{inner}` is not a parameter of this model"))
        };
        match w.as_slice() {
            ["input", "one-way"] => self.input = InputMode::OneWay,
            ["input", "two-way"] => self.input = InputMode::TwoWay,
            ["input", "on-tape"] => self.input = InputMode::OnTape,
            ["storage", rest @ ..] => {
                self.storage.clear();
                for s in rest {
                    match *s {
                        "none" => {}
                        "stack" => self.storage.push(StorageKind::Stack),
                        "tape" => self.storage.push(StorageKind::Tape),
                        "queue" => self.storage.push(StorageKind::Queue),
                        s if s.starts_with("tapes(") => {
                            for _ in 0..subst(s)?.max(0) {
                                self.storage.push(StorageKind::Tape);
                            }
                        }
                        s if s.starts_with("counters(") => {
                            for _ in 0..subst(s)?.max(0) {
                                self.storage.push(StorageKind::Counter);
                            }
                        }
                        other => {
                            return Err(format!(
                                "unknown storage `{other}`: use none, stack, tape, tapes(k), queue or counters(k)"
                            ));
                        }
                    }
                }
            }
            ["accept", "by", "final"] => self.accept = AcceptMode::Final,
            ["accept", "by", "empty-stack"] => self.accept = AcceptMode::EmptyStack,
            ["accept", "by", "halting"] => self.accept = AcceptMode::Halting,
            ["accept", "by", "final-and-empty"] => self.accept = AcceptMode::FinalAndEmpty,
            ["output", "none"] => self.output = OutputMode::None,
            ["output", "per-transition"] => self.output = OutputMode::PerTransition,
            ["output", "per-state"] => self.output = OutputMode::PerState,
            ["missing", "reject"] => self.missing = Missing::Reject,
            ["missing", "dead"] => self.missing = Missing::Dead,
            ["missing", "error"] => self.missing = Missing::Error,
            ["labels", "regex"] => self.labels_regex = true,
            ["not", flag] => self.set_flag(flag, false)?,
            [flag] => self.set_flag(flag, true)?,
            _ => return Err(format!("unknown model property `{}`", words.join(" "))),
        }
        Ok(())
    }

    fn set_flag(&mut self, flag: &str, on: bool) -> Result<(), String> {
        match flag {
            "deterministic" => self.deterministic = on,
            "total" => self.total = on,
            "epsilon" => self.epsilon = on,
            "special-form" => self.special_form = on,
            "bounded" => self.bounded = on,
            "labels" if !on => self.labels_regex = false,
            other => return Err(format!("unknown model property `{other}`")),
        }
        Ok(())
    }
}

/// Builds a model from its declaration (parameters left symbolic until instantiated).
pub fn from_decl(
    decl: &ModelDecl,
    models: &HashMap<String, Rc<Model>>,
) -> Result<Model, Diagnostic> {
    let mut m = match &decl.extends {
        Some(base) => match models.get(base) {
            Some(b) => {
                let mut m = (**b).clone();
                m.name = decl.name.clone();
                m.extends = Some(base.clone());
                m
            }
            None => {
                return Err(Diagnostic::error(
                    "E301",
                    decl.span,
                    format!("unknown model `{base}` after `extends`"),
                ));
            }
        },
        None => Model::blank(&decl.name),
    };
    m.params = decl.params.clone();
    let mut props = m.props.clone();
    props.extend(decl.props.iter().map(|(w, _)| w.clone()));
    m.props = props;
    // With parameters, check the phrases with placeholder values (1) so mistakes show early.
    let env: HashMap<String, i64> = decl.params.iter().map(|p| (p.clone(), 1)).collect();
    for (words, span) in &decl.props {
        m.apply(words, &env)
            .map_err(|msg| Diagnostic::error("E301", *span, msg))?;
    }
    Ok(m)
}

/// `mtm(2)`: the model with its parameters bound.
pub fn instantiate(m: &Model, args: &[i64], span: Span) -> Result<Model, Diagnostic> {
    if args.len() != m.params.len() {
        return Err(Diagnostic::error(
            "E507",
            span,
            format!(
                "model `{}` takes {} argument(s), but {} were given",
                m.name,
                m.params.len(),
                args.len()
            ),
        ));
    }
    let env: HashMap<String, i64> = m.params.iter().cloned().zip(args.iter().copied()).collect();
    let mut out = Model::blank(&m.name);
    out.params = m.params.clone();
    out.args = args.to_vec();
    out.extends = m.extends.clone();
    out.props = m.props.clone();
    for words in &m.props {
        out.apply(words, &env)
            .map_err(|msg| Diagnostic::error("E301", span, msg))?;
    }
    Ok(out)
}

/// A model as a value.
#[derive(Debug)]
pub struct ModelVal(pub Rc<Model>);

impl Object for ModelVal {
    fn tag(&self) -> u8 {
        TAG_MODEL
    }
    fn key(&self) -> String {
        format!("{}|{:?}", self.0.display(), self.0.props)
    }
    fn type_name(&self) -> &'static str {
        "model"
    }
    fn display(&self) -> String {
        self.0.display()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}
