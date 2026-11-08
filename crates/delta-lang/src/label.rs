//! Labels: a label is a sequence of operations on a configuration (SEMANTICS.md §4). Each model
//! reads labels in its short form (`a`, `a, X → YZ`, `0 → x, R`, `a / 1`) or in the keyword form
//! (`read a; pop X; push YZ`), and both give the same canonical operation list.

use crate::model::{InputMode, Model, OutputMode};
use crate::value::{Sym, sym_text};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dir {
    L,
    R,
    S,
}

impl Dir {
    fn text(self) -> &'static str {
        match self {
            Dir::L => "L",
            Dir::R => "R",
            Dir::S => "S",
        }
    }
}

/// One operation. Indexes count from 0 (the first stack, tape or counter).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LOp {
    /// Read a symbol: from the input (one-way, two-way) or under tape `t`'s head (on-tape).
    Read(usize, Sym),
    Pop(usize, Sym),
    Push(usize, Vec<Sym>),
    Dequeue(Sym),
    IfZero(usize),
    IfPos(usize),
    Dec(usize),
    Inc(usize),
    Enqueue(Vec<Sym>),
    Write(usize, Sym),
    Move(usize, Dir),
    /// Move the two-way input head.
    MoveInput(Dir),
    Emit(Vec<Sym>),
    /// A regex label (gnfa), as canonical regex text.
    Regex(String),
}

impl LOp {
    /// Position in the canonical order of a label's operations (tests first, then effects).
    fn rank(&self) -> u8 {
        match self {
            LOp::Read(..) => 0,
            LOp::Regex(_) => 0,
            LOp::Pop(..) => 1,
            LOp::Dequeue(_) => 2,
            LOp::IfZero(_) | LOp::IfPos(_) => 3,
            LOp::Dec(_) => 4,
            LOp::Inc(_) => 5,
            LOp::Push(..) => 6,
            LOp::Enqueue(_) => 7,
            LOp::Write(..) => 8,
            LOp::Move(..) | LOp::MoveInput(_) => 9,
            LOp::Emit(_) => 10,
        }
    }
}

/// A label: its operations in canonical order. No operations at all is an ε-move.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label {
    pub ops: Vec<LOp>,
}

impl Label {
    pub fn new(mut ops: Vec<LOp>) -> Label {
        // `sort_by_key` is stable: operations of the same kind keep their written order.
        ops.sort_by_key(LOp::rank);
        Label { ops }
    }

    pub fn eps() -> Label {
        Label { ops: Vec::new() }
    }

    /// The input symbol read (one-way models), if any.
    pub fn read(&self) -> Option<&Sym> {
        self.ops.iter().find_map(|o| match o {
            LOp::Read(0, s) => Some(s),
            _ => None,
        })
    }

    /// For finite automata: is this exactly "read a" or "nothing" (ε)?
    pub fn is_simple(&self) -> bool {
        self.ops.is_empty()
            || (self.ops.len() == 1 && matches!(self.ops.first(), Some(LOp::Read(0, _))))
    }

