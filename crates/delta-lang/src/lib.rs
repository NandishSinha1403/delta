//! Delta 2: the language. `run(source)` parses, checks and evaluates a program and returns the
//! output document of 02-ENGINE.md §7 as JSON.

pub mod ast;
pub mod build;
pub mod builtins;
pub mod check;
pub mod diag;
pub mod explain;
pub mod grammar;
pub mod hooks;
pub mod interp;
pub mod label;
pub mod lang;
pub mod lexer;
pub mod machine;
pub mod model;
pub mod parser;
pub mod prelude;
pub mod rx;
pub mod sim;
pub mod value;

use diag::{Diagnostic, LineMap};
use serde_json::json;

pub const VERSION: &str = "2.0.0-dev";

/// Options for one run.
#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    /// Overrides the default budget (a `budget { … }` directive still applies on top).
    pub budget: Option<interp::Budget>,
}

/// Parses, checks and runs `src`. Never panics: every problem is a diagnostic.
pub fn run(src: &str) -> serde_json::Value {
    run_with(src, &RunOptions::default())
}

pub fn run_with(src: &str, opts: &RunOptions) -> serde_json::Value {
    let models: Vec<String> = parser::PRELUDE_MODELS
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (prog, mut diags) = parser::parse(src, &[]);
    diags.extend(check::check(&prog, src, &models));
    let has_errors = diags.iter().any(Diagnostic::is_error);
    let mut outputs = Vec::new();
    let mut steps = 0;
    let mut stopped = false;
    if !has_errors {
        let mut it = interp::Interp::new(src);
        if let Some(b) = opts.budget {
            it.budget = b;
        }
        if let Some(d) = it.run_program(&prog) {
            stopped = true;
            diags.push(d);
        }
        diags.append(&mut it.pending);
        outputs = std::mem::take(&mut it.outputs);
        steps = it.steps;
    }
    diags.sort_by_key(|d| d.span.from);
    let lines = LineMap::new(src);
    let ok = !stopped && !diags.iter().any(Diagnostic::is_error);
    json!({
        "delta_version": VERSION,
        "ok": ok,
        "diagnostics": diags.iter().map(|d| d.to_json(src, &lines)).collect::<Vec<_>>(),
        "outputs": outputs,
        "stats": { "steps": steps },
    })
}
