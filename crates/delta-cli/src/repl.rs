//! `deltac repl`: type Delta a statement at a time.
//!
//! The session is a growing program. Each entry is run as "everything so far + the entry", and
//! only the outputs of the entry's own lines are printed; an entry with an error is shown and
//! then dropped, so the program always runs cleanly. A lone expression prints its value.

use crate::run::{self, Options};
use crate::term::{self, Protocol};
use rustyline::error::ReadlineError;
use serde_json::Value as Json;
use std::process::ExitCode;

const HELP: &str = "\
Type Delta statements; an expression on its own prints its value.
  :help          this text
  :show on|off   draw shown machines as pictures (when the terminal can)
  :load FILE     run a file and keep its definitions
  :program       print the session so far
  :reset         start again
  :quit          leave (or Ctrl-D)";

/// Words that start a statement, so the entry isn't tried as an expression to print.
const STATEMENT_WORDS: &[&str] = &[
    "let", "var", "fn", "if", "for", "while", "return", "print", "show", "save", "assert", "check",
    "run", "model", "machine", "lang", "use", "budget",
];

pub fn repl() -> ExitCode {
    let mut rl = match rustyline::DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("error: can't start the line editor: {e}");
            return ExitCode::FAILURE;
        }
    };
    let history =
        std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".deltac_history"));
    if let Some(h) = &history {
        let _ = rl.load_history(h);
    }
    println!(
        "Delta {} — :help for commands, Ctrl-D to leave",
        env!("CARGO_PKG_VERSION")
    );
    let mut session = Session {
        program: String::new(),
        opts: Options::default(),
        protocol: Protocol::None,
    };
    let mut entry = String::new();
    loop {
        let prompt = if entry.is_empty() { "δ> " } else { "·· " };
        match rl.readline(prompt) {
            Ok(line) => {
                if entry.is_empty() && line.trim_start().starts_with(':') {
                    let _ = rl.add_history_entry(line.as_str());
                    if !session.command(line.trim()) {
                        break;
                    }
                    continue;
                }
                entry.push_str(&line);
                entry.push('\n');
                // Keep reading while brackets are open (a blank line forces the entry).
                if open_brackets(&entry) > 0 && !line.trim().is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(entry.trim_end());
                session.enter(entry.trim_end());
                entry.clear();
            }
            Err(ReadlineError::Interrupted) => entry.clear(),
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
        }
    }
    if let Some(h) = &history {
        let _ = rl.save_history(h);
    }
    ExitCode::SUCCESS
}

struct Session {
    program: String,
    opts: Options,
    protocol: Protocol,
}

impl Session {
    /// Runs a `:command`. Returns false to leave.
    fn command(&mut self, cmd: &str) -> bool {
        let (name, arg) = cmd
            .split_once(' ')
            .map_or((cmd, ""), |(a, b)| (a, b.trim()));
        match name {
            ":q" | ":quit" | ":exit" => return false,
            ":help" | ":h" => println!("{HELP}"),
            ":reset" => {
                self.program.clear();
                println!("session cleared");
            }
            ":program" => print!("{}", self.program),
            ":show" => {
                self.opts.show = arg != "off";
                self.protocol = if self.opts.show {
                    term::detect()
                } else {
                    Protocol::None
                };
                let how = match self.protocol {
                    Protocol::None if self.opts.show => "as text (this terminal shows no images)",
                    Protocol::None => "as text",
                    _ => "as pictures",
                };
                println!("machines are shown {how}");
            }
            ":load" => match std::fs::read_to_string(arg) {
                Ok(src) => self.enter(&src),
                Err(e) => eprintln!("error: can't read {arg}: {e}"),
            },
            _ => eprintln!("unknown command {name}; :help lists them"),
        }
        true
    }

    /// Runs one entry on top of the session; keeps it if it has no errors.
    fn enter(&mut self, entry: &str) {
        if entry.trim().is_empty() {
            return;
        }
        let base_lines = self.program.lines().count();
        let first = entry.split_whitespace().next().unwrap_or("");
        let single = !entry.contains('\n');
        // A lone expression: try printing it first.
        if single && !STATEMENT_WORDS.contains(&first) {
            let printed = format!("{}print {entry}\n", self.program);
            if let Some(out) = run::evaluate(&printed)
                && new_errors(&out, base_lines).is_empty()
            {
                self.show(&out, base_lines);
                self.program.push_str(entry);
                self.program.push('\n');
                return;
            }
        }
        let candidate = format!("{}{entry}\n", self.program);
        let Some(out) = run::evaluate(&candidate) else {
            eprintln!("error: the engine stopped unexpectedly (X999)");
            return;
        };
        self.show(&out, base_lines);
        let errors = new_errors(&out, base_lines);
        if errors.is_empty() {
            self.program = candidate;
        } else {
            for d in errors {
                eprintln!(
                    "{}:{}: {}[{}]: {}",
                    d["line"]
                        .as_u64()
                        .unwrap_or(0)
                        .saturating_sub(base_lines as u64),
                    d["col"],
                    d["severity"].as_str().unwrap_or(""),
                    d["code"].as_str().unwrap_or(""),
                    d["message"].as_str().unwrap_or("")
                );
            }
        }
    }

    /// Prints the outputs that come from the entry's lines.
    fn show(&self, out: &Json, base_lines: usize) {
        for o in out["outputs"].as_array().into_iter().flatten() {
            if o["line"].as_u64().unwrap_or(0) as usize > base_lines {
                run::print_output(o, &self.opts, self.protocol);
            }
        }
    }
}

/// Errors and budget outcomes on the entry's lines.
fn new_errors(out: &Json, base_lines: usize) -> Vec<Json> {
    out["diagnostics"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| {
            d["severity"] == "error" && d["line"].as_u64().unwrap_or(0) as usize > base_lines
        })
        .cloned()
        .collect()
}

/// How many brackets are still open, ignoring those inside strings and `//` comments.
fn open_brackets(src: &str) -> i64 {
    let mut depth = 0i64;
    for line in src.lines() {
        let mut in_str = false;
        let mut prev = ' ';
        for c in line.chars() {
            if in_str {
                if c == '"' && prev != '\\' {
                    in_str = false;
                }
            } else if c == '"' {
                in_str = true;
            } else if c == '/' && prev == '/' {
                break;
            } else if matches!(c, '{' | '(' | '[') {
                depth += 1;
            } else if matches!(c, '}' | ')' | ']') {
                depth -= 1;
            }
            prev = c;
        }
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_in_strings_and_comments_dont_count() {
        assert_eq!(open_brackets("dfa M {\n  q0 -a-> q1"), 1);
        assert_eq!(open_brackets("print \"{\" // {"), 0);
        assert_eq!(open_brackets("let x = [1, (2"), 2);
    }

    #[test]
    fn entries_build_a_session_and_errors_are_dropped() {
        let mut s = Session {
            program: String::new(),
            opts: Options::default(),
            protocol: Protocol::None,
        };
        s.enter("let x = 2");
        s.enter("let y = nope");
        s.enter("x + 1");
        assert_eq!(s.program, "let x = 2\nx + 1\n");
    }
}
