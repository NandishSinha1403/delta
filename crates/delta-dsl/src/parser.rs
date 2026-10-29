//! Recursive-descent parser with error recovery (docs/plan/04-DSL.md §10, 08-ERRORS.md §7).
//!
//! One typo must never blank the screen: after an error the parser skips to a synchronization
//! point (the next line at statement level, or the closing `}` of a block) and carries on, so the
//! rest of the file is still checked and drawn.

// `crate::ast::*` imports every public name from the `ast` module of this crate.
use crate::ast::*;
use crate::diag::{Diagnostics, Span, TextEdit, suggest};
use crate::lexer::{Tok, Token};

/// Deepest parenthesis nesting accepted in a regular expression.
pub const MAX_NESTING: usize = 200;
/// Largest k accepted in Rᵏ (E404).
pub const MAX_POWER: u32 = 64;

const FIELD_KEYS: [&str; 9] = [
    "Q", "Σ", "Sigma", "δ", "delta", "q0", "start", "F", "accept",
];
const STATEMENTS: [&str; 6] = ["dfa", "nfa", "regex", "let", "check", "run"];

/// Parses a token stream into a program, reporting syntax errors as diagnostics.
pub fn parse(toks: Vec<Token>, diags: &mut Diagnostics) -> Program {
    let mut p = Parser {
        toks,
        pos: 0,
        d: diags,
        paren_depth: 0,
    };
    p.program()
}

// `'a` is a lifetime: the Parser borrows the Diagnostics collector `d` and must not outlive it.
struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    d: &'a mut Diagnostics,
    paren_depth: usize,
}

/// Which model arrives in which phase, for friendly "not yet" messages.
fn later_phase(word: &str) -> Option<&'static str> {
    match word {
        "gnfa" | "layout" | "show" | "export" | "pump" | "enumerate" => Some("Phase 2"),
        "cfg" | "pda" | "dpda" | "parse" => Some("Phase 3 (context-free languages)"),
        "tm" | "ntm" | "lba" => Some("Phase 4 (Turing machines)"),
        _ => None,
    }
}

