//! Diagnostics: errors, warnings and budget outcomes, with byte spans and quick fixes.

use serde::Serialize;

/// A byte range `from..to` in the source.
// `#[derive(...)]` asks the compiler to write standard trait implementations for us.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub from: usize,
    pub to: usize,
}

impl Span {
    pub fn new(from: usize, to: usize) -> Span {
        Span { from, to }
    }
    /// The smallest span covering both.
    pub fn join(self, other: Span) -> Span {
        Span::new(self.from.min(other.from), self.to.max(other.to))
    }
}

/// How serious a diagnostic is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// One text edit of a quick fix: replace `span` with `insert`.
#[derive(Clone, Debug)]
pub struct Edit {
    pub span: Span,
    pub insert: String,
}

/// A quick fix: a title and the edits that apply it.
#[derive(Clone, Debug)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<Edit>,
}

/// A diagnostic. `code` is stable (E0xx lexical, E1xx syntax, E2xx names and types, E3xx machine
/// and model rules, E4xx regex and grammar, E5xx runtime, B6xx budget, E7xx decidability).
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub span: Span,
    pub message: String,
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    pub fn error(code: &'static str, span: Span, message: impl Into<String>) -> Diagnostic {
        // `impl Into<String>` accepts anything convertible to a String (a &str or a String).
        Diagnostic {
            code,
            severity: Severity::Error,
            span,
            message: message.into(),
            fixes: Vec::new(),
        }
    }
    pub fn warning(code: &'static str, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            ..Diagnostic::error(code, span, message)
        }
    }
    /// Adds a quick fix that replaces `span` with `insert`.
    pub fn with_fix(
        mut self,
        title: impl Into<String>,
        span: Span,
        insert: impl Into<String>,
    ) -> Diagnostic {
        self.fixes.push(Fix {
            title: title.into(),
            edits: vec![Edit {
                span,
                insert: insert.into(),
            }],
        });
        self
    }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// Maps byte offsets to 1-based lines and 1-based UTF-16 columns (what editors use).
pub struct LineMap {
    /// Byte offset of the start of each line.
    starts: Vec<usize>,
}

impl LineMap {
    pub fn new(src: &str) -> LineMap {
        let mut starts = vec![0];
        // `char_indices` yields (byte offset, char) pairs.
        for (i, c) in src.char_indices() {
            if c == '\n' {
                starts.push(i + 1);
            }
        }
        LineMap { starts }
    }

    /// (line, col), both 1-based; col counts UTF-16 code units.
    pub fn line_col(&self, src: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(src.len());
        // `partition_point` is a binary search: the number of line starts ≤ offset.
        let line = self.starts.partition_point(|&s| s <= offset).max(1);
        let start = self.starts.get(line - 1).copied().unwrap_or(0);
        let col = src
            .get(start..offset)
            .map(|s| s.encode_utf16().count())
            .unwrap_or(0);
        (line, col + 1)
    }

    /// UTF-16 offset of a byte offset.
    pub fn utf16(src: &str, offset: usize) -> usize {
        src.get(..offset.min(src.len()))
            .map(|s| s.encode_utf16().count())
            .unwrap_or(0)
    }
}

/// A diagnostic as JSON (02-ENGINE.md §7).
#[derive(Serialize, Debug, Clone)]
pub struct DiagnosticJson {
    pub code: String,
    pub severity: String,
    pub line: usize,
    pub col: usize,
    pub from: usize,
    pub to: usize,
    pub message: String,
    pub fixes: Vec<FixJson>,
}

#[derive(Serialize, Debug, Clone)]
pub struct FixJson {
    pub title: String,
    pub edits: Vec<EditJson>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EditJson {
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

impl Diagnostic {
    pub fn to_json(&self, src: &str, lines: &LineMap) -> DiagnosticJson {
        let (line, col) = lines.line_col(src, self.span.from);
        DiagnosticJson {
            code: self.code.to_string(),
            severity: self.severity.as_str().to_string(),
            line,
            col,
            from: LineMap::utf16(src, self.span.from),
            to: LineMap::utf16(src, self.span.to),
            message: self.message.clone(),
            fixes: self
                .fixes
                .iter()
                .map(|f| FixJson {
                    title: f.title.clone(),
                    edits: f
                        .edits
                        .iter()
                        .map(|e| EditJson {
                            from: LineMap::utf16(src, e.span.from),
                            to: LineMap::utf16(src, e.span.to),
                            insert: e.insert.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
