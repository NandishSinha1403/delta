//! The lexer. It has two modes (01-LANGUAGE.md §2): **normal** for ordinary code, and
//! **machine** for the body of a machine literal, where `-a->` and `-[ … ]->` are edge tokens
//! and bare names are atoms. The parser chooses the mode for each token it asks for.

use crate::diag::{Diagnostic, Span};

/// Keywords. Contextual words (`on`, `from`, `as`, `by`, `accepts`, …) stay identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kw {
    Let,
    Var,
    Fn,
    If,
    Else,
    For,
    While,
    Return,
    Print,
    Show,
    /// `save x to "file.svg"` (01 §8).
    Save,
    Assert,
    Check,
    Run,
    Model,
    Machine,
    Lang,
    Use,
    Budget,
    Explore,
    Fixpoint,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Cfg,
    /// `import "m.jff" [as M]`: a JFLAP file (01 §8). Modules across files come in v2.1.
    Import,
    /// Reserved for later versions (01 §16): `choose`, `reduce`.
    Reserved,
}

/// Operators and punctuation. Unicode and ASCII spellings lex to the same operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    Dot,
    DotDot,
    DotDotEq,
    Assign,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    PlusPlus,
    Arrow,
    Bar,
    Question,
    Union,
    Inter,
    SetMinus,
    Compl,
    In,
    NotIn,
    Subset,
    Equiv,
    BigUnion,
    Eps,
    EmptySet,
    Blank,
    SupPlus,
    Concat,
}

/// A token.
#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    /// `'a'`, `'<RESET>'` (stored without the angle brackets), or `<RESET>` in machine mode.
    Sym(String),
    /// `"ab"`: the symbols of the word.
    Word(Vec<String>),
    /// `re"…"`: the raw regex text.
    Regex(String),
    /// `:q0`
    Atom(String),
    Kw(Kw),
    Op(Op),
    /// Machine mode: an edge `-label->` (bracketed = false) or `-[labels]->` (bracketed = true).
    Edge {
        label: String,
        bracketed: bool,
        label_span: Span,
    },
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

/// Which lexing mode to use for the next token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Machine,
}

/// Where the lexer is; restoring it lets the parser re-lex a token in another mode.
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pos: usize,
    prev_operand: bool,
    diag_len: usize,
}

pub struct Lexer<'a> {
    // `&'a str` is a borrowed string; `'a` says it lives as long as the Lexer uses it.
    src: &'a str,
    pos: usize,
    /// Whether the previous token ended an operand (a name, literal, `)`, `]` or `}`). An atom
    /// `:q0` can only start where an operand cannot have just ended.
    prev_operand: bool,
    pub diags: Vec<Diagnostic>,
}

/// ASCII spellings of Greek names (Q-19: they alias each other everywhere).
fn normalize_ident(s: &str) -> String {
    match s {
        "delta" => "δ",
        "Sigma" => "Σ",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        other => other,
    }
    .to_string()
}

fn keyword(s: &str) -> Option<Tok> {
    // `Option<T>` is either `Some(value)` or `None`.
    let kw = match s {
        "let" => Kw::Let,
        "var" => Kw::Var,
        "fn" => Kw::Fn,
        "if" => Kw::If,
        "else" => Kw::Else,
        "for" => Kw::For,
        "while" => Kw::While,
        "return" => Kw::Return,
        "print" => Kw::Print,
        "show" => Kw::Show,
        "save" => Kw::Save,
        "assert" => Kw::Assert,
        "check" => Kw::Check,
        "run" => Kw::Run,
        "model" => Kw::Model,
        "machine" => Kw::Machine,
        "lang" => Kw::Lang,
        "use" => Kw::Use,
        "budget" => Kw::Budget,
        "explore" => Kw::Explore,
        "fixpoint" => Kw::Fixpoint,
        "true" => Kw::True,
        "false" => Kw::False,
        "none" => Kw::None,
        "and" => Kw::And,
        "or" => Kw::Or,
        "not" => Kw::Not,
        "cfg" => Kw::Cfg,
        "import" => Kw::Import,
        "choose" | "reduce" => Kw::Reserved,
        "in" => return Some(Tok::Op(Op::In)),
        "union" => return Some(Tok::Op(Op::Union)),
        "inter" => return Some(Tok::Op(Op::Inter)),
        "minus" => return Some(Tok::Op(Op::SetMinus)),
        "subset" => return Some(Tok::Op(Op::Subset)),
        "equiv" => return Some(Tok::Op(Op::Equiv)),
        "union_all" => return Some(Tok::Op(Op::BigUnion)),
        "eps" | "ε" | "λ" => return Some(Tok::Op(Op::Eps)),
        "empty" => return Some(Tok::Op(Op::EmptySet)),
        _ => return None,
    };
    Some(Tok::Kw(kw))
}

