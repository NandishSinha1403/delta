//! Regular expressions (01-LANGUAGE.md §9): the surface syntax, its parser, and lowering to the
//! core regex (`delta_core::fa::regex::Re`) over a sorted alphabet.

use crate::diag::{Diagnostic, Span};
use crate::value::{Object, Sym, TAG_REGEX, sym_text};
use delta_core::fa::regex::Re;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rx {
    Empty,
    Eps,
    Sym(Sym),
    /// `Σ`: any one symbol of the alphabet.
    Any,
    /// `[abc]`
    Class(Vec<Sym>),
    Cat(Vec<Rx>),
    Alt(Vec<Rx>),
    Star(Box<Rx>),
    Plus(Box<Rx>),
    Opt(Box<Rx>),
    Pow(Box<Rx>, u32),
}

impl Rx {
    pub fn symbols(&self, out: &mut BTreeSet<Sym>) {
        match self {
            Rx::Sym(s) => {
                out.insert(s.clone());
            }
            Rx::Class(ss) => out.extend(ss.iter().cloned()),
            Rx::Cat(xs) | Rx::Alt(xs) => xs.iter().for_each(|x| x.symbols(out)),
            Rx::Star(x) | Rx::Plus(x) | Rx::Opt(x) | Rx::Pow(x, _) => x.symbols(out),
            Rx::Empty | Rx::Eps | Rx::Any => {}
        }
    }

    pub fn uses_any(&self) -> bool {
        match self {
            Rx::Any => true,
            Rx::Cat(xs) | Rx::Alt(xs) => xs.iter().any(Rx::uses_any),
            Rx::Star(x) | Rx::Plus(x) | Rx::Opt(x) | Rx::Pow(x, _) => x.uses_any(),
            _ => false,
        }
    }

    /// Lowers to the core regex over `alpha` (sorted); unknown symbols become ∅.
    pub fn lower(&self, alpha: &[Sym]) -> Re {
        let sym = |s: &Sym| {
            alpha
                .binary_search(s)
                .map(|i| Re::Sym(i as u32))
                .unwrap_or(Re::Empty)
        };
        match self {
            Rx::Empty => Re::Empty,
            Rx::Eps => Re::Eps,
            Rx::Sym(s) => sym(s),
            Rx::Any => (0..alpha.len() as u32)
                .map(Re::Sym)
                .fold(Re::Empty, Re::alt),
            Rx::Class(ss) => ss.iter().map(sym).fold(Re::Empty, Re::alt),
            Rx::Cat(xs) => xs.iter().map(|x| x.lower(alpha)).fold(Re::Eps, Re::cat),
            Rx::Alt(xs) => xs.iter().map(|x| x.lower(alpha)).fold(Re::Empty, Re::alt),
            Rx::Star(x) => Re::star(x.lower(alpha)),
            Rx::Plus(x) => {
                let r = x.lower(alpha);
                Re::cat(r.clone(), Re::star(r))
            }
            Rx::Opt(x) => Re::alt(Re::Eps, x.lower(alpha)),
            Rx::Pow(x, n) => {
                let r = x.lower(alpha);
                (0..*n).fold(Re::Eps, |acc, _| Re::cat(acc, r.clone()))
            }
        }
    }

    /// Back from a core regex (for `to_regex`, `simplify`).
    pub fn raise(re: &Re, alpha: &[Sym]) -> Rx {
        match re {
            Re::Empty => Rx::Empty,
            Re::Eps => Rx::Eps,
            Re::Sym(i) => alpha
                .get(*i as usize)
                .map(|s| Rx::Sym(s.clone()))
                .unwrap_or(Rx::Empty),
            Re::Cat(a, b) => {
                let mut v = Vec::new();
                for x in [Rx::raise(a, alpha), Rx::raise(b, alpha)] {
                    match x {
                        Rx::Cat(inner) => v.extend(inner),
                        other => v.push(other),
                    }
                }
                Rx::Cat(v)
            }
            Re::Alt(a, b) => {
                let mut v = Vec::new();
                for x in [Rx::raise(a, alpha), Rx::raise(b, alpha)] {
                    match x {
                        Rx::Alt(inner) => v.extend(inner),
                        other => v.push(other),
                    }
                }
                Rx::Alt(v)
            }
            Re::Star(a) => Rx::Star(Box::new(Rx::raise(a, alpha))),
        }
    }

