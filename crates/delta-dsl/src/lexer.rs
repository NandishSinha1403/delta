//! Lexer: source text → tokens (docs/plan/04-DSL.md §1).
//!
//! Unicode and ASCII spellings become the same token (`ε`/`eps`, `∅`/`empty`, `→`/`->`), so the
//! parser never needs to care which one the user typed. Lexical errors (E0xx) are reported and
//! lexing continues.

use crate::diag::{Diagnostics, Span};

/// A token kind.
#[derive(Debug, Clone, PartialEq, Eq)]
// An `enum` lists every possible kind of token; some variants carry data, e.g. `Word(String)`.
pub enum Tok {
    /// A run of letters, digits, `_` and `'`: names (`q1`, `M1`, `q0'`), keywords, numbers, and
    /// single-character symbols (`0`, `a`).
    Word(String),
    /// A multi-character symbol written `<RESET>`.
    Angle(String),
    /// A string literal's contents, `"0101"`.
    Str(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Eq,
    /// `|` (also ASCII for ∪)
    Pipe,
    /// `∪`
    Union,
    /// `->` or `→`
    Arrow,
    Minus,
    Star,
    Plus,
    Caret,
    /// `∘` (explicit concatenation)
    Circ,
    /// `ε` or `eps`
    Eps,
    /// `∅` or `empty`
    Empty,
    /// `Σ`
    Sigma,
    /// `δ`
    Delta,
    /// `#`
    Hash,
    /// Any other single character (`$`, `.`, `⊔` …), usable as a symbol.
    Other(char),
    Eof,
}

/// A token with its position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// True if a line break separates this token from the previous one (statements are lines).
    pub nl_before: bool,
}

// A private `fn` (no `pub`) is only visible inside this file's module.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '\''
}

/// Splits `src` into tokens. Always ends with `Tok::Eof`.
pub fn lex(src: &str, diags: &mut Diagnostics) -> Vec<Token> {
    let mut out = Vec::new();
    // `.char_indices()` yields (byte offset, char) pairs; `.peekable()` lets us look one ahead.
    let mut chars = src.char_indices().peekable();
    let mut nl = true;
    while let Some((i, c)) = chars.next() {
        let start = i;
        let mut end = i + c.len_utf8();
        let tok = match c {
            '\n' => {
                nl = true;
                continue;
            }
            c if c.is_whitespace() => continue,
            // `matches!(value, Pattern)` is true when the value fits the pattern; `x if cond =>` adds a guard.
            '/' if matches!(chars.peek(), Some((_, '/'))) => {
                // Line comment: skip to the end of the line.
                while let Some(&(_, ch)) = chars.peek() {
                    if ch == '\n' {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            '/' if matches!(chars.peek(), Some((_, '*'))) => {
                chars.next();
                let mut closed = false;
                let mut prev = ' ';
                // `.by_ref()` lets the `for` loop borrow the iterator, so we can keep using it afterwards.
                for (_, ch) in chars.by_ref() {
                    if ch == '\n' {
                        nl = true;
                    }
                    if prev == '*' && ch == '/' {
                        closed = true;
                        break;
                    }
                    prev = ch;
                }
                if !closed {
                    diags
                        .error(
                            "E002",
                            Span::new(start, start + 2),
                            "Unclosed block comment: add */",
                        )
                        .fix(
                            "Close the comment",
                            vec![crate::diag::TextEdit {
                                span: Span::at(src.len() as u32),
                                insert: " */".into(),
                            }],
                        );
                }
                continue;
            }
            '"' => {
                let mut text = String::new();
                let mut closed = false;
                while let Some(&(j, ch)) = chars.peek() {
                    if ch == '\n' {
                        break;
                    }
                    chars.next();
                    end = j + ch.len_utf8();
                    if ch == '"' {
                        closed = true;
                        break;
                    }
                    text.push(ch);
                }
                if !closed {
                    diags
                        .error(
                            "E001",
                            Span::new(start, end),
                            "Unclosed string: add a closing \"",
                        )
                        .fix(
                            "Close the string",
                            vec![crate::diag::TextEdit {
                                span: Span::at(end as u32),
                                insert: "\"".into(),
                            }],
                        );
                }
                Tok::Str(text)
            }
            '<' => {
                // <NAME> is one symbol; a lone '<' is just a character.
                let rest = src.get(i + 1..).unwrap_or("");
                let name: String = rest.chars().take_while(|ch| is_word_char(*ch)).collect();
                if !name.is_empty() && rest.get(name.len()..).is_some_and(|r| r.starts_with('>')) {
                    for _ in 0..=name.chars().count() {
                        chars.next();
                    }
                    end = i + 1 + name.len() + 1;
                    Tok::Angle(name)
                } else {
                    Tok::Other('<')
                }
            }
            '-' if matches!(chars.peek(), Some((_, '>'))) => {
                chars.next();
                end += 1;
                Tok::Arrow
            }
            '→' => Tok::Arrow,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '[' => Tok::LBracket,
            ']' => Tok::RBracket,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            ':' => Tok::Colon,
            '=' => Tok::Eq,
            '|' => Tok::Pipe,
            '∪' => Tok::Union,
            '-' => Tok::Minus,
            '*' => Tok::Star,
            '+' => Tok::Plus,
            '^' => Tok::Caret,
            '∘' => Tok::Circ,
            'ε' => Tok::Eps,
            '∅' => Tok::Empty,
            'Σ' => Tok::Sigma,
            'δ' => Tok::Delta,
            '#' => Tok::Hash,
            c if is_word_char(c) => {
                let mut w = String::from(c);
                while let Some(&(j, ch)) = chars.peek() {
                    if !is_word_char(ch) {
                        break;
                    }
                    w.push(ch);
                    chars.next();
                    end = j + 1;
                }
                match w.as_str() {
                    "eps" => Tok::Eps,
                    "empty" => Tok::Empty,
                    _ => Tok::Word(w),
                }
            }
            other => Tok::Other(other),
        };
        out.push(Token {
            tok,
            span: Span::new(start, end),
            nl_before: nl,
        });
        nl = false;
    }
    out.push(Token {
        tok: Tok::Eof,
        span: Span::at(src.len() as u32),
        nl_before: true,
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Tok> {
        let mut d = Diagnostics::default();
        lex(src, &mut d).into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn unicode_and_ascii_agree() {
        assert_eq!(kinds("q1 -[0,ε]-> q2"), kinds("q1 -[0,eps]-> q2"));
        assert_eq!(kinds("a → b"), kinds("a -> b"));
    }

    #[test]
    fn edges_and_symbols() {
        assert_eq!(
            kinds("q1 -0-> q2 <RESET>"),
            vec![
                Tok::Word("q1".into()),
                Tok::Minus,
                Tok::Word("0".into()),
                Tok::Arrow,
                Tok::Word("q2".into()),
                Tok::Angle("RESET".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn unclosed_string_is_reported() {
        let mut d = Diagnostics::default();
        let toks = lex("check M accepts \"01\nrun", &mut d);
        assert_eq!(d.list.first().map(|x| x.code), Some("E001"));
        assert!(
            toks.iter()
                .any(|t| t.tok == Tok::Word("run".into()) && t.nl_before)
        );
    }
}