pub fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

pub fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\''
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            src,
            pos: 0,
            prev_operand: false,
            diags: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            pos: self.pos,
            prev_operand: self.prev_operand,
            diag_len: self.diags.len(),
        }
    }

    pub fn restore(&mut self, s: Snapshot) {
        self.pos = s.pos;
        self.prev_operand = s.prev_operand;
        self.diags.truncate(s.diag_len);
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Moves to `pos` (used by the parser to capture raw text such as model and grammar bodies).
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos.min(self.src.len());
    }

    fn peek_char(&self) -> Option<char> {
        self.src.get(self.pos..).and_then(|s| s.chars().next())
    }

    fn peek_char_at(&self, n: usize) -> Option<char> {
        self.src.get(self.pos..).and_then(|s| s.chars().nth(n))
    }

    fn rest(&self) -> &'a str {
        self.src.get(self.pos..).unwrap_or("")
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        // `?` returns early with None if there is no char.
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skips spaces, tabs and comments, but not newlines.
    fn skip_trivia(&mut self) {
        loop {
            let rest = self.rest();
            if rest.starts_with("//") {
                while let Some(c) = self.peek_char() {
                    if c == '\n' {
                        break;
                    }
                    self.bump();
                }
            } else if rest.starts_with("/*") {
                let start = self.pos;
                self.pos += 2;
                let mut depth = 1;
                while depth > 0 {
                    let r = self.rest();
                    if r.is_empty() {
                        self.diags.push(Diagnostic::error(
                            "E003",
                            Span::new(start, start + 2),
                            "unterminated block comment: expected `*/`",
                        ));
                        break;
                    } else if r.starts_with("*/") {
                        depth -= 1;
                        self.pos += 2;
                    } else if r.starts_with("/*") {
                        depth += 1;
                        self.pos += 2;
                    } else {
                        self.bump();
                    }
                }
            } else {
                match self.peek_char() {
                    Some(' ' | '\t' | '\r') => {
                        self.bump();
                    }
                    _ => return,
                }
            }
        }
    }

    /// The next token in the given mode.
    pub fn next(&mut self, mode: Mode) -> Token {
        self.skip_trivia();
        let start = self.pos;
        let tok = match mode {
            Mode::Normal => self.lex_normal(start),
            Mode::Machine => self.lex_machine(start),
        };
        let span = Span::new(start, self.pos);
        self.prev_operand = matches!(
            tok,
            Tok::Ident(_)
                | Tok::Int(_)
                | Tok::Sym(_)
                | Tok::Word(_)
                | Tok::Regex(_)
                | Tok::Atom(_)
                | Tok::Kw(Kw::True | Kw::False | Kw::None)
                | Tok::Op(Op::RParen | Op::RBracket | Op::RBrace | Op::Eps | Op::EmptySet)
        );
        Token { tok, span }
    }

    fn ident_run(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if is_ident_char(c) {
                self.bump();
            } else {
                break;
            }
        }
        self.src.get(start..self.pos).unwrap_or("").to_string()
    }

    fn lex_normal(&mut self, start: usize) -> Tok {
        let Some(c) = self.peek_char() else {
            // `let … else` runs the else block when the pattern does not match.
            return Tok::Eof;
        };
        if c == '\n' {
            self.bump();
            return Tok::Newline;
        }
        if c.is_ascii_digit() {
            return self.number(start);
        }
        if is_ident_start(c) {
            let word = self.ident_run();
            if word == "re" && self.peek_char() == Some('"') {
                return self.regex(start);
            }
            if let Some(t) = keyword(&word) {
                return t;
            }
            return Tok::Ident(normalize_ident(&word));
        }
        if c == '"' {
            return self.word(start);
        }
        if c == '\'' {
            return self.symbol(start);
        }
        if c == ':' {
            let next_is_name = self.peek_char_at(1).is_some_and(is_ident_start);
            if !self.prev_operand && next_is_name {
                self.bump();
                let name = self.ident_run();
                return Tok::Atom(name);
            }
            self.bump();
            return Tok::Op(Op::Colon);
        }
        self.operator(start)
    }

    fn operator(&mut self, start: usize) -> Tok {
        let rest = self.rest();
        // Longest match first.
        let table: [(&str, Op); 50] = [
            ("..=", Op::DotDotEq),
            ("..", Op::DotDot),
            ("->", Op::Arrow),
            ("==", Op::EqEq),
            ("!=", Op::Ne),
            ("<=", Op::Le),
            (">=", Op::Ge),
            ("++", Op::PlusPlus),
            ("(", Op::LParen),
            (")", Op::RParen),
            ("[", Op::LBracket),
            ("]", Op::RBracket),
            ("{", Op::LBrace),
            ("}", Op::RBrace),
            (",", Op::Comma),
            (";", Op::Semi),
            (".", Op::Dot),
            ("=", Op::Assign),
            ("<", Op::Lt),
            (">", Op::Gt),
            ("+", Op::Plus),
            ("-", Op::Minus),
            ("*", Op::Star),
            ("/", Op::Slash),
            ("%", Op::Percent),
            ("^", Op::Caret),
            ("|", Op::Bar),
            ("?", Op::Question),
            ("~", Op::Compl),
            // manual:begin lexer-symbols
            ("∪", Op::Union),
            ("∩", Op::Inter),
            ("∖", Op::SetMinus),
            ("∁", Op::Compl),
            ("∈", Op::In),
            ("∉", Op::NotIn),
            ("⊆", Op::Subset),
            ("≡", Op::Equiv),
            // manual:end lexer-symbols
            ("≠", Op::Ne),
            ("≤", Op::Le),
            ("≥", Op::Ge),
            ("⋃", Op::BigUnion),
            ("→", Op::Arrow),
            ("∅", Op::EmptySet),
            ("⊔", Op::Blank),
            ("□", Op::Blank),
            ("⁺", Op::SupPlus),
            ("·", Op::Concat),
            ("∘", Op::Concat),
            ("⟶", Op::Arrow),
            ("⊂", Op::Subset),
        ];
        for (text, op) in table {
            if rest.starts_with(text) {
                self.pos += text.len();
                return Tok::Op(op);
            }
        }
        // Unknown character: report it and skip it.
        let c = self.bump().unwrap_or(' ');
        self.diags.push(Diagnostic::error(
            "E002",
            Span::new(start, self.pos),
            format!("unexpected character `{c}`"),
        ));
        // Lex the next token instead, so one bad character does not stop the parse.
        self.skip_trivia();
        let s = self.pos;
        self.lex_normal(s)
    }

    fn number(&mut self, start: usize) -> Tok {
        let mut digits = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                digits.push(c);
                self.bump();
            } else if c == '_' && self.peek_char_at(1).is_some_and(|d| d.is_ascii_digit()) {
                self.bump();
            } else {
                break;
            }
        }
        match digits.parse::<i64>() {
            Ok(n) => Tok::Int(n),
            Err(_) => {
                self.diags.push(Diagnostic::error(
                    "E004",
                    Span::new(start, self.pos),
                    "integer literal is too large (int is 64-bit)",
                ));
                Tok::Int(0)
            }
        }
    }

    fn word(&mut self, start: usize) -> Tok {
        self.bump(); // opening quote
        let mut syms = Vec::new();
        loop {
            match self.peek_char() {
                None | Some('\n') => {
                    self.diags.push(Diagnostic::error(
                        "E001",
                        Span::new(start, self.pos),
                        "unterminated word: expected `\"`",
                    ));
                    break;
                }
                Some('"') => {
                    self.bump();
                    break;
                }
                Some('\\') => {
                    self.bump();
                    match self.bump() {
                        Some('n') => syms.push("\n".to_string()),
                        Some('t') => syms.push("\t".to_string()),
                        Some(c) => syms.push(c.to_string()),
                        None => {}
                    }
                }
                Some('<') => {
                    // `<NAME>` is one symbol (Q-17), when it closes on the same line.
                    let rest = self.rest();
                    let close = rest.find('>');
                    let line_end = rest.find(['\n', '"']).unwrap_or(rest.len());
                    match close {
                        Some(k) if k > 1 && k < line_end => {
                            let name = rest.get(1..k).unwrap_or("").to_string();
                            self.pos += k + 1;
                            syms.push(name);
                        }
                        _ => {
                            self.bump();
                            syms.push("<".to_string());
                        }
                    }
                }
                Some(c) => {
                    self.bump();
                    syms.push(c.to_string());
                }
            }
        }
        Tok::Word(syms)
    }

    fn symbol(&mut self, start: usize) -> Tok {
        self.bump(); // opening quote
        let rest = self.rest();
        let end = rest.find(['\'', '\n']).unwrap_or(rest.len());
        let inner = rest.get(..end).unwrap_or("").to_string();
        let closed = rest.get(end..).is_some_and(|r| r.starts_with('\''));
        self.pos += end + usize::from(closed);
        if !closed {
            self.diags.push(Diagnostic::error(
                "E005",
                Span::new(start, self.pos),
                "unterminated symbol: expected `'`",
            ));
            return Tok::Sym(inner);
        }
        let n = inner.chars().count();
        if n == 1 {
            return Tok::Sym(inner);
        }
        if n > 2 && inner.starts_with('<') && inner.ends_with('>') {
            let name = inner.get(1..inner.len() - 1).unwrap_or("").to_string();
            return Tok::Sym(name);
        }
        self.diags.push(Diagnostic::error(
            "E005",
            Span::new(start, self.pos),
            "a symbol is one character, or a name in angle brackets like '<RESET>'",
        ));
        Tok::Sym(inner)
    }

    fn regex(&mut self, start: usize) -> Tok {
        self.bump(); // the quote after `re`
        let content_start = self.pos;
        loop {
            match self.peek_char() {
                None | Some('\n') => {
                    self.diags.push(Diagnostic::error(
                        "E001",
                        Span::new(start, self.pos),
                        "unterminated regex: expected `\"`",
                    ));
                    let text = self
                        .src
                        .get(content_start..self.pos)
                        .unwrap_or("")
                        .to_string();
                    return Tok::Regex(text);
                }
                Some('\\') => {
                    self.bump();
                    self.bump();
                }
                Some('"') => {
                    let text = self
                        .src
                        .get(content_start..self.pos)
                        .unwrap_or("")
                        .to_string();
                    self.bump();
                    return Tok::Regex(text);
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    fn lex_machine(&mut self, start: usize) -> Tok {
        let Some(c) = self.peek_char() else {
            return Tok::Eof;
        };
        match c {
            '\n' => {
                self.bump();
                Tok::Newline
            }
            '-' if self.rest().starts_with("->") => {
                self.pos += 2;
                Tok::Op(Op::Arrow)
            }
            '-' => self.edge(start),
            '\'' => self.symbol(start),
            '<' => {
                let rest = self.rest();
                match rest.find('>') {
                    Some(k)
                        if k > 1 && !rest.get(..k).unwrap_or("").contains(char::is_whitespace) =>
                    {
                        let name = rest.get(1..k).unwrap_or("").to_string();
                        self.pos += k + 1;
                        Tok::Sym(name)
                    }
                    _ => {
                        self.bump();
                        Tok::Op(Op::Lt)
                    }
                }
            }
            c if is_ident_char(c) && c != '\'' => {
                let word = self.ident_run();
                match word.as_str() {
                    "eps" | "ε" | "λ" => Tok::Op(Op::Eps),
                    _ => Tok::Ident(normalize_ident(&word)),
                }
            }
            _ => {
                let single = [
                    ('{', Op::LBrace),
                    ('}', Op::RBrace),
                    ('(', Op::LParen),
                    (')', Op::RParen),
                    ('[', Op::LBracket),
                    (']', Op::RBracket),
                    (',', Op::Comma),
                    (';', Op::Semi),
                    (':', Op::Colon),
                    ('=', Op::Assign),
                    ('|', Op::Bar),
                    ('→', Op::Arrow),
                    ('⊔', Op::Blank),
                    ('□', Op::Blank),
                    ('∅', Op::EmptySet),
                    ('∈', Op::In),
                ];
                for (ch, op) in single {
                    if c == ch {
                        self.bump();
                        return Tok::Op(op);
                    }
                }
                self.bump();
                self.diags.push(Diagnostic::error(
                    "E002",
                    Span::new(start, self.pos),
                    format!("unexpected character `{c}` in a machine body"),
                ));
                self.skip_trivia();
                let s = self.pos;
                self.lex_machine(s)
            }
        }
    }

    /// An edge token, starting at `-`.
    fn edge(&mut self, start: usize) -> Tok {
        self.bump(); // '-'
        if self.peek_char() == Some('[') {
            self.bump();
            let label_start = self.pos;
            let mut depth = 1usize;
            loop {
                match self.peek_char() {
                    None | Some('\n') => {
                        self.diags.push(Diagnostic::error(
                            "E104",
                            Span::new(start, self.pos),
                            "unbalanced brackets in an edge: expected `]->`",
                        ));
                        let label = self
                            .src
                            .get(label_start..self.pos)
                            .unwrap_or("")
                            .to_string();
                        return Tok::Edge {
                            label,
                            bracketed: true,
                            label_span: Span::new(label_start, self.pos),
                        };
                    }
                    Some('[') => {
                        depth += 1;
                        self.bump();
                    }
                    Some(']') => {
                        depth -= 1;
                        if depth == 0 && self.rest().starts_with("]->") {
                            let label = self
                                .src
                                .get(label_start..self.pos)
                                .unwrap_or("")
                                .to_string();
                            let label_span = Span::new(label_start, self.pos);
                            self.pos += 3;
                            return Tok::Edge {
                                label,
                                bracketed: true,
                                label_span,
                            };
                        }
                        if depth == 0 {
                            self.diags.push(Diagnostic::error(
                                "E104",
                                Span::new(self.pos, self.pos + 1),
                                "unbalanced brackets in an edge: `]` here does not close it with `]->`",
                            ));
                            depth = 1;
                        }
                        self.bump();
                    }
                    Some(_) => {
                        self.bump();
                    }
                }
            }
        }
        // A bare label runs to the first `->` on the line.
        let label_start = self.pos;
        let rest = self.rest();
        let line = rest
            .get(..rest.find('\n').unwrap_or(rest.len()))
            .unwrap_or("");
        let Some(first) = line.find("->") else {
            self.pos += line.len();
            self.diags.push(Diagnostic::error(
                "E105",
                Span::new(start, self.pos),
                "unterminated edge: expected `->` before the target state",
            ));
            return Tok::Edge {
                label: line.trim().to_string(),
                bracketed: false,
                label_span: Span::new(label_start, self.pos),
            };
        };
        // If another `->` follows with no new edge in between (`-a, X -> YZ-> q1`), the label
        // contains an ASCII arrow: that needs brackets (E103).
        let after = line.get(first + 2..).unwrap_or("");
        if let Some(second) = after.find("->") {
            let between = after.get(..second).unwrap_or("");
            if !between.contains('-') && !between.trim().is_empty() {
                let label_end = first + 2 + second;
                let label = line.get(..label_end).unwrap_or("").to_string();
                let label_span = Span::new(label_start, label_start + label_end);
                self.pos = label_start + label_end + 2;
                let edge_span = Span::new(start, self.pos);
                let fixed = format!("-[{}]->", label.trim());
                self.diags.push(
                    Diagnostic::error(
                        "E103",
                        edge_span,
                        "this label contains an ASCII `->`, which would end the edge; put the label in brackets",
                    )
                    .with_fix(format!("add brackets: `{fixed}`"), edge_span, fixed.clone()),
                );
                return Tok::Edge {
                    label,
                    bracketed: true,
                    label_span,
                };
            }
        }
        let label = line.get(..first).unwrap_or("").to_string();
        let label_span = Span::new(label_start, label_start + first);
        self.pos = label_start + first + 2;
        Tok::Edge {
            label,
            bracketed: false,
            label_span,
        }
    }

    pub fn source(&self) -> &'a str {
        self.src
    }

    /// The raw text up to the `)` matching an already-consumed `(`; moves past the `)`.
    pub fn raw_parens(&mut self) -> Option<(String, Span)> {
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(c) = self.peek_char() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        let text = self.src.get(start..self.pos).unwrap_or("").to_string();
                        let span = Span::new(start, self.pos);
                        self.bump();
                        self.prev_operand = true;
                        return Some((text, span));
                    }
                }
                '\n' => return None,
                _ => {}
            }
            self.bump();
        }
        None
    }

    /// The raw text from the current position to the `}` matching an already-consumed `{`.
    /// Returns the text and its span, and moves past the `}`.
    pub fn raw_block(&mut self) -> Option<(String, Span)> {
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(c) = self.peek_char() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let text = self.src.get(start..self.pos).unwrap_or("").to_string();
                        let span = Span::new(start, self.pos);
                        self.bump();
                        self.prev_operand = true;
                        return Some((text, span));
                    }
                }
                '/' if self.rest().starts_with("//") => {
                    while let Some(c) = self.peek_char() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                    continue;
                }
                _ => {}
            }
            self.bump();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    // `#[cfg(test)]` compiles this module only for `cargo test`.
    use super::*;

    fn lex_all(src: &str, mode: Mode) -> Vec<Tok> {
        let mut lx = Lexer::new(src);
        let mut out = Vec::new();
        loop {
            let t = lx.next(mode);
            if t.tok == Tok::Eof {
                break;
            }
            out.push(t.tok);
        }
        out
    }

    #[test]
    fn unicode_and_ascii_lex_the_same() {
        assert_eq!(
            lex_all("A ∪ B", Mode::Normal),
            lex_all("A union B", Mode::Normal)
        );
        assert_eq!(
            lex_all("x ∈ S", Mode::Normal),
            lex_all("x in S", Mode::Normal)
        );
        assert_eq!(lex_all("ε", Mode::Normal), lex_all("eps", Mode::Normal));
        assert_eq!(lex_all("delta", Mode::Normal), vec![Tok::Ident("δ".into())]);
    }

    #[test]
    fn atoms_only_where_an_expression_starts() {
        assert_eq!(
            lex_all("{:q0:0}", Mode::Normal),
            vec![
                Tok::Op(Op::LBrace),
                Tok::Atom("q0".into()),
                Tok::Op(Op::Colon),
                Tok::Int(0),
                Tok::Op(Op::RBrace)
            ]
        );
        assert_eq!(
            lex_all("q:atom", Mode::Normal),
            vec![
                Tok::Ident("q".into()),
                Tok::Op(Op::Colon),
                Tok::Ident("atom".into())
            ]
        );
    }

    #[test]
    fn words_and_symbols() {
        assert_eq!(
            lex_all("\"<RESET>a\" 'b' '<X>'", Mode::Normal),
            vec![
                Tok::Word(vec!["RESET".into(), "a".into()]),
                Tok::Sym("b".into()),
                Tok::Sym("X".into())
            ]
        );
    }

    #[test]
    fn edges_in_machine_mode() {
        let toks = lex_all("q0 -0, ε → AZ-> q1", Mode::Machine);
        assert!(
            matches!(&toks[1], Tok::Edge { label, bracketed: false, .. } if label == "0, ε → AZ")
        );
        let toks = lex_all("q0 -[a, X -> YZ]-> q1", Mode::Machine);
        assert!(
            matches!(&toks[1], Tok::Edge { label, bracketed: true, .. } if label == "a, X -> YZ")
        );
        let toks = lex_all("q0 -a-> q1; q1 -b-> q2", Mode::Machine);
        assert_eq!(toks.len(), 7);
    }

    #[test]
    fn ascii_arrow_in_bare_label_is_e103_with_a_fix() {
        let mut lx = Lexer::new("q0 -a, X -> YZ-> q1");
        let _ = lx.next(Mode::Machine);
        let t = lx.next(Mode::Machine);
        assert!(matches!(
            t.tok,
            Tok::Edge {
                bracketed: true,
                ..
            }
        ));
        assert_eq!(lx.diags.len(), 1);
        assert_eq!(lx.diags[0].code, "E103");
        assert_eq!(lx.diags[0].fixes.len(), 1);
    }
}