    /// The canonical text (Unicode operators, spaces between symbols).
    pub fn text(&self) -> String {
        fn go(r: &Rx, prec: u8) -> String {
            match r {
                Rx::Empty => "∅".to_string(),
                Rx::Eps => "ε".to_string(),
                Rx::Sym(s) => sym_text(s),
                Rx::Any => "Σ".to_string(),
                Rx::Class(ss) => {
                    format!("[{}]", ss.iter().map(|s| sym_text(s)).collect::<String>())
                }
                Rx::Star(x) => format!("{}*", go(x, 3)),
                Rx::Plus(x) => format!("{}⁺", go(x, 3)),
                Rx::Opt(x) => format!("{}?", go(x, 3)),
                Rx::Pow(x, n) => format!("{}^{n}", go(x, 3)),
                Rx::Cat(xs) => {
                    let s = xs.iter().map(|x| go(x, 2)).collect::<Vec<_>>().join(" ");
                    if prec > 2 { format!("({s})") } else { s }
                }
                Rx::Alt(xs) => {
                    let s = xs.iter().map(|x| go(x, 1)).collect::<Vec<_>>().join(" ∪ ");
                    if prec > 1 { format!("({s})") } else { s }
                }
            }
        }
        go(self, 0)
    }
}

/// A regex value: the expression and its alphabet (`over S`, or its own symbols).
#[derive(Debug)]
pub struct RegexVal {
    pub rx: Rx,
    pub alphabet: BTreeSet<Sym>,
}

impl RegexVal {
    pub fn alpha_vec(&self) -> Vec<Sym> {
        self.alphabet.iter().cloned().collect()
    }
}

impl Object for RegexVal {
    fn tag(&self) -> u8 {
        TAG_REGEX
    }
    fn key(&self) -> String {
        format!("{}|{:?}", self.rx.text(), self.alphabet)
    }
    fn type_name(&self) -> &'static str {
        "regex"
    }
    fn display(&self) -> String {
        format!("re\"{}\"", self.rx.text())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

struct P<'a> {
    chars: Vec<(usize, char)>,
    i: usize,
    base: usize,
    plus_is_union: bool,
    warnings: &'a mut Vec<Diagnostic>,
}

