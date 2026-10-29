//! Diagnostics: user mistakes as data, never as crashes (docs/plan/08-ERRORS.md §1, §5).
//!
//! The checker collects *all* problems in a file in one pass, each with the book definition it
//! breaks and, when possible, a one-click fix.

/// A byte range in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// The range [start, end).
    pub fn new(start: usize, end: usize) -> Span {
        Span {
            start: start as u32,
            end: end as u32,
        }
    }

    /// An empty range at `pos` (where text would be inserted).
    pub fn at(pos: u32) -> Span {
        Span {
            start: pos,
            end: pos,
        }
    }

    /// The smallest range covering both.
    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// How serious a diagnostic is (§3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// The definition a diagnostic is about, quoted on hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookRef {
    pub item: &'static str,
    pub quote: &'static str,
}

/// Replace `span` with `insert`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub span: Span,
    pub insert: String,
}

/// A quick fix: a titled set of edits the editor applies in one go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<TextEdit>,
}

/// One problem in the user's file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// e.g. "E201"
    pub code: &'static str,
    pub severity: Severity,
    pub span: Span,
    pub message: String,
    pub book: Option<BookRef>,
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    /// Builder: attach the book definition this breaks.
    // Returning `&mut Self` lets calls chain: `d.error(..).book(..).fix(..)`.
    pub fn book(&mut self, r: BookRef) -> &mut Self {
        self.book = Some(r);
        self
    }

    /// Builder: attach a quick fix.
    pub fn fix(&mut self, title: impl Into<String>, edits: Vec<TextEdit>) -> &mut Self {
        self.fixes.push(Fix {
            title: title.into(),
            edits,
        });
        self
    }
}

/// Collects diagnostics while a file is checked.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub list: Vec<Diagnostic>,
}

impl Diagnostics {
    fn push(
        &mut self,
        code: &'static str,
        severity: Severity,
        span: Span,
        msg: String,
    ) -> &mut Diagnostic {
        self.list.push(Diagnostic {
            code,
            severity,
            span,
            message: msg,
            book: None,
            fixes: Vec::new(),
        });
        // `list[i]` indexes a Vec (it would panic if out of range); here i = len − 1 is always valid
        // because we pushed one element just above.
        let len = self.list.len();
        &mut self.list[len - 1]
    }

    /// Records an error.
    pub fn error(
        &mut self,
        code: &'static str,
        span: Span,
        msg: impl Into<String>,
    ) -> &mut Diagnostic {
        self.push(code, Severity::Error, span, msg.into())
    }

    /// Records a warning.
    pub fn warning(
        &mut self,
        code: &'static str,
        span: Span,
        msg: impl Into<String>,
    ) -> &mut Diagnostic {
        self.push(code, Severity::Warning, span, msg.into())
    }

    /// Records an info.
    pub fn info(
        &mut self,
        code: &'static str,
        span: Span,
        msg: impl Into<String>,
    ) -> &mut Diagnostic {
        self.push(code, Severity::Info, span, msg.into())
    }

    /// Number of errors so far.
    pub fn error_count(&self) -> usize {
        self.list
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }
}

/// Edit distance between two short strings, for "did you mean …?" suggestions (§7).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev.get(j).copied().unwrap_or(0) + usize::from(ca != cb);
            let del = prev.get(j + 1).copied().unwrap_or(0) + 1;
            let ins = cur.get(j).copied().unwrap_or(0) + 1;
            cur.push(sub.min(del).min(ins));
        }
        prev = cur;
    }
    prev.last().copied().unwrap_or(0)
}

/// The closest candidate within edit distance 2, if any.
// `impl IntoIterator<Item = &'a str>` accepts any list of string slices that live for `'a`.
pub fn suggest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .into_iter()
        .map(|c| (edit_distance(word, c), c))
        .filter(|(d, _)| *d <= 2)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}