// `'_` says "some lifetime, I don't need to name it here".
impl Parser<'_> {
    // ---------- token helpers ----------

    fn tok(&self) -> &Token {
        // The token list always ends with Eof, so `.last()` is Some; `self.toks.get(..)` stays safe.
        self.toks
            .get(self.pos)
            // `.or_else(f)` keeps a `Some`, or calls f to try another value when it was `None`.
            .or_else(|| self.toks.last())
            .unwrap_or(&EOF)
    }

    fn peek(&self) -> &Tok {
        &self.tok().tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        self.toks.get(self.pos + n).map_or(&Tok::Eof, |t| &t.tok)
    }

    fn bump(&mut self) -> Token {
        let t = self.tok().clone();
        if self.pos < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn at(&self, t: &Tok) -> bool {
        self.peek() == t
    }

    fn at_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Word(x) if x == w)
    }

    fn at_eof(&self) -> bool {
        self.at(&Tok::Eof)
    }

    fn line_start(&self) -> bool {
        self.tok().nl_before
    }

    fn prev_end(&self) -> u32 {
        self.pos
            // `.checked_sub(1)` returns None instead of underflowing when the number is 0.
            .checked_sub(1)
            .and_then(|i| self.toks.get(i))
            .map_or(0, |t| t.span.end)
    }

    fn span(&self) -> Span {
        self.tok().span
    }

    /// Reports "expected X" at the current token.
    fn expected(&mut self, what: &str) {
        let (span, found) = (self.span(), describe(self.peek()));
        self.d
            .error("E120", span, format!("Expected {what}, found {found}."));
    }

    fn expect(&mut self, t: Tok, what: &str) -> bool {
        if self.at(&t) {
            self.bump();
            true
        } else {
            self.expected(what);
            false
        }
    }

    fn ident(&mut self, what: &str) -> Option<Ident> {
        if let Tok::Word(w) = self.peek() {
            let text = w.clone();
            let span = self.bump().span;
            Some(Ident { text, span })
        } else {
            self.expected(what);
            None
        }
    }

    /// Skips to the start of the next statement, jumping over whole `{ … }` blocks.
    fn skip_statement(&mut self) {
        let mut depth = 0usize;
        let mut first = true;
        while !self.at_eof() {
            if !first && depth == 0 && self.line_start() {
                break;
            }
            first = false;
            match self.peek() {
                Tok::LBrace => depth += 1,
                Tok::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.bump();
        }
    }

    /// Inside a block: skips to the next line, without eating the block's closing `}`.
    fn skip_line_in_block(&mut self) {
        let mut depth = 0usize;
        let mut first = true;
        while !self.at_eof() {
            if !first && depth == 0 && self.line_start() {
                break;
            }
            match self.peek() {
                Tok::LBrace => depth += 1,
                Tok::RBrace if depth == 0 => break,
                Tok::RBrace => depth -= 1,
                _ => {}
            }
            first = false;
            self.bump();
        }
    }

    // ---------- program ----------

    fn program(&mut self) -> Program {
        let mut items = Vec::new();
        let mut mode = Mode::Lenient;
        while !self.at_eof() {
            let t = self.tok().clone();
            match &t.tok {
                Tok::Word(w) => match w.as_str() {
                    "dfa" | "nfa" => {
                        if let Some(m) = self.machine() {
                            items.push(Item::Machine(m));
                        }
                    }
                    "regex" => {
                        if let Some(r) = self.regex_decl() {
                            items.push(Item::Regex(r));
                        }
                    }
                    "let" => {
                        if let Some(i) = self.let_stmt() {
                            items.push(i);
                        }
                    }
                    "check" => {
                        if let Some(i) = self.check() {
                            items.push(i);
                        }
                    }
                    "run" => {
                        if let Some(i) = self.run() {
                            items.push(i);
                        }
                    }
                    other => {
                        if let Some(phase) = later_phase(other) {
                            self.d.info(
                                "E123",
                                t.span,
                                format!("`{other}` arrives in {phase}. This statement is skipped for now."),
                            );
                        } else {
                            let msg = match suggest(other, STATEMENTS) {
                                Some(s) => {
                                    format!("Unknown statement `{other}`. Did you mean `{s}`?")
                                }
                                None => format!(
                                    "Expected a statement (`dfa`, `nfa`, `regex`, `let`, `check` or `run`), found `{other}`."
                                ),
                            };
                            let diag = self.d.error("E120", t.span, msg);
                            if let Some(s) = suggest(other, STATEMENTS) {
                                diag.fix(
                                    format!("Change to `{s}`"),
                                    vec![TextEdit {
                                        span: t.span,
                                        insert: s.to_string(),
                                    }],
                                );
                            }
                        }
                        self.skip_statement();
                    }
                },
                Tok::Hash => {
                    self.bump();
                    let ok = self.at_word("mode");
                    if ok {
                        self.bump();
                    }
                    match self.peek() {
                        Tok::Word(w) if ok && w == "strict" => mode = Mode::Strict,
                        Tok::Word(w) if ok && w == "lenient" => mode = Mode::Lenient,
                        _ => {
                            self.d.error(
                                "E120",
                                t.span.to(self.span()),
                                "Expected `#mode strict` or `#mode lenient`.",
                            );
                        }
                    }
                    self.skip_statement();
                }
                _ => {
                    self.expected("a statement (`dfa`, `nfa`, `regex`, `let`, `check` or `run`)");
                    self.skip_statement();
                }
            }
        }
        Program { items, mode }
    }

    // ---------- machines ----------

    fn machine(&mut self) -> Option<MachineDecl> {
        let kw = self.bump();
        let kind = if matches!(&kw.tok, Tok::Word(w) if w == "dfa") {
            Kind::Dfa
        } else {
            Kind::Nfa
        };
        // `let Some(x) = expr else { … }` binds x, or runs the else block (which must return).
        let Some(name) = self.ident("a machine name, e.g. `dfa M1 { … }`") else {
            self.skip_statement();
            return None;
        };
        let mut partial = false;
        if self.at_word("partial") && !self.line_start() {
            self.bump();
            partial = true;
        }
        if !self.at(&Tok::LBrace) {
            self.expected("`{` to start the machine's tuple");
            self.skip_statement();
            return None;
        }
        self.bump();
        let mut fields = Vec::new();
        loop {
            if self.at_eof() {
                let at = self.prev_end();
                self.d
                    .error(
                        "E121",
                        kw.span.to(name.span),
                        format!("`{}` is missing its closing `}}`.", name.text),
                    )
                    .fix(
                        "Insert `}`",
                        vec![TextEdit {
                            span: Span::at(at),
                            insert: "\n}".into(),
                        }],
                    );
                break;
            }
            if self.at(&Tok::RBrace) {
                self.bump();
                break;
            }
            let key = self.tok().clone();
            let field = match &key.tok {
                Tok::Word(w) if w == "Q" => {
                    self.bump();
                    self.expect(Tok::Eq, "`=` after Q")
                        .then(|| self.set())
                        .flatten()
                        .map(Field::Q)
                }
                Tok::Sigma => self.sigma_field(),
                Tok::Word(w) if w == "Sigma" => self.sigma_field(),
                Tok::Delta => self.delta_field(),
                Tok::Word(w) if w == "delta" => self.delta_field(),
                Tok::Word(w) if w == "q0" || w == "start" => {
                    self.bump();
                    if self.expect(Tok::Eq, "`=` after q0") {
                        self.ident("the start state").map(Field::Start)
                    } else {
                        None
                    }
                }
                Tok::Word(w) if w == "F" || w == "accept" => {
                    self.bump();
                    if self.expect(Tok::Eq, "`=` after F") {
                        self.set_or_single().map(Field::Accept)
                    } else {
                        None
                    }
                }
                Tok::Word(w) => {
                    let w = w.clone();
                    let diag = self.d.error(
                        "E122",
                        key.span,
                        match suggest(&w, FIELD_KEYS) {
                            Some(s) => format!("Unknown field `{w}`. Did you mean `{s}`?"),
                            None => format!(
                                "Unknown field `{w}`. A machine's fields are Q, Σ, δ, q0 and F."
                            ),
                        },
                    );
                    if let Some(s) = suggest(&w, FIELD_KEYS) {
                        diag.fix(
                            format!("Change to `{s}`"),
                            vec![TextEdit {
                                span: key.span,
                                insert: s.to_string(),
                            }],
                        );
                    }
                    None
                }
                _ => {
                    self.expected("a field (Q, Σ, δ, q0 or F)");
                    None
                }
            };
            match field {
                Some(f) => fields.push((f, key.span)),
                None => self.skip_line_in_block(),
            }
        }
        let span = kw.span.to(Span::at(self.prev_end()));
        Some(MachineDecl {
            kind,
            kind_span: kw.span,
            name,
            partial,
            fields,
            span,
        })
    }

    fn sigma_field(&mut self) -> Option<Field> {
        self.bump();
        if self.expect(Tok::Eq, "`=` after Σ") {
            self.set().map(Field::Sigma)
        } else {
            None
        }
    }

    fn elem(&mut self) -> Option<Elem> {
        let t = self.tok().clone();
        let (text, kind) = match &t.tok {
            Tok::Word(w) => (w.clone(), ElemKind::Word),
            Tok::Angle(n) => (n.clone(), ElemKind::Angle),
            Tok::Eps => ("ε".to_string(), ElemKind::Eps),
            Tok::Empty => ("∅".to_string(), ElemKind::Empty),
            Tok::Other(c) => (c.to_string(), ElemKind::Word),
            Tok::Hash => ("#".to_string(), ElemKind::Word),
            Tok::Plus => ("+".to_string(), ElemKind::Word),
            Tok::Star => ("*".to_string(), ElemKind::Word),
            Tok::Caret => ("^".to_string(), ElemKind::Word),
            _ => return None,
        };
        self.bump();
        Some(Elem {
            text,
            kind,
            span: t.span,
        })
    }

    /// `{ a, b }`, or `∅` for the empty set.
    fn set(&mut self) -> Option<SetLit> {
        if self.at(&Tok::Empty) {
            let span = self.bump().span;
            return Some(SetLit {
                items: Vec::new(),
                span,
                close: None,
            });
        }
        let open = self.span();
        if !self.expect(Tok::LBrace, "`{` to start a set") {
            return None;
        }
        let mut items = Vec::new();
        let close;
        loop {
            if self.at(&Tok::RBrace) {
                close = Some(self.bump().span);
                break;
            }
            // A word followed by `=` is the next field: the set was never closed.
            let next_field = matches!(self.peek(), Tok::Word(_))
                && self.peek_at(1) == &Tok::Eq
                && self.line_start();
            if self.at_eof() || next_field || self.at(&Tok::Delta) && self.line_start() {
                let at = self.prev_end();
                self.d
                    .error("E121", open, "This set is missing its closing `}`.")
                    .fix(
                        "Insert `}`",
                        vec![TextEdit {
                            span: Span::at(at),
                            insert: " }".into(),
                        }],
                    );
                close = None;
                break;
            }
            match self.elem() {
                Some(e) => items.push(e),
                None => {
                    self.expected("a set element");
                    self.bump();
                    continue;
                }
            }
            if self.at(&Tok::Comma) {
                self.bump();
            } else if !self.at(&Tok::RBrace) && self.elem_start() && !self.line_start() {
                let at = self.prev_end();
                self.d
                    .error("E120", self.span(), "Expected `,` between set elements.")
                    .fix(
                        "Insert `,`",
                        vec![TextEdit {
                            span: Span::at(at),
                            insert: ",".into(),
                        }],
                    );
            }
        }
        let end = close.map_or(self.prev_end(), |c| c.end);
        Some(SetLit {
            items,
            span: Span {
                start: open.start,
                end,
            },
            close,
        })
    }

    fn elem_start(&self) -> bool {
        matches!(
            self.peek(),
            Tok::Word(_) | Tok::Angle(_) | Tok::Eps | Tok::Other(_) | Tok::Hash
        )
    }

    /// `{ q2 }`, `∅`, or a single state `q2` (F = q2 is accepted as a one-element set).
    fn set_or_single(&mut self) -> Option<SetLit> {
        if let Tok::Word(_) = self.peek() {
            let e = self.elem()?;
            let span = e.span;
            return Some(SetLit {
                items: vec![e],
                span,
                close: None,
            });
        }
        self.set()
    }

    // ---------- δ ----------

    fn delta_field(&mut self) -> Option<Field> {
        self.bump();
        match self.peek() {
            Tok::Eq => {
                self.bump();
                if self.at_word("table") {
                    self.bump();
                    self.table().map(|e| Field::Delta(e, DeltaForm::Table))
                } else {
                    self.expected("`table { … }` after δ = (for arrows write `δ { q1 -0-> q2 }`)");
                    None
                }
            }
            Tok::LBrace => self.edges_block(),
            Tok::LParen => self
                .delta_fn()
                .map(|e| Field::Delta(vec![e], DeltaForm::Function)),
            _ => {
                self.expected("`= table { … }`, `{ … }` or `(q, a) = …` after δ");
                None
            }
        }
    }

    fn table(&mut self) -> Option<Vec<DeltaEntry>> {
        let open = self.span();
        if !self.expect(Tok::LBrace, "`{` after `table`") {
            return None;
        }
        let mut header: Vec<Elem> = Vec::new();
        if self.at(&Tok::Pipe) {
            while self.at(&Tok::Pipe) {
                self.bump();
                match self.elem() {
                    Some(e) => header.push(e),
                    None => self.expected("a symbol in the table header"),
                }
                if self.line_start() {
                    break;
                }
            }
        } else {
            self.d.error(
                "E205",
                self.span(),
                "The first row of a δ table lists the input symbols, starting with `|`: e.g. `| 0 | 1`.",
            );
        }
        let mut entries = Vec::new();
        loop {
            if self.at(&Tok::RBrace) {
                self.bump();
                break;
            }
            if self.at_eof() {
                self.d
                    .error("E121", open, "This table is missing its closing `}`.");
                break;
            }
            let Some(state) = self.ident("a state name at the start of a table row") else {
                self.skip_line_in_block();
                continue;
            };
            let mut col = 0usize;
            while self.at(&Tok::Pipe) && !self.line_start() {
                self.bump();
                let cell_start = self.span();
                let (to, set) = if self.at(&Tok::LBrace) || self.at(&Tok::Empty) {
                    let s = self.set().map(|s| s.items).unwrap_or_default();
                    (
                        s.into_iter()
                            .map(|e| Ident {
                                text: e.text,
                                span: e.span,
                            })
                            .collect(),
                        true,
                    )
                } else if let Some(i) = self.ident("a target state") {
                    (vec![i], false)
                } else {
                    break;
                };
                match header.get(col) {
                    Some(sym) => entries.push(DeltaEntry {
                        from: state.clone(),
                        label: sym.clone(),
                        to,
                        set,
                        span: cell_start.to(Span::at(self.prev_end())),
                    }),
                    None => {
                        self.d.error(
                            "E205",
                            cell_start,
                            format!(
                                "Row `{}` has more cells than the header has symbols.",
                                state.text
                            ),
                        );
                    }
                }
                col += 1;
            }
            if col < header.len() && !header.is_empty() {
                self.d.error(
                    "E205",
                    state.span,
                    format!(
                        "Row `{}` has {col} cells but the header lists {} symbols.",
                        state.text,
                        header.len()
                    ),
                );
            }
            if !self.line_start() && !self.at(&Tok::RBrace) && !self.at_eof() {
                self.expected("`|` between table cells");
                self.skip_line_in_block();
            }
        }
        Some(entries)
    }

    fn edges_block(&mut self) -> Option<Field> {
        let open = self.bump().span;
        let mut entries = Vec::new();
        let close;
        loop {
            if self.at(&Tok::RBrace) {
                close = self.bump().span;
                break;
            }
            if self.at_eof() {
                let at = self.prev_end();
                self.d
                    .error("E121", open, "This δ block is missing its closing `}`.")
                    .fix(
                        "Insert `}`",
                        vec![TextEdit {
                            span: Span::at(at),
                            insert: "\n  }".into(),
                        }],
                    );
                close = Span::at(at);
                break;
            }
            let before = entries.len();
            if !self.edge(&mut entries) {
                entries.truncate(before);
                self.skip_line_in_block();
            }
        }
        Some(Field::Delta(entries, DeltaForm::Edges(close)))
    }

    /// `q1 -0-> q2` or `q1 -[0,1]-> q2`. Returns false after reporting a syntax error.
    fn edge(&mut self, out: &mut Vec<DeltaEntry>) -> bool {
        let Some(from) = self.ident("a state, e.g. `q1 -0-> q2`") else {
            return false;
        };
        if !self.expect(Tok::Minus, "`-` after the source state, e.g. `q1 -0-> q2`") {
            return false;
        }
        let mut labels = Vec::new();
        if self.at(&Tok::LBracket) {
            self.bump();
            loop {
                match self.elem() {
                    Some(e) => labels.push(e),
                    None => {
                        self.expected("a symbol or ε");
                        return false;
                    }
                }
                if self.at(&Tok::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
            if !self.expect(Tok::RBracket, "`]` to close the label list") {
                return false;
            }
        } else {
            match self.elem() {
                Some(e) => labels.push(e),
                None => {
                    self.expected("a symbol or ε between `-` and `->`");
                    return false;
                }
            }
        }
        if !self.expect(Tok::Arrow, "`->` (or →) before the target state") {
            return false;
        }
        let Some(to) = self.ident("the target state") else {
            return false;
        };
        let span = from.span.to(to.span);
        for label in labels {
            out.push(DeltaEntry {
                from: from.clone(),
                label,
                to: vec![to.clone()],
                set: false,
                span,
            });
        }
        true
    }

    fn delta_fn(&mut self) -> Option<DeltaEntry> {
        let open = self.bump().span;
        let from = self.ident("a state, e.g. δ(q1, 0)")?;
        if !self.expect(Tok::Comma, "`,` between the state and the symbol") {
            return None;
        }
        let Some(label) = self.elem() else {
            self.expected("a symbol or ε");
            return None;
        };
        if !self.expect(Tok::RParen, "`)`") || !self.expect(Tok::Eq, "`=` after δ(q, a)") {
            return None;
        }
        let (to, set) = if self.at(&Tok::LBrace) || self.at(&Tok::Empty) {
            let s = self.set()?;
            (
                s.items
                    .into_iter()
                    .map(|e| Ident {
                        text: e.text,
                        span: e.span,
                    })
                    .collect(),
                true,
            )
        } else {
            (vec![self.ident("the target state")?], false)
        };
        Some(DeltaEntry {
            from,
            label,
            to,
            set,
            span: open.to(Span::at(self.prev_end())),
        })
    }

    // ---------- regular expressions ----------

    fn regex_decl(&mut self) -> Option<RegexDecl> {
        let kw = self.bump();
        let Some(name) = self.ident("a name, e.g. `regex R1 = (a ∪ b)*`") else {
            self.skip_statement();
            return None;
        };
        let over = if self.at_word("over") {
            self.bump();
            self.set()
        } else {
            None
        };
        if !self.expect(Tok::Eq, "`=` before the expression") {
            self.skip_statement();
            return None;
        }
        self.paren_depth = 0;
        let start = self.span();
        let Some(body) = self.rx_union() else {
            self.d
                .error("E402", start, "Expected a regular expression after `=`.");
            self.skip_statement();
            return None;
        };
        if self.at(&Tok::RParen) && !self.line_start() {
            self.d.error(
                "E401",
                self.span(),
                "Unbalanced `)`: there is no matching `(`.",
            );
            self.skip_statement();
        } else if !self.line_start() && !self.at_eof() {
            self.expected("the end of the expression");
            self.skip_statement();
        }
        Some(RegexDecl {
            name,
            over,
            body,
            span: kw.span.to(Span::at(self.prev_end())),
        })
    }

    fn continues(&self) -> bool {
        !self.at_eof() && (!self.line_start() || self.paren_depth > 0)
    }

    fn rx_union(&mut self) -> Option<Rx> {
        let first = self.rx_concat()?;
        let mut parts = vec![first];
        while self.continues() && (self.at(&Tok::Pipe) || self.at(&Tok::Union)) {
            let op = self.bump();
            match self.rx_concat() {
                Some(x) => parts.push(x),
                None => {
                    self.d
                        .error("E402", op.span, "`∪` needs an expression on its right.");
                }
            }
        }
        if parts.len() == 1 {
            parts.pop()
        } else {
            Some(Rx::Union(parts))
        }
    }

    fn atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Tok::Word(_)
                | Tok::Angle(_)
                | Tok::Eps
                | Tok::Empty
                | Tok::Sigma
                | Tok::LParen
                | Tok::Other(_)
                | Tok::Hash
        )
    }

    fn rx_concat(&mut self) -> Option<Rx> {
        let mut parts = Vec::new();
        loop {
            if !self.continues() {
                break;
            }
            if self.at(&Tok::Circ) {
                self.bump();
                continue;
            }
            if !self.atom_start() {
                if parts.is_empty() && (self.at(&Tok::Star) || self.at(&Tok::Plus)) {
                    self.d.error(
                        "E402",
                        self.span(),
                        "`*` and `+` need an expression on their left.",
                    );
                    self.bump();
                    continue;
                }
                break;
            }
            match self.rx_postfix() {
                Some(mut pieces) => parts.append(&mut pieces),
                None => break,
            }
        }
        match parts.len() {
            0 => None,
            1 => parts.pop(),
            _ => Some(Rx::Concat(parts)),
        }
    }

    fn rx_postfix(&mut self) -> Option<Vec<Rx>> {
        let mut atoms = self.rx_atom()?;
        let mut last = atoms.pop()?;
        while self.continues() {
            match self.peek() {
                Tok::Star => {
                    self.bump();
                    last = Rx::Star(Box::new(last));
                }
                Tok::Plus => {
                    let plus = self.bump();
                    // W405: `a+b` written tight usually means union in other tools, not R⁺.
                    let tight_before = self
                        .toks
                        .get(self.pos.saturating_sub(2))
                        .is_some_and(|t| t.span.end == plus.span.start);
                    let tight_after = self.span().start == plus.span.end && self.atom_start();
                    if tight_before && tight_after {
                        self.d
                            .warning(
                                "W405",
                                plus.span,
                                "In Sipser, + means one-or-more (R⁺ = RR*), not union. Did you mean ∪?",
                            )
                            .book(crate::book::DEF_1_52_PLUS)
                            .fix("Replace + with ∪", vec![TextEdit { span: plus.span, insert: " ∪ ".into() }]);
                    }
                    last = Rx::Plus(Box::new(last));
                }
                Tok::Caret => {
                    let caret = self.bump();
                    let k = match self.peek() {
                        Tok::Word(w) if w.chars().all(|c| c.is_ascii_digit()) => {
                            w.parse::<u32>().ok()
                        }
                        _ => None,
                    };
                    match k {
                        Some(k) if k <= MAX_POWER => {
                            self.bump();
                            last = Rx::Pow(Box::new(last), k);
                        }
                        Some(_) => {
                            let s = self.bump().span;
                            self.d.error(
                                "E404",
                                caret.span.to(s),
                                format!("Rᵏ: k may be at most {MAX_POWER}."),
                            );
                        }
                        None => {
                            self.d.error(
                                "E404",
                                caret.span,
                                "`^` needs a whole number after it, e.g. (ΣΣ)^3.",
                            );
                        }
                    }
                }
                _ => break,
            }
        }
        atoms.push(last);
        Some(atoms)
    }

    fn rx_atom(&mut self) -> Option<Vec<Rx>> {
        let t = self.tok().clone();
        match &t.tok {
            Tok::Word(w) if w == "Sigma" => {
                self.bump();
                Some(vec![Rx::Sigma(t.span)])
            }
            Tok::Word(w) => {
                self.bump();
                // Every character is one symbol: `aba` is a ∘ b ∘ a (postfix binds to the last one).
                let start = t.span.start as usize;
                Some(
                    w.char_indices()
                        .map(|(i, c)| {
                            Rx::Sym(
                                c.to_string(),
                                Span::new(start + i, start + i + c.len_utf8()),
                            )
                        })
                        .collect(),
                )
            }
            Tok::Angle(n) => {
                self.bump();
                Some(vec![Rx::Sym(n.clone(), t.span)])
            }
            Tok::Other(c) => {
                self.bump();
                Some(vec![Rx::Sym(c.to_string(), t.span)])
            }
            Tok::Hash => {
                self.bump();
                Some(vec![Rx::Sym("#".into(), t.span)])
            }
            Tok::Eps => {
                self.bump();
                Some(vec![Rx::Eps(t.span)])
            }
            Tok::Empty => {
                self.bump();
                Some(vec![Rx::Empty(t.span)])
            }
            Tok::Sigma => {
                self.bump();
                Some(vec![Rx::Sigma(t.span)])
            }
            Tok::LParen => {
                if self.paren_depth >= MAX_NESTING {
                    self.d.error(
                        "E408",
                        t.span,
                        format!("Expression nested more than {MAX_NESTING} levels deep."),
                    );
                    return None;
                }
                self.bump();
                self.paren_depth += 1;
                let inner = self.rx_union();
                self.paren_depth -= 1;
                let inner = match inner {
                    Some(i) => i,
                    None => {
                        self.d.error(
                            "E402",
                            t.span,
                            "Empty parentheses `()`: write ε for the empty string.",
                        );
                        Rx::Eps(t.span)
                    }
                };
                if self.at(&Tok::RParen) {
                    self.bump();
                } else {
                    let at = self.prev_end();
                    self.d
                        .error("E401", t.span, "Unbalanced `(`: add a matching `)`.")
                        .fix(
                            "Insert `)`",
                            vec![TextEdit {
                                span: Span::at(at),
                                insert: ")".into(),
                            }],
                        );
                }
                Some(vec![inner])
            }
            _ => None,
        }
    }

    // ---------- commands ----------

    fn value(&mut self) -> Option<Value> {
        let t = self.tok().clone();
        match &t.tok {
            Tok::Word(w) if w == "true" || w == "false" => {
                self.bump();
                Some(Value::Bool(w == "true", t.span))
            }
            Tok::Word(w)
                if w.chars().all(|c| c.is_ascii_digit()) && self.peek_at(1) != &Tok::LParen =>
            {
                self.bump();
                Some(Value::Int(w.parse().unwrap_or(0), t.span))
            }
            Tok::Word(_) if self.peek_at(1) == &Tok::LParen => {
                self.call().map(|c| Value::Call(Box::new(c)))
            }
            Tok::Word(w) => {
                self.bump();
                Some(Value::Name(Ident {
                    text: w.clone(),
                    span: t.span,
                }))
            }
            Tok::LBracket => {
                self.bump();
                let mut list = Vec::new();
                while !self.at(&Tok::RBracket) && !self.at_eof() && !self.line_start() {
                    list.push(self.ident("a state name")?);
                    if self.at(&Tok::Comma) {
                        self.bump();
                    }
                }
                let end = self.span();
                self.expect(Tok::RBracket, "`]`");
                Some(Value::List(list, t.span.to(end)))
            }
            _ => {
                self.expected("a value");
                None
            }
        }
    }

    fn call(&mut self) -> Option<Call> {
        let func = self.ident("a function, e.g. dfa(N)")?;
        if !self.expect(Tok::LParen, "`(`") {
            return None;
        }
        let mut args = Vec::new();
        let mut options = Vec::new();
        while !self.at(&Tok::RParen) {
            if self.at_eof() || self.line_start() {
                let at = self.prev_end();
                self.d
                    .error("E120", func.span, "This call is missing its closing `)`.")
                    .fix(
                        "Insert `)`",
                        vec![TextEdit {
                            span: Span::at(at),
                            insert: ")".into(),
                        }],
                    );
                return Some(Call {
                    span: func.span.to(Span::at(at)),
                    func,
                    args,
                    options,
                });
            }
            if matches!(self.peek(), Tok::Word(_)) && self.peek_at(1) == &Tok::Colon {
                let key = self.ident("an option name")?;
                self.bump();
                options.push((key, self.value()?));
            } else {
                args.push(self.value()?);
            }
            if self.at(&Tok::Comma) {
                self.bump();
            } else if !self.at(&Tok::RParen) {
                self.expected("`,` or `)`");
                return None;
            }
        }
        let close = self.bump().span;
        Some(Call {
            span: func.span.to(close),
            func,
            args,
            options,
        })
    }

    fn let_stmt(&mut self) -> Option<Item> {
        let kw = self.bump();
        // An immediately-called closure lets `?` stop early without leaving the whole function.
        let result = (|| {
            let name = self.ident("a name, e.g. `let D = dfa(N)`")?;
            if !self.expect(Tok::Eq, "`=`") {
                return None;
            }
            let call = self.call()?;
            Some((name, call))
        })();
        match result {
            Some((name, call)) => {
                let span = kw.span.to(call.span);
                if !self.line_start() && !self.at_eof() {
                    self.expected("the end of the line");
                    self.skip_statement();
                }
                Some(Item::Let { name, call, span })
            }
            None => {
                self.skip_statement();
                None
            }
        }
    }

    fn strings(&mut self) -> Vec<StrLit> {
        let mut out = Vec::new();
        while let Tok::Str(s) = self.peek() {
            let text = s.clone();
            let span = self.bump().span;
            out.push(StrLit { text, span });
            if self.at(&Tok::Comma) && !self.line_start() {
                self.bump();
            } else {
                break;
            }
        }
        out
    }

    fn check(&mut self) -> Option<Item> {
        let kw = self.bump();
        if matches!(self.peek(), Tok::Word(_)) && self.peek_at(1) == &Tok::LParen {
            let s = self.span();
            self.d.info("E123", s, "`check equivalent(…)`, `check empty(…)` and other decision procedures arrive in Phase 2.");
            self.skip_statement();
            return None;
        }
        let Some(target) = self.ident("a machine name, e.g. `check M1 accepts \"01\"`") else {
            self.skip_statement();
            return None;
        };
        let accepts = match self.peek() {
            Tok::Word(w) if w == "accepts" => true,
            Tok::Word(w) if w == "rejects" => false,
            _ => {
                self.expected("`accepts` or `rejects`");
                self.skip_statement();
                return None;
            }
        };
        self.bump();
        let inputs = self.strings();
        if inputs.is_empty() {
            self.expected("one or more strings, e.g. \"0101\", \"\"");
            self.skip_statement();
            return None;
        }
        if !self.line_start() && !self.at_eof() {
            self.expected("`,` between strings or the end of the line");
            self.skip_statement();
        }
        let span = kw.span.to(Span::at(self.prev_end()));
        Some(Item::Check {
            target,
            accepts,
            inputs,
            span,
        })
    }

    fn run(&mut self) -> Option<Item> {
        let kw = self.bump();
        let Some(target) = self.ident("a machine name, e.g. `run M1 on \"0101\"`") else {
            self.skip_statement();
            return None;
        };
        if !self.at_word("on") {
            self.expected("`on` before the input string");
            self.skip_statement();
            return None;
        }
        self.bump();
        let input = match self.peek() {
            Tok::Str(s) => {
                let text = s.clone();
                StrLit {
                    text,
                    span: self.bump().span,
                }
            }
            _ => {
                self.expected("an input string in quotes, e.g. \"0101\" (\"\" is ε)");
                self.skip_statement();
                return None;
            }
        };
        if !self.line_start() && !self.at_eof() {
            let s = self.span();
            self.d.info("E123", s, "Run options (`limit`, `tree`) arrive with Turing machines and the computation-tree view.");
            self.skip_statement();
        }
        let span = kw.span.to(input.span);
        Some(Item::Run {
            target,
            input,
            span,
        })
    }
}