    /// The label in the model's short form where there is one, else the keyword form.
    pub fn display(&self, m: &Model) -> String {
        let word = |w: &[Sym]| -> String {
            if w.is_empty() {
                "ε".to_string()
            } else {
                w.iter().map(|s| sym_text(s)).collect()
            }
        };
        if let [LOp::Regex(r)] = self.ops.as_slice() {
            return r.clone();
        }
        if m.is_finite_automaton() && self.is_simple() {
            return self
                .read()
                .map(|s| sym_text(s))
                .unwrap_or_else(|| "ε".to_string());
        }
        if m.input == InputMode::OneWay
            && m.storage.len() == 1
            && m.has_stack()
            && m.output == OutputMode::None
        {
            let mut read = None;
            let mut pop = None;
            let mut push = None;
            let mut other = false;
            for o in &self.ops {
                match o {
                    LOp::Read(0, s) => read = Some(s.clone()),
                    LOp::Pop(0, s) => pop = Some(s.clone()),
                    LOp::Push(0, w) => push = Some(w.clone()),
                    _ => other = true,
                }
            }
            if !other {
                let r = read
                    .map(|s| sym_text(&s))
                    .unwrap_or_else(|| "ε".to_string());
                let p = pop.map(|s| sym_text(&s)).unwrap_or_else(|| "ε".to_string());
                let w = push.map(|w| word(&w)).unwrap_or_else(|| "ε".to_string());
                return format!("{r}, {p} → {w}");
            }
        }
        if m.input == InputMode::OnTape && m.tapes() == 1 {
            match self.ops.as_slice() {
                [LOp::Read(0, a), LOp::Write(0, x), LOp::Move(0, d)] => {
                    return format!("{} → {}, {}", sym_text(a), sym_text(x), d.text());
                }
                [LOp::Read(0, a), LOp::Move(0, d)] => {
                    return format!("{} → {}", sym_text(a), d.text());
                }
                _ => {}
            }
        }
        if m.output == OutputMode::PerTransition && m.storage.is_empty() {
            match self.ops.as_slice() {
                [LOp::Read(0, a), LOp::Emit(w)] => return format!("{} / {}", sym_text(a), word(w)),
                [LOp::Read(0, a)] => return sym_text(a),
                [LOp::Emit(w)] => return format!("ε / {}", word(w)),
                _ => {}
            }
        }
        if self.ops.is_empty() {
            return "ε".to_string();
        }
        let parts: Vec<String> = self
            .ops
            .iter()
            .map(|o| match o {
                LOp::Read(t, s) => on(*t, m, format!("read {}", sym_text(s))),
                LOp::Pop(i, s) => format!("pop{} {}", stack_suffix(*i), sym_text(s)),
                LOp::Push(i, w) => format!("push{} {}", stack_suffix(*i), word(w)),
                LOp::Dequeue(s) => format!("dequeue {}", sym_text(s)),
                LOp::Enqueue(w) => format!("enqueue {}", word(w)),
                LOp::IfZero(c) => format!("if c{} == 0", c + 1),
                LOp::IfPos(c) => format!("if c{} > 0", c + 1),
                LOp::Dec(c) => format!("dec c{}", c + 1),
                LOp::Inc(c) => format!("inc c{}", c + 1),
                LOp::Write(t, s) => on(*t, m, format!("write {}", sym_text(s))),
                LOp::Move(t, d) => on(*t, m, format!("move {}", d.text())),
                LOp::MoveInput(d) => format!("move {}", d.text()),
                LOp::Emit(w) => format!("emit {}", word(w)),
                LOp::Regex(r) => r.clone(),
            })
            .collect();
        parts.join("; ")
    }
}

fn stack_suffix(i: usize) -> String {
    if i == 0 {
        String::new()
    } else {
        format!("{}", i + 1)
    }
}

fn on(t: usize, m: &Model, s: String) -> String {
    if m.tapes() > 1 {
        format!("on {}: {s}", t + 1)
    } else {
        s
    }
}

// ----- parsing -----

#[derive(Clone, Debug, PartialEq)]
enum T {
    Arrow,
    Comma,
    Slash,
    Colon,
    Semi,
    Eps,
    Blank,
    Sym(String),
    Word(String),
    Op(String),
}

fn tokenize(text: &str) -> Result<Vec<T>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while let Some(&c) = chars.get(i) {
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let next = chars.get(i + 1).copied();
        match c {
            '→' => {
                out.push(T::Arrow);
                i += 1;
            }
            '-' if next == Some('>') => {
                out.push(T::Arrow);
                i += 2;
            }
            ',' => {
                out.push(T::Comma);
                i += 1;
            }
            '/' => {
                out.push(T::Slash);
                i += 1;
            }
            ':' => {
                out.push(T::Colon);
                i += 1;
            }
            ';' => {
                out.push(T::Semi);
                i += 1;
            }
            'ε' | 'λ' => {
                out.push(T::Eps);
                i += 1;
            }
            '⊔' | '_' => {
                out.push(T::Blank);
                i += 1;
            }
            '=' | '>' | '!' if next == Some('=') => {
                out.push(T::Op(format!("{c}=")));
                i += 2;
            }
            '=' | '>' | '<'
                if c != '<' || !chars.get(i + 1..).is_some_and(|r| r.contains(&'>')) =>
            {
                out.push(T::Op(c.to_string()));
                i += 1;
            }
            '\'' => {
                let rest: String = chars.get(i + 1..).unwrap_or(&[]).iter().collect();
                let end = rest.find('\'').ok_or("unterminated symbol in a label")?;
                let inner = rest.get(..end).unwrap_or("");
                let inner = inner
                    .strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or(inner);
                out.push(T::Sym(inner.to_string()));
                i += 1 + rest.get(..end + 1).map(|s| s.chars().count()).unwrap_or(1);
            }
            '<' => {
                let rest: String = chars.get(i + 1..).unwrap_or(&[]).iter().collect();
                match rest.find('>') {
                    Some(end) if end > 0 => {
                        out.push(T::Sym(rest.get(..end).unwrap_or("").to_string()));
                        i += 2 + rest.get(..end).map(|s| s.chars().count()).unwrap_or(0);
                    }
                    _ => {
                        out.push(T::Op("<".to_string()));
                        i += 1;
                    }
                }
            }
            '"' => {
                let rest: String = chars.get(i + 1..).unwrap_or(&[]).iter().collect();
                let end = rest.find('"').ok_or("unterminated word in a label")?;
                out.push(T::Word(rest.get(..end).unwrap_or("").to_string()));
                i += 2 + rest.get(..end).map(|s| s.chars().count()).unwrap_or(0);
            }
            c if c.is_alphanumeric() => {
                let start = i;
                while chars
                    .get(i)
                    .is_some_and(|c| c.is_alphanumeric() || *c == '\'')
                {
                    i += 1;
                }
                let w: String = chars.get(start..i).unwrap_or(&[]).iter().collect();
                if w == "eps" {
                    out.push(T::Eps);
                } else {
                    out.push(T::Word(w));
                }
            }
            other => {
                out.push(T::Sym(other.to_string()));
                i += 1;
            }
        }
    }
    Ok(out)
}