impl P<'_> {
    fn skip_ws(&mut self) {
        while self
            .chars
            .get(self.i)
            .is_some_and(|(_, c)| c.is_whitespace())
        {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.skip_ws();
        self.chars.get(self.i).map(|(_, c)| *c)
    }
    fn pos(&self) -> usize {
        self.base
            + self.chars.get(self.i).map(|(o, _)| *o).unwrap_or_else(|| {
                self.chars
                    .last()
                    .map(|(o, c)| o + c.len_utf8())
                    .unwrap_or(0)
            })
    }
    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        let p = self.pos();
        Diagnostic::error("E401", Span::new(p, p + 1), msg)
    }

    fn alt(&mut self) -> Result<Rx, Diagnostic> {
        let mut parts = vec![self.cat()?];
        loop {
            match self.peek() {
                Some('∪' | '|') => {
                    self.i += 1;
                    parts.push(self.cat()?);
                }
                Some('+') if self.plus_is_union => {
                    self.i += 1;
                    parts.push(self.cat()?);
                }
                _ => break,
            }
        }
        Ok(if parts.len() == 1 {
            parts.pop().unwrap_or(Rx::Empty)
        } else {
            Rx::Alt(parts)
        })
    }

    fn starts_atom(c: char) -> bool {
        !matches!(
            c,
            ')' | '∪' | '|' | '*' | '⁺' | '?' | '^' | '+' | '∘' | '·' | ']'
        )
    }

    fn cat(&mut self) -> Result<Rx, Diagnostic> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Some('∘' | '·') => {
                    self.i += 1;
                    continue;
                }
                Some(c) if Self::starts_atom(c) => parts.push(self.post()?),
                Some('+') if !self.plus_is_union && parts.is_empty() => {
                    return Err(self.err("`+` needs an expression before it"));
                }
                _ => break,
            }
        }
        if parts.is_empty() {
            return Err(self.err("expected a regex here (use ε for the empty word)"));
        }
        Ok(if parts.len() == 1 {
            parts.pop().unwrap_or(Rx::Eps)
        } else {
            Rx::Cat(parts)
        })
    }

    fn post(&mut self) -> Result<Rx, Diagnostic> {
        let mut r = self.atom()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.i += 1;
                    r = Rx::Star(Box::new(r));
                }
                Some('⁺') => {
                    self.i += 1;
                    r = Rx::Plus(Box::new(r));
                }
                Some('?') => {
                    self.i += 1;
                    r = Rx::Opt(Box::new(r));
                }
                Some('+') if !self.plus_is_union => {
                    // Default profile: `+` is one-or-more. If an operand follows (`a+b`), the
                    // reader may have meant union: warn with both fixes (01 §10).
                    let at = self.pos();
                    self.i += 1;
                    if self.peek().is_some_and(Self::starts_atom) {
                        let span = Span::new(at, at + 1);
                        self.warnings.push(
                            Diagnostic::warning(
                                "W401",
                                span,
                                "`+` here means one-or-more (the default notation); if you meant union, write ∪",
                            )
                            .with_fix("union: write `∪`", span, " ∪ ")
                            .with_fix("one-or-more: write `⁺`", span, "⁺"),
                        );
                    }
                    r = Rx::Plus(Box::new(r));
                }
                Some('^') => {
                    self.i += 1;
                    self.skip_ws();
                    let start = self.i;
                    while self
                        .chars
                        .get(self.i)
                        .is_some_and(|(_, c)| c.is_ascii_digit())
                    {
                        self.i += 1;
                    }
                    let digits: String = self
                        .chars
                        .get(start..self.i)
                        .unwrap_or(&[])
                        .iter()
                        .map(|(_, c)| *c)
                        .collect();
                    let n: u32 = digits
                        .parse()
                        .map_err(|_| self.err("`^` needs a number, like a^3"))?;
                    if n > 1000 {
                        return Err(self.err("the exponent is too large"));
                    }
                    r = Rx::Pow(Box::new(r), n);
                }
                _ => break,
            }
        }
        Ok(r)
    }

    fn atom(&mut self) -> Result<Rx, Diagnostic> {
        let Some(c) = self.peek() else {
            return Err(self.err("the regex ended too early"));
        };
        self.i += 1;
        match c {
            '(' => {
                if self.peek() == Some(')') {
                    self.i += 1;
                    return Ok(Rx::Eps);
                }
                let r = self.alt()?;
                if self.peek() != Some(')') {
                    return Err(self.err("unbalanced `(`: expected `)`"));
                }
                self.i += 1;
                Ok(r)
            }
            ')' => Err(self.err("unbalanced `)`")),
            'ε' | 'λ' => Ok(Rx::Eps),
            '∅' => Ok(Rx::Empty),
            'Σ' => Ok(Rx::Any),
            '[' => {
                let mut syms = Vec::new();
                loop {
                    match self.chars.get(self.i).map(|(_, c)| *c) {
                        None => return Err(self.err("unbalanced `[`: expected `]`")),
                        Some(']') => {
                            self.i += 1;
                            break;
                        }
                        Some(c) if c.is_whitespace() || c == ',' => self.i += 1,
                        Some('<') => syms.push(self.angle()?),
                        Some(c) => {
                            self.i += 1;
                            syms.push(Sym::from(c.to_string().as_str()));
                        }
                    }
                }
                Ok(Rx::Class(syms))
            }
            '<' => {
                self.i -= 1;
                Ok(Rx::Sym(self.angle()?))
            }
            '\\' => match self.chars.get(self.i).map(|(_, c)| *c) {
                Some(e) => {
                    self.i += 1;
                    Ok(Rx::Sym(Sym::from(e.to_string().as_str())))
                }
                None => Err(self.err("`\\` at the end of the regex")),
            },
            c => Ok(Rx::Sym(Sym::from(c.to_string().as_str()))),
        }
    }

    /// `<NAME>`, with the cursor on `<`.
    fn angle(&mut self) -> Result<Sym, Diagnostic> {
        self.i += 1;
        let start = self.i;
        while self.chars.get(self.i).is_some_and(|(_, c)| *c != '>') {
            self.i += 1;
        }
        if self.chars.get(self.i).is_none() {
            return Err(self.err("unterminated `<…>` symbol"));
        }
        let name: String = self
            .chars
            .get(start..self.i)
            .unwrap_or(&[])
            .iter()
            .map(|(_, c)| *c)
            .collect();
        self.i += 1;
        if name.is_empty() {
            return Err(self.err("empty `<>` symbol"));
        }
        Ok(Sym::from(name.as_str()))
    }
}

/// Parses regex text. `base` is the byte offset of the text in the source (for spans).
pub fn parse(
    text: &str,
    base: usize,
    plus_is_union: bool,
    warnings: &mut Vec<Diagnostic>,
) -> Result<Rx, Diagnostic> {
    let mut p = P {
        chars: text.char_indices().collect(),
        i: 0,
        base,
        plus_is_union,
        warnings,
    };
    if p.peek().is_none() {
        return Ok(Rx::Eps);
    }
    let r = p.alt()?;
    if p.peek().is_some() {
        return Err(p.err("unexpected character in the regex (unbalanced `)`?)"));
    }
    Ok(r)
}

pub fn regex_value(rx: Rx, over: Option<BTreeSet<Sym>>) -> Rc<RegexVal> {
    let alphabet = over.unwrap_or_else(|| {
        let mut s = BTreeSet::new();
        rx.symbols(&mut s);
        s
    });
    Rc::new(RegexVal { rx, alphabet })
}
