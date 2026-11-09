//! The parser: recursive descent for statements, Pratt parsing for expressions, a separate
//! machine-body parser (lexer machine mode), and error recovery at statement boundaries so one
//! bad line never hides the rest of the program.

use crate::ast::*;
use crate::diag::{Diagnostic, Span};
use crate::lexer::{Kw, Lexer, Mode, Op, Snapshot, Tok, Token};
// `HashSet` is an unordered set; we only ask "is this name in it?".
use std::collections::HashSet;
use std::rc::Rc;

/// The models of the standard prelude: their names start machine literals (Q-06).
pub const PRELUDE_MODELS: [&str; 14] = [
    "dfa", "nfa", "gnfa", "pda", "dpda", "tm", "ntm", "lba", "mtm", "twdfa", "mealy", "moore",
    "fst", "ctr",
];

/// Field names allowed in a machine body (01 §5.1).
const FIELD_NAMES: [&str; 17] = [
    "states", "Q", "alphabet", "Σ", "input", "stack", "Γ", "tape", "output", "Δ", "start", "s",
    "accept", "F", "final", "reject", "blank",
];

/// The parse failed here; the diagnostic has already been recorded.
#[derive(Debug)]
pub struct Fail;

type PResult<T> = Result<T, Fail>;

struct Peeked {
    mode: Mode,
    skip_nl: bool,
    before: Snapshot,
    token: Token,
}

pub struct Parser<'a> {
    lx: Lexer<'a>,
    peeked: Option<Peeked>,
    /// One entry per open delimiter: `true` when newlines end statements there (blocks), `false`
    /// inside `( … )`, `[ … ]` and set literals, where newlines are just spaces.
    nl_ctx: Vec<bool>,
    pub diags: Vec<Diagnostic>,
    models: HashSet<String>,
    prev_end: usize,
}

