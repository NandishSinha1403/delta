//! Delta 2: the language. `run(source)` parses, checks and evaluates a program and returns the
//! output document of 02-ENGINE.md §7 as JSON.

pub mod ast;
pub mod build;
pub mod builtins;
pub mod check;
pub mod diag;
pub mod docs;
pub mod explain;
pub mod grammar;
pub mod hooks;
pub mod interp;
pub mod label;
pub mod lang;
pub mod lexer;
pub mod machine;
pub mod model;
pub mod pad;
pub mod parser;
pub mod prelude;
pub mod render;
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

/// Everything one evaluation produced.
pub struct Execution<'s> {
    pub program: ast::Program,
    pub diags: Vec<Diagnostic>,
    /// The interpreter after the run (None when errors stopped the program before it ran).
    pub interp: Option<interp::Interp<'s>>,
    pub outputs: Vec<serde_json::Value>,
    pub ok: bool,
    pub steps: u64,
}

/// Parses, checks and runs `src`.
pub fn execute<'s>(src: &'s str, opts: &RunOptions) -> Execution<'s> {
    let mut it = interp::Interp::new(src);
    let mut models: Vec<String> = it.models.keys().cloned().collect();
    models.sort();
    // manual:begin pipeline
    let (program, mut diags) = parser::parse(src, &[]);
    diags.extend(check::check(&program, src, &models));
    let has_errors = diags.iter().any(Diagnostic::is_error);
    let mut outputs = Vec::new();
    let mut steps = 0;
    let mut stopped = false;
    let mut interp = None;
    if !has_errors {
        if let Some(b) = opts.budget {
            it.budget = b;
        }
        if let Some(d) = it.run_program(&program) {
            stopped = true;
            diags.push(d);
        }
        diags.append(&mut it.pending);
        outputs = std::mem::take(&mut it.outputs);
        steps = it.steps;
        interp = Some(it);
    }
    // manual:end pipeline
    diags.sort_by_key(|d| d.span.from);
    let ok = !stopped && !diags.iter().any(Diagnostic::is_error);
    Execution {
        program,
        diags,
        interp,
        outputs,
        ok,
        steps,
    }
}

/// Parses, checks and runs `src`. Never panics: every problem is a diagnostic.
pub fn run(src: &str) -> serde_json::Value {
    run_with(src, &RunOptions::default())
}

pub fn run_with(src: &str, opts: &RunOptions) -> serde_json::Value {
    let ex = execute(src, opts);
    let lines = LineMap::new(src);
    json!({
        "delta_version": VERSION,
        "ok": ex.ok,
        "diagnostics": ex.diags.iter().map(|d| d.to_json(src, &lines)).collect::<Vec<_>>(),
        "outputs": ex.outputs,
        "stats": { "steps": ex.steps },
    })
}