const KEYWORDS: [&str; 16] = [
    "read", "pop", "push", "write", "move", "emit", "inc", "dec", "if", "enqueue", "dequeue", "on",
    "pop2", "push2", "pop3", "push3",
];

struct P<'a> {
    toks: &'a [T],
    i: usize,
    model: &'a Model,
    blank: Sym,
}

impl P<'_> {
    fn peek(&self) -> Option<&T> {
        self.toks.get(self.i)
    }
    fn peek_at(&self, k: usize) -> Option<&T> {
        self.toks.get(self.i + k)
    }
    fn bump(&mut self) -> Option<T> {
        let t = self.toks.get(self.i).cloned();
        self.i += 1;
        t
    }
    fn eat(&mut self, t: &T) -> bool {
        if self.peek() == Some(t) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// One symbol, or None for ε.
    fn sym_or_eps(&mut self) -> Result<Option<Sym>, String> {
        match self.bump() {
            Some(T::Eps) => Ok(None),
            Some(T::Blank) => Ok(Some(self.blank.clone())),
            Some(T::Sym(s)) => Ok(Some(Sym::from(s.as_str()))),
            Some(T::Word(w)) if w.chars().count() == 1 => Ok(Some(Sym::from(w.as_str()))),
            Some(T::Word(w)) => Err(format!(
                "`{w}` is not one symbol: a symbol is one character, or a name in angle brackets like <{w}>"
            )),
            Some(other) => Err(format!("expected a symbol, found {}", tok_text(&other))),
            None => Err("expected a symbol".to_string()),
        }
    }

    fn sym(&mut self) -> Result<Sym, String> {
        self.sym_or_eps()?
            .ok_or_else(|| "expected a symbol here, not ε".to_string())
    }

    /// A word of symbols (`YZ`, `<A><B>`, `ε`), up to `,` `;` or the end.
    fn word(&mut self) -> Result<Vec<Sym>, String> {
        let mut w = Vec::new();
        while let Some(t) = self.peek() {
            match t {
                T::Eps => {
                    self.bump();
                }
                T::Blank => {
                    self.bump();
                    w.push(self.blank.clone());
                }
                T::Sym(s) => {
                    w.push(Sym::from(s.as_str()));
                    self.bump();
                }
                T::Word(s) => {
                    w.extend(s.chars().map(|c| Sym::from(c.to_string().as_str())));
                    self.bump();
                }
                _ => break,
            }
        }
        Ok(w)
    }

    fn dir(&mut self) -> Result<Dir, String> {
        match self.bump() {
            Some(T::Word(w)) if w == "L" => Ok(Dir::L),
            Some(T::Word(w)) if w == "R" => Ok(Dir::R),
            Some(T::Word(w)) if w == "S" => Ok(Dir::S),
            other => Err(format!(
                "expected a move L, R or S, found {}",
                other
                    .map(|t| tok_text(&t))
                    .unwrap_or_else(|| "nothing".to_string())
            )),
        }
    }

    fn at_keyword(&self) -> bool {
        match (self.peek(), self.peek_at(1)) {
            (Some(T::Word(w)), next) if KEYWORDS.contains(&w.as_str()) => {
                !matches!(next, Some(T::Comma | T::Arrow | T::Slash) | None) || w == "if"
            }
            _ => false,
        }
    }

    fn counter(&mut self) -> Result<usize, String> {
        match self.bump() {
            Some(T::Word(w)) if w.starts_with('c') => w
                .get(1..)
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|n| *n >= 1)
                .map(|n| n - 1)
                .ok_or_else(|| format!("expected a counter c1, c2, …, found `{w}`")),
            _ => Err("expected a counter c1, c2, …".to_string()),
        }
    }

    /// The keyword form: `read a; pop X; push YZ` (clauses until `,` or the end).
    fn keyword_label(&mut self) -> Result<Label, String> {
        let mut ops = Vec::new();
        let mut tape = 0usize;
        loop {
            let Some(T::Word(kw)) = self.bump() else {
                return Err(
                    "expected an operation such as read, pop, push, write or move".to_string(),
                );
            };
            // `pop2`, `push2`: the second stack (stacks are numbered from 1).
            let (kw, stack) = match kw.strip_prefix("pop").or_else(|| kw.strip_prefix("push")) {
                Some(n) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => {
                    let i = n.parse::<usize>().unwrap_or(1).max(1) - 1;
                    (
                        kw.trim_end_matches(|c: char| c.is_ascii_digit())
                            .to_string(),
                        i,
                    )
                }
                _ => (kw, 0),
            };
            match kw.as_str() {
                "on" => {
                    let n = match self.bump() {
                        Some(T::Word(n)) => n
                            .parse::<usize>()
                            .map_err(|_| "expected a tape number after `on`")?,
                        _ => return Err("expected a tape number after `on`".to_string()),
                    };
                    if n == 0 {
                        return Err("tapes are numbered from 1".to_string());
                    }
                    tape = n - 1;
                    self.eat(&T::Colon);
                    continue;
                }
                "read" => {
                    let s = self.sym()?;
                    ops.push(LOp::Read(tape, s));
                }
                "pop" => ops.push(LOp::Pop(stack, self.sym()?)),
                "push" => ops.push(LOp::Push(stack, self.word()?)),
                "write" => ops.push(LOp::Write(tape, self.sym()?)),
                "move" => {
                    let d = self.dir()?;
                    if self.model.input == InputMode::TwoWay {
                        ops.push(LOp::MoveInput(d));
                    } else {
                        ops.push(LOp::Move(tape, d));
                    }
                }
                "emit" => ops.push(LOp::Emit(self.word()?)),
                "enqueue" => ops.push(LOp::Enqueue(self.word()?)),
                "dequeue" => ops.push(LOp::Dequeue(self.sym()?)),
                "inc" => ops.push(LOp::Inc(self.counter()?)),
                "dec" => ops.push(LOp::Dec(self.counter()?)),
                "if" => {
                    let c = self.counter()?;
                    let op = match self.bump() {
                        Some(T::Op(o)) => o,
                        _ => return Err("expected `== 0` or `> 0` after the counter".to_string()),
                    };
                    match self.bump() {
                        Some(T::Word(z)) if z == "0" => {}
                        _ => {
                            return Err(
                                "counter tests compare with 0: `if c1 == 0` or `if c1 > 0`"
                                    .to_string(),
                            );
                        }
                    }
                    match op.as_str() {
                        "==" => ops.push(LOp::IfZero(c)),
                        ">" | "!=" => ops.push(LOp::IfPos(c)),
                        _ => return Err("counter tests are `== 0` or `> 0`".to_string()),
                    }
                }
                other => return Err(format!("unknown operation `{other}`")),
            }
            if !self.eat(&T::Semi) {
                break;
            }
            tape = 0;
        }
        Ok(Label::new(ops))
    }

    /// One label in the model's short form (or the keyword form).
    fn label(&mut self) -> Result<Label, String> {
        if self.at_keyword() {
            return self.keyword_label();
        }
        let m = self.model;
        if m.input == InputMode::OnTape {
            // `0 → x, R` or `x → R` (also `0 → x` with no move).
            let a = self.sym()?;
            if !self.eat(&T::Arrow) {
                return Ok(Label::new(vec![LOp::Read(0, a)]));
            }
            let is_move = matches!(self.peek(), Some(T::Word(w)) if w == "L" || w == "R" || w == "S")
                && self.peek_at(1) != Some(&T::Comma);
            if is_move {
                let d = self.dir()?;
                return Ok(Label::new(vec![LOp::Read(0, a), LOp::Move(0, d)]));
            }
            let x = self.sym()?;
            let mut ops = vec![LOp::Read(0, a), LOp::Write(0, x)];
            if self.eat(&T::Comma) {
                ops.push(LOp::Move(0, self.dir()?));
            }
            return Ok(Label::new(ops));
        }
        if m.input == InputMode::TwoWay {
            // `a → R`: read a, then move.
            let a = self.sym_or_eps()?;
            let mut ops: Vec<LOp> = a.map(|a| LOp::Read(0, a)).into_iter().collect();
            if self.eat(&T::Arrow) {
                ops.push(LOp::MoveInput(self.dir()?));
            }
            return Ok(Label::new(ops));
        }
        if m.has_stack() && m.storage.len() == 1 {
            // `a, X → YZ`
            let a = self.sym_or_eps()?;
            let mut ops: Vec<LOp> = a.map(|a| LOp::Read(0, a)).into_iter().collect();
            let comma_then_arrow = self.peek() == Some(&T::Comma)
                && self
                    .toks
                    .get(self.i + 2..)
                    .is_some_and(|r| r.first() == Some(&T::Arrow));
            if comma_then_arrow {
                self.bump();
                if let Some(x) = self.sym_or_eps()? {
                    ops.push(LOp::Pop(0, x));
                }
                self.bump(); // arrow
                let w = self.word()?;
                if !w.is_empty() {
                    ops.push(LOp::Push(0, w));
                }
            }
            return Ok(Label::new(ops));
        }
        if m.output == OutputMode::PerTransition {
            // `a / 1` (mealy) or `a : xy` (fst)
            let a = self.sym_or_eps()?;
            let mut ops: Vec<LOp> = a.map(|a| LOp::Read(0, a)).into_iter().collect();
            if self.eat(&T::Slash) || self.eat(&T::Colon) {
                let w = self.word()?;
                ops.push(LOp::Emit(w));
            }
            return Ok(Label::new(ops));
        }
        // Finite automata: one symbol or ε.
        let a = self.sym_or_eps()?;
        Ok(Label::new(a.map(|a| LOp::Read(0, a)).into_iter().collect()))
    }
}