pub fn parse(src: &str, extra_models: &[String]) -> (Program, Vec<Diagnostic>) {
    let mut p = Parser::new(src);
    for m in extra_models {
        p.models.insert(m.clone());
    }
    let stmts = p.stmts_until_eof();
    let mut diags = p.lx.diags;
    diags.extend(p.diags);
    diags.sort_by_key(|d| d.span.from);
    (Program { stmts }, diags)
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Parser<'a> {
        Parser {
            lx: Lexer::new(src),
            peeked: None,
            nl_ctx: vec![true],
            diags: Vec::new(),
            models: PRELUDE_MODELS.iter().map(|s| s.to_string()).collect(),
            prev_end: 0,
        }
    }

    // ----- token access -----

    fn skip_nl(&self) -> bool {
        !self.nl_ctx.last().copied().unwrap_or(true)
    }

    fn peek_in(&mut self, mode: Mode) -> Token {
        let skip_nl = mode == Mode::Normal && self.skip_nl();
        if let Some(p) = &self.peeked {
            if p.mode == mode && p.skip_nl == skip_nl {
                return p.token.clone();
            }
            let before = p.before;
            self.lx.restore(before);
            self.peeked = None;
        }
        let before = self.lx.snapshot();
        let token = loop {
            let t = self.lx.next(mode);
            if !(skip_nl && t.tok == Tok::Newline) {
                break t;
            }
        };
        self.peeked = Some(Peeked {
            mode,
            skip_nl,
            before,
            token: token.clone(),
        });
        token
    }

    fn bump_in(&mut self, mode: Mode) -> Token {
        let t = self.peek_in(mode);
        self.peeked = None;
        self.prev_end = t.span.to;
        t
    }

    fn peek(&mut self) -> Token {
        self.peek_in(Mode::Normal)
    }

    fn bump(&mut self) -> Token {
        self.bump_in(Mode::Normal)
    }

    /// The token `n` places after the peeked one (normal mode, newlines included).
    fn peek_nth(&mut self, n: usize) -> Tok {
        let _ = self.peek();
        let snap = self.lx.snapshot();
        let mut t = Tok::Eof;
        for _ in 0..n {
            t = self.lx.next(Mode::Normal).tok;
        }
        self.lx.restore(snap);
        t
    }

    /// Drops the peeked token so the lexer can be read raw from where it starts.
    fn unpeek(&mut self) {
        if let Some(p) = self.peeked.take() {
            self.lx.restore(p.before);
        }
    }

    fn at(&mut self, op: Op) -> bool {
        self.peek().tok == Tok::Op(op)
    }

    fn at_kw(&mut self, kw: Kw) -> bool {
        self.peek().tok == Tok::Kw(kw)
    }

    fn at_ident(&mut self, name: &str) -> bool {
        matches!(self.peek().tok, Tok::Ident(ref n) if n == name)
    }

    fn eat(&mut self, op: Op) -> bool {
        if self.at(op) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn error(&mut self, code: &'static str, span: Span, msg: impl Into<String>) -> Fail {
        self.diags.push(Diagnostic::error(code, span, msg));
        Fail
    }

    fn describe(tok: &Tok) -> String {
        match tok {
            Tok::Ident(n) => format!("`{n}`"),
            Tok::Int(n) => format!("`{n}`"),
            Tok::Sym(s) => format!("symbol '{s}'"),
            Tok::Word(_) => "a word".to_string(),
            Tok::Regex(_) => "a regex".to_string(),
            Tok::Atom(a) => format!("`:{a}`"),
            Tok::Kw(k) => format!("keyword `{}`", format!("{k:?}").to_lowercase()),
            Tok::Op(o) => format!("`{}`", op_text(*o)),
            Tok::Edge { .. } => "an edge".to_string(),
            Tok::Newline => "the end of the line".to_string(),
            Tok::Eof => "the end of the file".to_string(),
        }
    }

    fn expect(&mut self, op: Op, context: &str) -> PResult<Span> {
        let t = self.peek();
        if t.tok == Tok::Op(op) {
            self.bump();
            return Ok(t.span);
        }
        let msg = format!(
            "expected `{}` {context}, found {}",
            op_text(op),
            Self::describe(&t.tok)
        );
        Err(self.error("E101", t.span, msg))
    }

    fn expect_m(&mut self, op: Op, context: &str) -> PResult<Span> {
        let t = self.peek_in(Mode::Machine);
        if t.tok == Tok::Op(op) {
            self.bump_in(Mode::Machine);
            return Ok(t.span);
        }
        let msg = format!(
            "expected `{}` {context}, found {}",
            op_text(op),
            Self::describe(&t.tok)
        );
        Err(self.error("E101", t.span, msg))
    }

    fn ident(&mut self, what: &str) -> PResult<(String, Span)> {
        let t = self.peek();
        match t.tok {
            Tok::Ident(n) => {
                self.bump();
                Ok((n, t.span))
            }
            Tok::Kw(Kw::Reserved) => {
                self.bump();
                Err(self.reserved(t.span))
            }
            other => {
                let msg = format!("expected {what}, found {}", Self::describe(&other));
                Err(self.error("E101", t.span, msg))
            }
        }
    }

    fn reserved(&mut self, span: Span) -> Fail {
        let word = self.lx_text(span);
        self.error(
            "E102",
            span,
            format!(
                "`{word}` is reserved for a later version of Delta and can't be used as a name"
            ),
        )
    }

    fn lx_text(&self, span: Span) -> String {
        self.src().get(span.from..span.to).unwrap_or("").to_string()
    }

    fn src(&self) -> &'a str {
        self.lx_src()
    }

    fn lx_src(&self) -> &'a str {
        // The lexer owns the source reference; this reads it back.
        self.lx.source()
    }

    fn skip_newlines(&mut self) {
        loop {
            let t = self.peek();
            if t.tok == Tok::Newline {
                self.bump();
            } else {
                break;
            }
        }
    }

    // ----- statements -----

    fn stmts_until_eof(&mut self) -> Vec<Stmt> {
        let mut out = Vec::new();
        loop {
            let t = self.peek();
            match t.tok {
                Tok::Eof => break,
                Tok::Newline | Tok::Op(Op::Semi) => {
                    self.bump();
                }
                Tok::Op(Op::RBrace) => {
                    self.bump();
                    self.diags
                        .push(Diagnostic::error("E101", t.span, "unmatched `}`"));
                }
                _ => {
                    if let Some(s) = self.stmt_line() {
                        out.push(s);
                    }
                }
            }
        }
        out
    }

    /// A block `{ stmts }` (the `{` not yet consumed).
    fn block(&mut self) -> PResult<Block> {
        self.expect(Op::LBrace, "to start a block")?;
        self.nl_ctx.push(true);
        let mut out = Vec::new();
        loop {
            let t = self.peek();
            match t.tok {
                Tok::Eof => {
                    self.nl_ctx.pop();
                    return Err(self.error("E101", t.span, "expected `}` to close the block"));
                }
                Tok::Newline | Tok::Op(Op::Semi) => {
                    self.bump();
                }
                Tok::Op(Op::RBrace) => {
                    self.bump();
                    break;
                }
                _ => {
                    if let Some(s) = self.stmt_line() {
                        out.push(s);
                    }
                }
            }
        }
        self.nl_ctx.pop();
        Ok(out)
    }

    /// One statement plus its terminator; recovers to the next line on error.
    fn stmt_line(&mut self) -> Option<Stmt> {
        let depth = self.nl_ctx.len();
        let start = self.peek().span.from;
        match self.stmt() {
            Ok(kind) => {
                let span = Span::new(start, self.prev_end.max(start));
                let t = self.peek();
                match t.tok {
                    Tok::Newline | Tok::Op(Op::Semi) => {
                        self.bump();
                    }
                    Tok::Op(Op::RBrace) | Tok::Eof => {}
                    other => {
                        let msg = format!(
                            "expected a newline or `;` after the statement, found {}",
                            Self::describe(&other)
                        );
                        self.diags.push(Diagnostic::error("E101", t.span, msg));
                        self.recover(depth);
                    }
                }
                Some(Stmt { kind, span })
            }
            Err(Fail) => {
                self.recover(depth);
                None
            }
        }
    }

    /// Skips to the end of the current line (outside nested braces).
    fn recover(&mut self, depth: usize) {
        self.nl_ctx.truncate(depth.max(1));
        self.unpeek();
        let mut braces = 0usize;
        loop {
            let t = self.peek();
            match t.tok {
                Tok::Eof => return,
                Tok::Newline | Tok::Op(Op::Semi) if braces == 0 => {
                    self.bump();
                    return;
                }
                Tok::Op(Op::LBrace) => {
                    braces += 1;
                    self.bump();
                }
                Tok::Op(Op::RBrace) => {
                    if braces == 0 {
                        return;
                    }
                    braces -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn stmt(&mut self) -> PResult<StmtKind> {
        let t = self.peek();
        match &t.tok {
            Tok::Kw(Kw::Let) => {
                self.bump();
                let pat = self.pattern()?;
                self.expect(Op::Assign, "after the name in `let`")?;
                self.skip_newlines();
                let e = self.expr()?;
                Ok(StmtKind::Let(pat, e))
            }
            Tok::Kw(Kw::Var) => {
                self.bump();
                let (name, _) = self.ident("a variable name")?;
                self.expect(Op::Assign, "after the name in `var`")?;
                self.skip_newlines();
                let e = self.expr()?;
                Ok(StmtKind::Var(name, e))
            }
            Tok::Kw(Kw::Fn) if matches!(self.peek_nth(1), Tok::Ident(_)) => {
                let def = self.fn_def(true)?;
                Ok(StmtKind::Fn(Rc::new(def)))
            }
            Tok::Kw(Kw::If) => self.if_stmt(),
            Tok::Kw(Kw::For) => {
                self.bump();
                let pat = self.pattern()?;
                if !self.eat(Op::In) {
                    let t = self.peek();
                    return Err(self.error(
                        "E101",
                        t.span,
                        "expected `∈` or `in` after the loop variable",
                    ));
                }
                let iter = self.expr()?;
                let body = self.block()?;
                Ok(StmtKind::For(pat, iter, body))
            }
            Tok::Kw(Kw::While) => {
                self.bump();
                let cond = self.expr()?;
                let body = self.block()?;
                Ok(StmtKind::While(cond, body))
            }
            Tok::Kw(Kw::Return) => {
                self.bump();
                let t = self.peek();
                if matches!(
                    t.tok,
                    Tok::Newline | Tok::Eof | Tok::Op(Op::Semi | Op::RBrace)
                ) {
                    Ok(StmtKind::Return(None))
                } else {
                    Ok(StmtKind::Return(Some(self.expr()?)))
                }
            }
            Tok::Kw(Kw::Print) => {
                self.bump();
                let mut args = vec![self.expr()?];
                while self.eat(Op::Comma) {
                    self.skip_newlines();
                    args.push(self.expr()?);
                }
                Ok(StmtKind::Print(args))
            }
            Tok::Kw(Kw::Show) => {
                self.bump();
                Ok(StmtKind::Show(self.expr()?))
            }
            Tok::Kw(Kw::Save) => {
                self.bump();
                let v = self.expr()?;
                if !self.at_ident("to") {
                    let t = self.peek();
                    return Err(self.error(
                        "E101",
                        t.span,
                        "expected `to` in `save x to \"file\"`",
                    ));
                }
                self.bump();
                let path = self.expr()?;
                Ok(StmtKind::Save(v, path))
            }
            Tok::Kw(Kw::Assert) => {
                self.bump();
                Ok(StmtKind::Assert(self.expr()?))
            }
            Tok::Kw(Kw::Check) => self.check_stmt(),
            Tok::Kw(Kw::Run) => {
                self.bump();
                let m = self.expr()?;
                if !self.at_ident("on") {
                    let t = self.peek();
                    return Err(self.error("E101", t.span, "expected `on` in `run M on \"w\"`"));
                }
                self.bump();
                let w = self.expr()?;
                Ok(StmtKind::Run(m, w))
            }
            Tok::Kw(Kw::Lang) => self.lang_stmt(),
            Tok::Kw(Kw::Model) => self.model_decl(),
            Tok::Kw(Kw::Use) => self.use_stmt(),
            Tok::Kw(Kw::Budget) => self.budget_stmt(),
            Tok::Kw(Kw::Reserved) => {
                self.bump();
                Err(self.reserved(t.span))
            }
            Tok::Ident(name) if self.models.contains(name) => {
                // `dfa M { … }` or `pda P (options) { … }`
                let next = self.peek_nth(1);
                let after = self.peek_nth(2);
                if matches!(next, Tok::Ident(_))
                    && matches!(after, Tok::Op(Op::LBrace | Op::LParen))
                {
                    let e = self.machine_literal()?;
                    let (n, s) = match &e.kind {
                        ExprKind::Machine(m) => (m.name.clone().unwrap_or_default(), m.model_span),
                        _ => (String::new(), e.span),
                    };
                    return Ok(StmtKind::MachineDecl(n, s, e));
                }
                Ok(StmtKind::Expr(self.expr()?))
            }
            Tok::Ident(name) if self.peek_nth(1) == Tok::Op(Op::Assign) => {
                let name = name.clone();
                self.bump();
                self.bump();
                self.skip_newlines();
                let e = self.expr()?;
                Ok(StmtKind::Assign(name, t.span, e))
            }
            Tok::Op(Op::LBrace) => Ok(StmtKind::Block(self.block()?)),
            // `cfg G { … }` at statement level binds G.
            Tok::Kw(Kw::Cfg) if matches!(self.peek_nth(1), Tok::Ident(_)) => {
                let e = self.expr()?;
                let name = match &e.kind {
                    ExprKind::Cfg { name: Some(n), .. } => n.clone(),
                    _ => String::new(),
                };
                Ok(StmtKind::MachineDecl(name, t.span, e))
            }
            _ => Ok(StmtKind::Expr(self.expr()?)),
        }
    }

    fn if_stmt(&mut self) -> PResult<StmtKind> {
        self.bump(); // if
        let mut arms = Vec::new();
        let cond = self.expr()?;
        let body = self.block()?;
        arms.push((cond, body));
        let mut els = None;
        while self.at_kw(Kw::Else) {
            self.bump();
            if self.at_kw(Kw::If) {
                self.bump();
                let c = self.expr()?;
                let b = self.block()?;
                arms.push((c, b));
            } else {
                els = Some(self.block()?);
                break;
            }
        }
        Ok(StmtKind::If(arms, els))
    }

    fn check_stmt(&mut self) -> PResult<StmtKind> {
        self.bump(); // check
        let subject = self.expr()?;
        let kind = if self.at_ident("accepts") || self.at_ident("rejects") {
            let accepts = self.at_ident("accepts");
            self.bump();
            let mut words = vec![self.expr()?];
            while self.eat(Op::Comma) {
                self.skip_newlines();
                words.push(self.expr()?);
            }
            if accepts {
                CheckKind::Accepts(words)
            } else {
                CheckKind::Rejects(words)
            }
        } else if self.at_ident("up") && self.peek_nth(1) == Tok::Ident("to".into()) {
            self.bump();
            self.bump();
            CheckKind::Holds(Some(self.expr()?))
        } else {
            CheckKind::Holds(None)
        };
        Ok(StmtKind::Check(subject, kind))
    }

    fn lang_stmt(&mut self) -> PResult<StmtKind> {
        self.bump(); // lang
        let (name, name_span) = self.ident("the language's name")?;
        self.expect(Op::Assign, "after the language's name")?;
        self.skip_newlines();
        let value = self.expr()?;
        let claim = if self.at_ident("as") {
            self.bump();
            let (class, cspan) = self.ident("a class name such as `Regular`")?;
            if !self.at_ident("by") {
                let t = self.peek();
                return Err(self.error(
                    "E101",
                    t.span,
                    "expected `by` and the evidence after the claimed class",
                ));
            }
            self.bump();
            let evidence = self.expr()?;
            Some((class, cspan, evidence))
        } else {
            None
        };
        Ok(StmtKind::Lang {
            name,
            name_span,
            value,
            claim,
        })
    }

    fn model_decl(&mut self) -> PResult<StmtKind> {
        let start = self.bump().span.from; // model
        let (name, _) = self.ident("the model's name")?;
        let mut params = Vec::new();
        if self.eat(Op::LParen) {
            self.nl_ctx.push(false);
            while !self.at(Op::RParen) {
                let (p, _) = self.ident("a parameter name")?;
                params.push(p);
                if !self.eat(Op::Comma) {
                    break;
                }
            }
            self.nl_ctx.pop();
            self.expect(Op::RParen, "after the model's parameters")?;
        }
        let extends = if self.at_ident("extends") {
            self.bump();
            Some(self.ident("the model it extends")?.0)
        } else {
            None
        };
        self.expect(Op::LBrace, "to start the model's properties")?;
        self.unpeek();
        let Some((body, body_span)) = self.lx.raw_block() else {
            let s = Span::new(start, self.lx.pos());
            return Err(self.error("E101", s, "expected `}` to close the model"));
        };
        self.prev_end = body_span.to + 1;
        let mut props = Vec::new();
        let mut offset = body_span.from;
        for line in body.split_inclusive(['\n', ';']) {
            let content = line.split("//").next().unwrap_or("");
            let words: Vec<String> = content
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .filter(|w| !w.is_empty())
                .map(|w| w.to_string())
                .collect();
            if !words.is_empty() {
                props.push((words, Span::new(offset, offset + line.len())));
            }
            offset += line.len();
        }
        self.models.insert(name.clone());
        Ok(StmtKind::Model(Rc::new(ModelDecl {
            name,
            params,
            extends,
            props,
            span: Span::new(start, self.prev_end),
        })))
    }

    fn use_stmt(&mut self) -> PResult<StmtKind> {
        self.bump(); // use
        let (what, span) = self.ident("`strict`, `notation` or `explain`")?;
        match what.as_str() {
            "strict" => Ok(StmtKind::UseStrict),
            "notation" | "explain" => {
                let t = self.peek();
                let value = match t.tok {
                    Tok::Word(syms) => {
                        self.bump();
                        syms.concat()
                    }
                    Tok::Op(Op::LBrace) => {
                        self.bump();
                        self.unpeek();
                        let (body, s) = self.lx.raw_block().ok_or(Fail)?;
                        self.prev_end = s.to + 1;
                        format!("{{{body}}}")
                    }
                    _ => {
                        return Err(self.error(
                            "E101",
                            t.span,
                            "expected a profile name in quotes",
                        ));
                    }
                };
                if what == "notation" {
                    Ok(StmtKind::UseNotation(value))
                } else {
                    Ok(StmtKind::UseExplain(value))
                }
            }
            _ => Err(self.error(
                "E101",
                span,
                format!(
                    "unknown directive `use {what}`: expected `strict`, `notation` or `explain`"
                ),
            )),
        }
    }

    fn budget_stmt(&mut self) -> PResult<StmtKind> {
        self.bump(); // budget
        self.expect(Op::LBrace, "after `budget`")?;
        self.nl_ctx.push(false);
        let mut pairs = Vec::new();
        while !self.at(Op::RBrace) {
            let (k, s) = self.ident("a budget name (steps, states, depth, output)")?;
            self.expect(Op::Colon, "after the budget name")?;
            let v = self.expr()?;
            pairs.push((k, s, v));
            if !self.eat(Op::Comma) {
                break;
            }
        }
        self.nl_ctx.pop();
        self.expect(Op::RBrace, "to close `budget`")?;
        Ok(StmtKind::Budget(pairs))
    }

    fn fn_def(&mut self, named: bool) -> PResult<FnDef> {
        let start = self.bump().span.from; // fn
        let name = if named {
            Some(self.ident("the function's name")?.0)
        } else {
            None
        };
        self.expect(Op::LParen, "to start the parameters")?;
        self.nl_ctx.push(false);
        let mut params = Vec::new();
        while !self.at(Op::RParen) {
            let (pname, pspan) = self.ident("a parameter name")?;
            let ty = if self.eat(Op::Colon) {
                Some(self.type_expr()?)
            } else {
                None
            };
            params.push(Param {
                name: pname,
                ty,
                span: pspan,
            });
            if !self.eat(Op::Comma) {
                break;
            }
        }
        self.nl_ctx.pop();
        self.expect(Op::RParen, "to close the parameters")?;
        let ret = if self.eat(Op::Arrow) {
            Some(self.type_expr()?)
        } else {
            None
        };
        let body = if self.eat(Op::Assign) {
            self.skip_newlines();
            FnBody::Expr(self.expr()?)
        } else if self.at(Op::LBrace) {
            FnBody::Block(self.block()?)
        } else {
            let t = self.peek();
            return Err(self.error("E101", t.span, "expected `=` or `{` for the function body"));
        };
        Ok(FnDef {
            name,
            params,
            ret,
            body,
            span: Span::new(start, self.prev_end),
        })
    }

    fn type_expr(&mut self) -> PResult<TypeExpr> {
        let t = self.peek();
        let (name, span) = match t.tok {
            Tok::Ident(n) => {
                self.bump();
                (n, t.span)
            }
            Tok::Kw(Kw::Fn) => {
                self.bump();
                ("fn".to_string(), t.span)
            }
            Tok::Kw(Kw::None) => {
                self.bump();
                ("none".to_string(), t.span)
            }
            Tok::Kw(Kw::Machine) => {
                self.bump();
                ("machine".to_string(), t.span)
            }
            Tok::Kw(Kw::Lang) => {
                self.bump();
                ("lang".to_string(), t.span)
            }
            Tok::Kw(Kw::Model) => {
                self.bump();
                ("model".to_string(), t.span)
            }
            _ => return Err(self.error("E101", t.span, "expected a type")),
        };
        let mut args = Vec::new();
        if self.eat(Op::LBracket) {
            self.nl_ctx.push(false);
            while !self.at(Op::RBracket) {
                args.push(self.type_expr()?);
                if !self.eat(Op::Comma) {
                    break;
                }
            }
            self.nl_ctx.pop();
            self.expect(Op::RBracket, "to close the type arguments")?;
        }
        Ok(TypeExpr {
            name,
            args,
            span: span.join(Span::new(span.from, self.prev_end)),
        })
    }

    fn pattern(&mut self) -> PResult<Pattern> {
        let t = self.peek();
        match t.tok {
            Tok::Ident(n) if n == "_" => {
                self.bump();
                Ok(Pattern::Wild(t.span))
            }
            Tok::Ident(n) => {
                self.bump();
                Ok(Pattern::Var(n, t.span))
            }
            Tok::Op(Op::LParen) => {
                self.bump();
                self.nl_ctx.push(false);
                let mut ps = Vec::new();
                while !self.at(Op::RParen) {
                    ps.push(self.pattern()?);
                    if !self.eat(Op::Comma) {
                        break;
                    }
                }
                self.nl_ctx.pop();
                self.expect(Op::RParen, "to close the pattern")?;
                Ok(Pattern::Tuple(ps, Span::new(t.span.from, self.prev_end)))
            }
            Tok::Kw(Kw::Reserved) => {
                self.bump();
                Err(self.reserved(t.span))
            }
            other => {
                let msg = format!(
                    "expected a name or a tuple pattern, found {}",
                    Self::describe(&other)
                );
                Err(self.error("E109", t.span, msg))
            }
        }
    }

    // ----- expressions -----

    pub fn expr(&mut self) -> PResult<Expr> {
        self.expr_bp(0)
    }

    /// Is the `*` at the cursor a postfix star (Q-12)? Only when no operand follows it.
    fn star_is_postfix(&mut self) -> bool {
        matches!(
            self.peek_nth(1),
            Tok::Op(
                Op::RParen
                    | Op::RBracket
                    | Op::RBrace
                    | Op::Dot
                    | Op::Comma
                    | Op::Bar
                    | Op::Semi
                    | Op::Union
                    | Op::Inter
                    | Op::SetMinus
                    | Op::Equiv
                    | Op::Subset
                    | Op::In
                    | Op::EqEq
                    | Op::Ne
                    | Op::PlusPlus
            ) | Tok::Newline
                | Tok::Eof
        )
    }

    /// The binary operator at the cursor: (op, left bp, right bp, number of tokens).
    fn binop(&mut self) -> Option<(BinOp, u8, u8, usize)> {
        let t = self.peek();
        let r = match t.tok {
            Tok::Kw(Kw::Or) => (BinOp::Or, 1, 2, 1),
            Tok::Kw(Kw::And) => (BinOp::And, 3, 4, 1),
            Tok::Op(Op::EqEq) => (BinOp::Eq, 5, 6, 1),
            Tok::Op(Op::Ne) => (BinOp::Ne, 5, 6, 1),
            Tok::Op(Op::Lt) => (BinOp::Lt, 5, 6, 1),
            Tok::Op(Op::Le) => (BinOp::Le, 5, 6, 1),
            Tok::Op(Op::Gt) => (BinOp::Gt, 5, 6, 1),
            Tok::Op(Op::Ge) => (BinOp::Ge, 5, 6, 1),
            Tok::Op(Op::In) => (BinOp::In, 5, 6, 1),
            Tok::Op(Op::NotIn) => (BinOp::NotIn, 5, 6, 1),
            Tok::Kw(Kw::Not) if self.peek_nth(1) == Tok::Op(Op::In) => (BinOp::NotIn, 5, 6, 2),
            Tok::Op(Op::Subset) => (BinOp::Subset, 5, 6, 1),
            Tok::Op(Op::Equiv) => (BinOp::Equiv, 5, 6, 1),
            Tok::Ident(ref n) if n == "ends" && self.peek_nth(1) == Tok::Ident("with".into()) => {
                (BinOp::EndsWith, 5, 6, 2)
            }
            Tok::Ident(ref n) if n == "starts" && self.peek_nth(1) == Tok::Ident("with".into()) => {
                (BinOp::StartsWith, 5, 6, 2)
            }
            Tok::Ident(ref n) if n == "contains" => (BinOp::Contains, 5, 6, 1),
            Tok::Op(Op::Union) => (BinOp::Union, 9, 10, 1),
            Tok::Op(Op::SetMinus) => (BinOp::Diff, 9, 10, 1),
            Tok::Op(Op::Inter) => (BinOp::Inter, 11, 12, 1),
            Tok::Op(Op::PlusPlus) => (BinOp::Concat, 13, 14, 1),
            Tok::Op(Op::Plus) => (BinOp::Add, 13, 14, 1),
            Tok::Op(Op::Minus) => (BinOp::Sub, 13, 14, 1),
            Tok::Op(Op::Star) => (BinOp::Mul, 15, 16, 1),
            Tok::Op(Op::Slash) => (BinOp::Div, 15, 16, 1),
            Tok::Op(Op::Percent) => (BinOp::Mod, 15, 16, 1),
            _ => return None,
        };
        Some(r)
    }

    fn expr_bp(&mut self, min: u8) -> PResult<Expr> {
        let mut lhs = self.prefix()?;
        loop {
            let t = self.peek();
            let start = lhs.span.from;
            // Postfix operators bind tightest.
            match t.tok {
                Tok::Op(Op::LParen) => {
                    self.bump();
                    let args = self.call_args()?;
                    lhs = Expr {
                        kind: ExprKind::Call(Box::new(lhs), args),
                        span: Span::new(start, self.prev_end),
                    };
                    continue;
                }
                Tok::Op(Op::LBracket) => {
                    self.bump();
                    lhs = self.index_or_slice(lhs)?;
                    continue;
                }
                Tok::Op(Op::Dot) => {
                    self.bump();
                    let t = self.peek();
                    let name = match t.tok {
                        Tok::Ident(n) => n,
                        Tok::Kw(Kw::Model) => "model".to_string(),
                        Tok::Kw(Kw::Lang) => "lang".to_string(),
                        Tok::Kw(Kw::Run) => "run".to_string(),
                        _ => {
                            return Err(self.error(
                                "E101",
                                t.span,
                                "expected a member name after `.`",
                            ));
                        }
                    };
                    self.bump();
                    lhs = Expr {
                        kind: ExprKind::Member(Box::new(lhs), name),
                        span: Span::new(start, self.prev_end),
                    };
                    continue;
                }
                Tok::Op(Op::SupPlus) => {
                    self.bump();
                    lhs = Expr {
                        kind: ExprKind::Plus(Box::new(lhs)),
                        span: Span::new(start, self.prev_end),
                    };
                    continue;
                }
                Tok::Op(Op::Star) if self.star_is_postfix() => {
                    self.bump();
                    lhs = Expr {
                        kind: ExprKind::Star(Box::new(lhs)),
                        span: Span::new(start, self.prev_end),
                    };
                    continue;
                }
                Tok::Op(Op::Caret) => {
                    // power: right-associative, binds tighter than unary operators
                    if 19 < min {
                        break;
                    }
                    self.bump();
                    let upto = self.eat(Op::Le);
                    let exp = self.expr_bp(18)?;
                    lhs = Expr {
                        span: Span::new(start, exp.span.to),
                        kind: ExprKind::Power {
                            base: Box::new(lhs),
                            exp: Box::new(exp),
                            upto,
                        },
                    };
                    continue;
                }
                Tok::Op(Op::DotDot | Op::DotDotEq) => {
                    if 7 < min {
                        break;
                    }
                    let inclusive = t.tok == Tok::Op(Op::DotDotEq);
                    self.bump();
                    let hi = self.expr_bp(8)?;
                    lhs = Expr {
                        span: Span::new(start, hi.span.to),
                        kind: ExprKind::Range {
                            lo: Box::new(lhs),
                            hi: Box::new(hi),
                            inclusive,
                        },
                    };
                    continue;
                }
                _ => {}
            }
            let Some((op, lbp, rbp, ntok)) = self.binop() else {
                break;
            };
            if lbp < min {
                break;
            }
            for _ in 0..ntok {
                self.bump();
            }
            self.skip_newlines();
            let rhs = self.expr_bp(rbp)?;
            lhs = Expr {
                span: Span::new(start, rhs.span.to),
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }

    fn call_args(&mut self) -> PResult<Vec<Arg>> {
        self.nl_ctx.push(false);
        let mut args = Vec::new();
        while !self.at(Op::RParen) {
            let named = match self.peek().tok {
                Tok::Ident(n) if self.peek_nth(1) == Tok::Op(Op::Colon) => Some(n),
                _ => None,
            };
            if named.is_some() {
                self.bump();
                self.bump();
            }
            let value = match self.expr() {
                Ok(v) => v,
                Err(f) => {
                    self.nl_ctx.pop();
                    return Err(f);
                }
            };
            args.push(Arg { name: named, value });
            if !self.eat(Op::Comma) {
                break;
            }
        }
        self.nl_ctx.pop();
        self.expect(Op::RParen, "to close the arguments")?;
        Ok(args)
    }

    fn index_or_slice(&mut self, lhs: Expr) -> PResult<Expr> {
        self.nl_ctx.push(false);
        let start = lhs.span.from;
        let result = (|| {
            // A closure `|| { … }` is an anonymous function; here it lets `?` exit just this part.
            if self.eat(Op::DotDot) {
                let hi = if self.at(Op::RBracket) {
                    None
                } else {
                    Some(Box::new(self.expr_bp(8)?))
                };
                return Ok(ExprKind::Slice(Box::new(lhs.clone()), None, hi));
            }
            let lo = self.expr_bp(8)?;
            if self.eat(Op::DotDot) {
                let hi = if self.at(Op::RBracket) {
                    None
                } else {
                    Some(Box::new(self.expr_bp(8)?))
                };
                return Ok(ExprKind::Slice(
                    Box::new(lhs.clone()),
                    Some(Box::new(lo)),
                    hi,
                ));
            }
            Ok(ExprKind::Index(Box::new(lhs.clone()), Box::new(lo)))
        })();
        self.nl_ctx.pop();
        let kind = result?;
        self.expect(Op::RBracket, "to close the index")?;
        Ok(Expr {
            kind,
            span: Span::new(start, self.prev_end),
        })
    }

    fn prefix(&mut self) -> PResult<Expr> {
        let t = self.peek();
        let span = t.span;
        let unary = |p: &mut Self, op: UnOp, bp: u8| -> PResult<Expr> {
            p.bump();
            let e = p.expr_bp(bp)?;
            Ok(Expr {
                span: Span::new(span.from, e.span.to),
                kind: ExprKind::Unary(op, Box::new(e)),
            })
        };
        let simple = |p: &mut Self, kind: ExprKind| -> PResult<Expr> {
            p.bump();
            Ok(Expr { kind, span })
        };
        match t.tok {
            Tok::Op(Op::Minus) => unary(self, UnOp::Neg, 17),
            Tok::Op(Op::Compl) => unary(self, UnOp::Compl, 17),
            Tok::Op(Op::BigUnion) => unary(self, UnOp::BigUnion, 17),
            // `union(A, B)` is the library function, not the ASCII operator.
            Tok::Op(Op::Union) if self.peek_nth(1) == Tok::Op(Op::LParen) => {
                simple(self, ExprKind::Var("union".to_string()))
            }
            Tok::Kw(Kw::Not) => unary(self, UnOp::Not, 5),
            Tok::Int(n) => simple(self, ExprKind::Int(n)),
            Tok::Sym(s) => simple(self, ExprKind::Sym(s)),
            Tok::Word(w) => simple(self, ExprKind::Word(w)),
            Tok::Atom(a) => simple(self, ExprKind::Atom(a)),
            Tok::Kw(Kw::True) => simple(self, ExprKind::Bool(true)),
            Tok::Kw(Kw::False) => simple(self, ExprKind::Bool(false)),
            Tok::Kw(Kw::None) => simple(self, ExprKind::None),
            Tok::Op(Op::Eps) => simple(self, ExprKind::Eps),
            Tok::Op(Op::EmptySet) => simple(self, ExprKind::EmptySet),
            Tok::Op(Op::Blank) => simple(self, ExprKind::Sym("⊔".to_string())),
            Tok::Regex(text) => {
                self.bump();
                let text_span =
                    Span::new(span.from + 3, span.to.saturating_sub(1).max(span.from + 3));
                let over = if self.at_ident("over") {
                    self.bump();
                    Some(Box::new(self.expr_bp(17)?))
                } else {
                    None
                };
                Ok(Expr {
                    kind: ExprKind::Regex {
                        text,
                        text_span,
                        over,
                    },
                    span: Span::new(span.from, self.prev_end),
                })
            }
            Tok::Ident(ref name)
                if self.models.contains(name) && self.peek_nth(1) == Tok::Op(Op::LBrace) =>
            {
                self.machine_literal()
            }
            Tok::Ident(name) => simple(self, ExprKind::Var(name)),
            Tok::Op(Op::LParen) => {
                self.bump();
                self.nl_ctx.push(false);
                let r = (|| {
                    let mut items = vec![self.expr()?];
                    let mut tuple = false;
                    while self.eat(Op::Comma) {
                        tuple = true;
                        if self.at(Op::RParen) {
                            break;
                        }
                        items.push(self.expr()?);
                    }
                    Ok((items, tuple))
                })();
                self.nl_ctx.pop();
                let (mut items, tuple) = r?;
                self.expect(Op::RParen, "to close the parenthesis")?;
                let span = Span::new(span.from, self.prev_end);
                if !tuple
                    && items.len() == 1
                    && let Some(mut e) = items.pop()
                {
                    e.span = span;
                    return Ok(e);
                }
                Ok(Expr {
                    kind: ExprKind::Tuple(items),
                    span,
                })
            }
            Tok::Op(Op::LBracket) => {
                self.bump();
                self.nl_ctx.push(false);
                let r = self.list_body();
                self.nl_ctx.pop();
                let kind = r?;
                self.expect(Op::RBracket, "to close the list")?;
                Ok(Expr {
                    kind,
                    span: Span::new(span.from, self.prev_end),
                })
            }
            Tok::Op(Op::LBrace) => {
                self.bump();
                self.nl_ctx.push(false);
                let r = self.brace_body();
                self.nl_ctx.pop();
                let kind = r?;
                self.expect(Op::RBrace, "to close the set")?;
                Ok(Expr {
                    kind,
                    span: Span::new(span.from, self.prev_end),
                })
            }
            Tok::Kw(Kw::Fn) => {
                let def = self.fn_def(false)?;
                let span = def.span;
                Ok(Expr {
                    kind: ExprKind::Lambda(Rc::new(def)),
                    span,
                })
            }
            Tok::Kw(Kw::Machine) => self.canonical_machine(),
            Tok::Kw(Kw::Explore) => self.explore(),
            Tok::Kw(Kw::Fixpoint) => {
                self.bump();
                let (var, _) = self.ident("the fixpoint variable")?;
                if !self.at_ident("from") {
                    let t = self.peek();
                    return Err(self.error(
                        "E101",
                        t.span,
                        "expected `from` after the fixpoint variable",
                    ));
                }
                self.bump();
                let init = self.expr()?;
                self.expect(Op::LBrace, "to start the fixpoint body")?;
                self.nl_ctx.push(false);
                let body = self.expr();
                self.nl_ctx.pop();
                let body = body?;
                self.expect(Op::RBrace, "to close the fixpoint body")?;
                Ok(Expr {
                    kind: ExprKind::Fixpoint {
                        var,
                        init: Box::new(init),
                        body: Box::new(body),
                    },
                    span: Span::new(span.from, self.prev_end),
                })
            }
            Tok::Kw(Kw::Cfg) => {
                self.bump();
                let name = match self.peek().tok {
                    Tok::Ident(n) => {
                        self.bump();
                        Some(n)
                    }
                    _ => None,
                };
                self.expect(Op::LBrace, "to start the grammar")?;
                self.unpeek();
                let Some((body, body_span)) = self.lx.raw_block() else {
                    return Err(self.error("E101", span, "expected `}` to close the grammar"));
                };
                self.prev_end = body_span.to + 1;
                Ok(Expr {
                    kind: ExprKind::Cfg {
                        name,
                        body,
                        body_span,
                    },
                    span: Span::new(span.from, self.prev_end),
                })
            }
            Tok::Kw(Kw::Reserved) => {
                self.bump();
                Err(self.reserved(span))
            }
            other => {
                let msg = format!("expected an expression, found {}", Self::describe(&other));
                Err(self.error("E110", span, msg))
            }
        }
    }

    fn list_body(&mut self) -> PResult<ExprKind> {
        if self.at(Op::RBracket) {
            return Ok(ExprKind::List(Vec::new()));
        }
        let first = self.expr()?;
        if self.eat(Op::Bar) {
            let (head, quals) = self.builder_quals(first)?;
            return Ok(ExprKind::Builder {
                head: Box::new(head),
                quals,
                list: true,
            });
        }
        let mut items = vec![first];
        while self.eat(Op::Comma) {
            if self.at(Op::RBracket) {
                break;
            }
            items.push(self.expr()?);
        }
        Ok(ExprKind::List(items))
    }

    fn brace_body(&mut self) -> PResult<ExprKind> {
        if self.at(Op::RBrace) {
            return Ok(ExprKind::Set(Vec::new()));
        }
        // Record: `{ name: e, … }` (Q-07).
        if matches!(self.peek().tok, Tok::Ident(_)) && self.peek_nth(1) == Tok::Op(Op::Colon) {
            let mut fields = Vec::new();
            loop {
                let (name, _) = self.ident("a field name")?;
                self.expect(Op::Colon, "after the field name")?;
                let v = self.expr()?;
                fields.push((name, v));
                if !self.eat(Op::Comma) || self.at(Op::RBrace) {
                    break;
                }
            }
            return Ok(ExprKind::Record(fields));
        }
        let first = self.expr()?;
        if self.eat(Op::Colon) {
            let mut pairs = vec![(first, self.expr()?)];
            while self.eat(Op::Comma) {
                if self.at(Op::RBrace) {
                    break;
                }
                let k = self.expr()?;
                self.expect(Op::Colon, "between a map key and its value")?;
                let v = self.expr()?;
                pairs.push((k, v));
            }
            return Ok(ExprKind::Map(pairs));
        }
        if self.eat(Op::Bar) {
            let (head, quals) = self.builder_quals(first)?;
            return Ok(ExprKind::Builder {
                head: Box::new(head),
                quals,
                list: false,
            });
        }
        let mut items = vec![first];
        while self.eat(Op::Comma) {
            if self.at(Op::RBrace) {
                break;
            }
            items.push(self.expr()?);
        }
        Ok(ExprKind::Set(items))
    }

    /// After `head |`: the qualifiers. The filter form `{ x ∈ S | c }` becomes `{ x | x ∈ S, c }`.
    fn builder_quals(&mut self, head: Expr) -> PResult<(Expr, Vec<Qual>)> {
        let mut quals = Vec::new();
        let head = match head.kind {
            ExprKind::Binary(BinOp::In, ref lhs, ref rhs) if expr_to_pattern(lhs).is_some() => {
                let pat = expr_to_pattern(lhs).ok_or(Fail)?;
                quals.push(Qual::Gen(pat, (**rhs).clone()));
                (**lhs).clone()
            }
            _ => head,
        };
        loop {
            let e = self.expr()?;
            match e.kind {
                ExprKind::Binary(BinOp::In, ref lhs, ref rhs) if expr_to_pattern(lhs).is_some() => {
                    let pat = expr_to_pattern(lhs).ok_or(Fail)?;
                    quals.push(Qual::Gen(pat, (**rhs).clone()));
                }
                _ => quals.push(Qual::Cond(e)),
            }
            if !self.eat(Op::Comma) {
                break;
            }
        }
        Ok((head, quals))
    }

    fn explore(&mut self) -> PResult<Expr> {
        let start = self.bump().span.from; // explore
        let model = match self.peek().tok {
            Tok::Ident(n) => {
                let s = self.bump().span;
                Some((n, s))
            }
            _ => None,
        };
        self.expect(Op::LBrace, "to start the explore clauses")?;
        self.nl_ctx.push(true);
        let r = self.explore_clauses();
        self.nl_ctx.pop();
        let clauses = r?;
        let span = Span::new(start, self.prev_end);
        if !clauses
            .iter()
            .any(|c| matches!(c, ExploreClause::Alphabet(_)))
        {
            self.diags.push(
                Diagnostic::error(
                    "E107",
                    Span::new(start, start + "explore".len()),
                    "`explore` needs an `alphabet` clause (the input alphabet of the machine it builds)",
                ),
            );
        }
        Ok(Expr {
            kind: ExprKind::Explore(Rc::new(ExploreLit {
                model,
                clauses,
                span,
            })),
            span,
        })
    }

    fn explore_clauses(&mut self) -> PResult<Vec<ExploreClause>> {
        let mut clauses = Vec::new();
        loop {
            let t = self.peek();
            match t.tok {
                Tok::Newline | Tok::Op(Op::Semi) => {
                    self.bump();
                }
                Tok::Op(Op::RBrace) => {
                    self.bump();
                    return Ok(clauses);
                }
                Tok::Ident(ref kw) => {
                    let kw = kw.clone();
                    self.bump();
                    let clause = match kw.as_str() {
                        "alphabet" => ExploreClause::Alphabet(self.expr()?),
                        "start" => ExploreClause::Start(self.expr()?),
                        "accept" => {
                            let pat = self.pattern()?;
                            if !self.at_kw(Kw::If) {
                                let t = self.peek();
                                return Err(self.error(
                                    "E101",
                                    t.span,
                                    "expected `if` and a condition after the accept pattern",
                                ));
                            }
                            self.bump();
                            ExploreClause::Accept {
                                pat,
                                cond: self.expr()?,
                            }
                        }
                        "edges" => {
                            let pat = self.pattern()?;
                            self.expect(Op::Arrow, "after the state pattern")?;
                            ExploreClause::Edges {
                                pat,
                                target: self.expr()?,
                            }
                        }
                        "step" => {
                            let pat = self.pattern()?;
                            if !self.at_ident("on") {
                                let t = self.peek();
                                return Err(self.error(
                                    "E101",
                                    t.span,
                                    "expected `on` and a symbol variable (or ε)",
                                ));
                            }
                            self.bump();
                            let t = self.peek();
                            let on = match t.tok {
                                Tok::Op(Op::Eps) => {
                                    self.bump();
                                    StepOn::Eps
                                }
                                Tok::Ident(n) => {
                                    self.bump();
                                    StepOn::Sym(n, t.span)
                                }
                                _ => {
                                    return Err(self.error(
                                        "E101",
                                        t.span,
                                        "expected a symbol variable or ε after `on`",
                                    ));
                                }
                            };
                            self.expect(Op::Arrow, "before the successor")?;
                            let any = if self.at_ident("any") {
                                self.bump();
                                true
                            } else {
                                false
                            };
                            let target = self.expr()?;
                            ExploreClause::Step {
                                pat,
                                on,
                                any,
                                target,
                                span: Span::new(t.span.from, self.prev_end),
                            }
                        }
                        other => {
                            return Err(self.error(
                                "E106",
                                t.span,
                                format!("unknown explore clause `{other}`: expected alphabet, start, step, accept or edges"),
                            ));
                        }
                    };
                    clauses.push(clause);
                    let t = self.peek();
                    match t.tok {
                        Tok::Newline | Tok::Op(Op::Semi | Op::RBrace) => {}
                        _ => {
                            return Err(self.error(
                                "E101",
                                t.span,
                                "expected a newline or `;` after the clause",
                            ));
                        }
                    }
                }
                Tok::Eof => {
                    return Err(self.error("E101", t.span, "expected `}` to close `explore`"));
                }
                other => {
                    let msg = format!(
                        "expected an explore clause, found {}",
                        Self::describe(&other)
                    );
                    return Err(self.error("E106", t.span, msg));
                }
            }
        }
    }

    fn canonical_machine(&mut self) -> PResult<Expr> {
        let start = self.bump().span.from; // machine
        let (model, model_span) = self.ident("a model name")?;
        self.expect(Op::LBrace, "to start the machine record")?;
        self.nl_ctx.push(true);
        let r = (|| {
            let mut fields = Vec::new();
            loop {
                let t = self.peek();
                match t.tok {
                    Tok::Newline | Tok::Op(Op::Semi) => {
                        self.bump();
                    }
                    Tok::Op(Op::RBrace) => {
                        self.bump();
                        return Ok(fields);
                    }
                    Tok::Ident(n) => {
                        self.bump();
                        self.eat(Op::Colon);
                        let e = self.expr()?;
                        fields.push((n, t.span, e));
                    }
                    other => {
                        let msg =
                            format!("expected a field name, found {}", Self::describe(&other));
                        return Err(self.error("E101", t.span, msg));
                    }
                }
            }
        })();
        self.nl_ctx.pop();
        let fields = r?;
        let span = Span::new(start, self.prev_end);
        Ok(Expr {
            kind: ExprKind::Canonical(Rc::new(CanonicalLit {
                model,
                model_span,
                fields,
                span,
            })),
            span,
        })
    }

    // ----- machine literals (machine lexing mode) -----

    /// `model [Name] [(options)] { items }`, starting at the model name.
    fn machine_literal(&mut self) -> PResult<Expr> {
        let mt = self.bump();
        let model = match mt.tok {
            Tok::Ident(n) => n,
            _ => return Err(self.error("E101", mt.span, "expected a model name")),
        };
        let name = match self.peek().tok {
            Tok::Ident(n) => {
                self.bump();
                Some(n)
            }
            _ => None,
        };
        let mut options = Vec::new();
        if self.at(Op::LParen) {
            self.bump();
            self.unpeek();
            let Some((text, span)) = self.lx.raw_parens() else {
                return Err(self.error(
                    "E101",
                    mt.span,
                    "expected `)` to close the machine options",
                ));
            };
            self.prev_end = span.to + 1;
            let mut offset = span.from;
            for part in text.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    let lead = part.len() - part.trim_start().len();
                    options.push((
                        trimmed.to_string(),
                        Span::new(offset + lead, offset + lead + trimmed.len()),
                    ));
                }
                offset += part.len() + 1;
            }
        }
        self.expect(Op::LBrace, "to start the machine body")?;
        let items = self.machine_items()?;
        self.expect_m(Op::RBrace, "to close the machine body")?;
        let span = Span::new(mt.span.from, self.prev_end);
        Ok(Expr {
            kind: ExprKind::Machine(Rc::new(MachineLit {
                model,
                model_span: mt.span,
                name,
                options,
                items,
                span,
            })),
            span,
        })
    }

    /// Items up to (not including) the closing `}`.
    fn machine_items(&mut self) -> PResult<Vec<MItem>> {
        let mut items = Vec::new();
        loop {
            let t = self.peek_in(Mode::Machine);
            match &t.tok {
                Tok::Newline | Tok::Op(Op::Semi) => {
                    self.bump_in(Mode::Machine);
                    continue;
                }
                Tok::Op(Op::RBrace) => return Ok(items),
                Tok::Eof => {
                    return Err(self.error(
                        "E101",
                        t.span,
                        "expected `}` to close the machine body",
                    ));
                }
                _ => {}
            }
            match self.machine_item() {
                Ok(item) => {
                    items.push(item);
                    let t = self.peek_in(Mode::Machine);
                    match t.tok {
                        Tok::Newline | Tok::Op(Op::Semi | Op::RBrace) | Tok::Eof => {}
                        other => {
                            let msg = format!(
                                "expected a newline or `;` here, found {}: write one item per line or separate them with `;`",
                                Self::describe(&other)
                            );
                            self.diags.push(Diagnostic::error("E101", t.span, msg));
                            self.recover_machine_line();
                        }
                    }
                }
                Err(Fail) => self.recover_machine_line(),
            }
        }
    }

    fn recover_machine_line(&mut self) {
        loop {
            let t = self.peek_in(Mode::Machine);
            match t.tok {
                Tok::Newline | Tok::Op(Op::Semi) => {
                    self.bump_in(Mode::Machine);
                    return;
                }
                Tok::Op(Op::RBrace) | Tok::Eof => return,
                _ => {
                    self.bump_in(Mode::Machine);
                }
            }
        }
    }

    fn machine_item(&mut self) -> PResult<MItem> {
        let t = self.peek_in(Mode::Machine);
        match t.tok {
            Tok::Ident(ref kw) if kw == "for" => {
                self.bump_in(Mode::Machine);
                let pat = self.pattern()?;
                if !self.eat(Op::In) {
                    let t = self.peek();
                    return Err(self.error(
                        "E101",
                        t.span,
                        "expected `∈` or `in` after the loop variable",
                    ));
                }
                let iter = self.expr()?;
                self.expect(Op::LBrace, "to start the loop body")?;
                let body = self.machine_items()?;
                self.expect_m(Op::RBrace, "to close the loop body")?;
                Ok(MItem::For { pat, iter, body })
            }
            Tok::Ident(ref kw) if kw == "if" => {
                self.bump_in(Mode::Machine);
                let cond = self.expr()?;
                self.expect(Op::LBrace, "to start the if body")?;
                let then = self.machine_items()?;
                self.expect_m(Op::RBrace, "to close the if body")?;
                let mut els = Vec::new();
                if matches!(self.peek_in(Mode::Machine).tok, Tok::Ident(ref e) if e == "else") {
                    self.bump_in(Mode::Machine);
                    if matches!(self.peek_in(Mode::Machine).tok, Tok::Ident(ref e) if e == "if") {
                        els.push(self.machine_item()?);
                    } else {
                        self.expect_m(Op::LBrace, "to start the else body")?;
                        els = self.machine_items()?;
                        self.expect_m(Op::RBrace, "to close the else body")?;
                    }
                }
                Ok(MItem::If { cond, then, els })
            }
            Tok::Ident(ref d) if d == "δ" => {
                self.bump_in(Mode::Machine);
                let next = self.peek_in(Mode::Machine);
                match next.tok {
                    Tok::Op(Op::LParen) => {
                        self.bump_in(Mode::Machine);
                        let mut args = Vec::new();
                        loop {
                            args.push(self.mval()?);
                            if self.peek_in(Mode::Machine).tok == Tok::Op(Op::Comma) {
                                self.bump_in(Mode::Machine);
                            } else {
                                break;
                            }
                        }
                        self.expect_m(Op::RParen, "to close δ's arguments")?;
                        self.expect_m(Op::Assign, "after δ(…)")?;
                        let targets = self.mval_list()?;
                        Ok(MItem::Delta {
                            args,
                            targets,
                            span: Span::new(t.span.from, self.prev_end),
                        })
                    }
                    Tok::Op(Op::Assign) => {
                        self.bump_in(Mode::Machine);
                        if !matches!(self.peek_in(Mode::Machine).tok, Tok::Ident(ref n) if n == "table")
                        {
                            let s = self.peek_in(Mode::Machine).span;
                            return Err(self.error(
                                "E101",
                                s,
                                "expected `table { … }` after `δ =`",
                            ));
                        }
                        self.bump_in(Mode::Machine);
                        self.expect_m(Op::LBrace, "to start the table")?;
                        self.table(t.span.from)
                    }
                    _ => Err(self.error("E101", next.span, "expected `(` or `= table` after δ")),
                }
            }
            Tok::Ident(ref name) => {
                let name = name.clone();
                self.bump_in(Mode::Machine);
                let next = self.peek_in(Mode::Machine);
                if let Tok::Edge { .. } = next.tok {
                    return self.edge_rest(MState::Name(name, t.span));
                }
                let is_field = FIELD_NAMES.contains(&name.as_str())
                    || (name == "q0" && next.tok == Tok::Op(Op::Assign));
                if !is_field {
                    let msg = format!(
                        "expected an edge like `{name} -a-> q1` or a field (start, accept, alphabet, …), found {}",
                        Self::describe(&next.tok)
                    );
                    return Err(self.error("E101", next.span, msg));
                }
                if matches!(next.tok, Tok::Op(Op::Colon | Op::Assign)) {
                    self.bump_in(Mode::Machine);
                }
                let values = self.mval_list()?;
                let name = if name == "q0" {
                    "start".to_string()
                } else {
                    name
                };
                Ok(MItem::Field {
                    name,
                    name_span: t.span,
                    values,
                })
            }
            Tok::Op(Op::LBracket) => {
                let from = self.state_expr()?;
                self.edge_rest(from)
            }
            Tok::Edge { .. } => Err(self.error("E101", t.span, "an edge needs a state before it")),
            other => {
                let msg = format!(
                    "expected a transition or a field, found {}",
                    Self::describe(&other)
                );
                Err(self.error("E101", t.span, msg))
            }
        }
    }

    /// `[expr]` in state position (the `[` not yet consumed, machine mode).
    fn state_expr(&mut self) -> PResult<MState> {
        let open = self.bump_in(Mode::Machine);
        self.nl_ctx.push(false);
        let e = self.expr();
        self.nl_ctx.pop();
        let mut e = e?;
        self.expect(Op::RBracket, "to close the state expression")?;
        e.span = Span::new(open.span.from, self.prev_end);
        Ok(MState::Expr(e))
    }

    /// After the source state: the edge token and the target state.
    fn edge_rest(&mut self, from: MState) -> PResult<MItem> {
        let t = self.bump_in(Mode::Machine);
        let Tok::Edge {
            label,
            bracketed,
            label_span,
        } = t.tok
        else {
            return Err(self.error("E101", t.span, "expected an edge"));
        };
        let nt = self.peek_in(Mode::Machine);
        let to = match nt.tok {
            Tok::Ident(n) => {
                self.bump_in(Mode::Machine);
                MState::Name(n, nt.span)
            }
            Tok::Op(Op::LBracket) => self.state_expr()?,
            other => {
                let msg = format!(
                    "expected the target state after the edge, found {}",
                    Self::describe(&other)
                );
                return Err(self.error("E101", nt.span, msg));
            }
        };
        let span = from.span().join(to.span());
        Ok(MItem::Edge {
            from,
            label: LabelSrc {
                text: label,
                bracketed,
                span: label_span,
            },
            to,
            span,
        })
    }

    fn mval(&mut self) -> PResult<MVal> {
        let t = self.peek_in(Mode::Machine);
        match t.tok {
            Tok::Ident(n) => {
                self.bump_in(Mode::Machine);
                Ok(MVal::Name(n, t.span))
            }
            Tok::Sym(s) => {
                self.bump_in(Mode::Machine);
                Ok(MVal::Sym(s, t.span))
            }
            Tok::Op(Op::Eps) => {
                self.bump_in(Mode::Machine);
                Ok(MVal::Eps(t.span))
            }
            Tok::Op(Op::Blank) => {
                self.bump_in(Mode::Machine);
                Ok(MVal::Blank(t.span))
            }
            Tok::Op(Op::EmptySet) => {
                self.bump_in(Mode::Machine);
                Ok(MVal::Set(Vec::new(), t.span))
            }
            Tok::Op(Op::LBracket) => match self.state_expr()? {
                MState::Expr(e) => Ok(MVal::Expr(e)),
                MState::Name(n, s) => Ok(MVal::Name(n, s)),
            },
            Tok::Op(Op::LBrace) => {
                self.bump_in(Mode::Machine);
                let mut vals = Vec::new();
                loop {
                    let t2 = self.peek_in(Mode::Machine);
                    match t2.tok {
                        Tok::Op(Op::RBrace) => {
                            self.bump_in(Mode::Machine);
                            break;
                        }
                        Tok::Op(Op::Comma) | Tok::Newline => {
                            self.bump_in(Mode::Machine);
                        }
                        _ => vals.push(self.mval()?),
                    }
                }
                Ok(MVal::Set(vals, Span::new(t.span.from, self.prev_end)))
            }
            other => {
                let msg = format!(
                    "expected a state or a symbol, found {}",
                    Self::describe(&other)
                );
                Err(self.error("E101", t.span, msg))
            }
        }
    }

    /// Comma-separated values to the end of the line.
    fn mval_list(&mut self) -> PResult<Vec<MVal>> {
        let mut vals = Vec::new();
        loop {
            let t = self.peek_in(Mode::Machine);
            if matches!(
                t.tok,
                Tok::Newline | Tok::Op(Op::Semi | Op::RBrace) | Tok::Eof
            ) {
                break;
            }
            vals.push(self.mval()?);
            if self.peek_in(Mode::Machine).tok == Tok::Op(Op::Comma) {
                self.bump_in(Mode::Machine);
            } else {
                break;
            }
        }
        Ok(vals)
    }

    fn table(&mut self, start: usize) -> PResult<MItem> {
        let mut header = Vec::new();
        let mut rows = Vec::new();
        loop {
            let t = self.peek_in(Mode::Machine);
            match t.tok {
                Tok::Newline => {
                    self.bump_in(Mode::Machine);
                }
                Tok::Op(Op::RBrace) => {
                    self.bump_in(Mode::Machine);
                    break;
                }
                Tok::Op(Op::Bar) if header.is_empty() && rows.is_empty() => {
                    while self.peek_in(Mode::Machine).tok == Tok::Op(Op::Bar) {
                        self.bump_in(Mode::Machine);
                        header.push(self.mval()?);
                    }
                }
                Tok::Eof => {
                    return Err(self.error("E101", t.span, "expected `}` to close the table"));
                }
                _ => {
                    let state = match t.tok {
                        Tok::Op(Op::LBracket) => self.state_expr()?,
                        Tok::Ident(n) => {
                            self.bump_in(Mode::Machine);
                            MState::Name(n, t.span)
                        }
                        _ => {
                            return Err(self.error(
                                "E101",
                                t.span,
                                "expected a state at the start of a table row",
                            ));
                        }
                    };
                    let mut cells = Vec::new();
                    while self.peek_in(Mode::Machine).tok == Tok::Op(Op::Bar) {
                        self.bump_in(Mode::Machine);
                        let mut cell = Vec::new();
                        loop {
                            let c = self.peek_in(Mode::Machine);
                            if matches!(
                                c.tok,
                                Tok::Op(Op::Bar | Op::RBrace) | Tok::Newline | Tok::Eof
                            ) {
                                break;
                            }
                            cell.push(self.mval()?);
                            if self.peek_in(Mode::Machine).tok == Tok::Op(Op::Comma) {
                                self.bump_in(Mode::Machine);
                            }
                        }
                        cells.push(cell);
                    }
                    rows.push((state, cells));
                }
            }
        }
        Ok(MItem::Table {
            header,
            rows,
            span: Span::new(start, self.prev_end),
        })
    }
}

/// `x` or `(x, y)` as a pattern, for set-builder generators.
fn expr_to_pattern(e: &Expr) -> Option<Pattern> {
    match &e.kind {
        ExprKind::Var(n) if n == "_" => Some(Pattern::Wild(e.span)),
        ExprKind::Var(n) => Some(Pattern::Var(n.clone(), e.span)),
        ExprKind::Tuple(items) => {
            let ps: Option<Vec<Pattern>> = items.iter().map(expr_to_pattern).collect();
            Some(Pattern::Tuple(ps?, e.span))
        }
        _ => None,
    }
}

pub fn op_text(op: Op) -> &'static str {
    match op {
        Op::LParen => "(",
        Op::RParen => ")",
        Op::LBracket => "[",
        Op::RBracket => "]",
        Op::LBrace => "{",
        Op::RBrace => "}",
        Op::Comma => ",",
        Op::Semi => ";",
        Op::Colon => ":",
        Op::Dot => ".",
        Op::DotDot => "..",
        Op::DotDotEq => "..=",
        Op::Assign => "=",
        Op::EqEq => "==",
        Op::Ne => "≠",
        Op::Lt => "<",
        Op::Le => "≤",
        Op::Gt => ">",
        Op::Ge => "≥",
        Op::Plus => "+",
        Op::Minus => "-",
        Op::Star => "*",
        Op::Slash => "/",
        Op::Percent => "%",
        Op::Caret => "^",
        Op::PlusPlus => "++",
        Op::Arrow => "->",
        Op::Bar => "|",
        Op::Question => "?",
        Op::Union => "∪",
        Op::Inter => "∩",
        Op::SetMinus => "∖",
        Op::Compl => "∁",
        Op::In => "∈",
        Op::NotIn => "∉",
        Op::Subset => "⊆",
        Op::Equiv => "≡",
        Op::BigUnion => "⋃",
        Op::Eps => "ε",
        Op::EmptySet => "∅",
        Op::Blank => "⊔",
        Op::SupPlus => "⁺",
        Op::Concat => "·",
    }
}