// A `static` is a single value that lives for the whole program (here: a fallback Eof token).
static EOF: Token = Token {
    tok: Tok::Eof,
    span: Span { start: 0, end: 0 },
    nl_before: true,
};

fn describe(t: &Tok) -> String {
    match t {
        Tok::Word(w) => format!("`{w}`"),
        Tok::Angle(n) => format!("`<{n}>`"),
        Tok::Str(s) => format!("\"{s}\""),
        Tok::Eof => "the end of the file".into(),
        Tok::LBrace => "`{`".into(),
        Tok::RBrace => "`}`".into(),
        Tok::LParen => "`(`".into(),
        Tok::RParen => "`)`".into(),
        Tok::LBracket => "`[`".into(),
        Tok::RBracket => "`]`".into(),
        Tok::Comma => "`,`".into(),
        Tok::Semi => "`;`".into(),
        Tok::Colon => "`:`".into(),
        Tok::Eq => "`=`".into(),
        Tok::Pipe => "`|`".into(),
        Tok::Union => "`∪`".into(),
        Tok::Arrow => "`->`".into(),
        Tok::Minus => "`-`".into(),
        Tok::Star => "`*`".into(),
        Tok::Plus => "`+`".into(),
        Tok::Caret => "`^`".into(),
        Tok::Circ => "`∘`".into(),
        Tok::Eps => "`ε`".into(),
        Tok::Empty => "`∅`".into(),
        Tok::Sigma => "`Σ`".into(),
        Tok::Delta => "`δ`".into(),
        Tok::Hash => "`#`".into(),
        Tok::Other(c) => format!("`{c}`"),
    }
}