fn tok_text(t: &T) -> String {
    match t {
        T::Arrow => "`→`".to_string(),
        T::Comma => "`,`".to_string(),
        T::Slash => "`/`".to_string(),
        T::Colon => "`:`".to_string(),
        T::Semi => "`;`".to_string(),
        T::Eps => "ε".to_string(),
        T::Blank => "⊔".to_string(),
        T::Sym(s) | T::Word(s) | T::Op(s) => format!("`{s}`"),
    }
}

/// Parses the text of an edge into one or more labels (a list separated by `,`).
pub fn parse_labels(text: &str, model: &Model, blank: &Sym) -> Result<Vec<Label>, String> {
    if model.labels_regex {
        return Ok(vec![Label::new(vec![LOp::Regex(text.trim().to_string())])]);
    }
    let toks = tokenize(text)?;
    if toks.is_empty() {
        return Err("an edge needs a label (use ε for a move that reads nothing)".to_string());
    }
    let mut p = P {
        toks: &toks,
        i: 0,
        model,
        blank: blank.clone(),
    };
    let mut out = Vec::new();
    loop {
        out.push(p.label()?);
        if p.i >= toks.len() {
            break;
        }
        if !p.eat(&T::Comma) {
            let rest: Vec<String> = toks
                .get(p.i..)
                .unwrap_or(&[])
                .iter()
                .map(tok_text)
                .collect();
            return Err(format!(
                "unexpected {} in this {} label",
                rest.first().cloned().unwrap_or_default(),
                model.display()
            ));
        }
    }
    Ok(out)
}

/// The label of a `δ(q, a) = p` entry: just "read a" (or ε).
pub fn read_label(a: Option<Sym>) -> Label {
    Label::new(a.map(|a| LOp::Read(0, a)).into_iter().collect())
}

/// The blank symbol (⊔).
pub fn blank() -> Sym {
    Rc::from("⊔")
}
