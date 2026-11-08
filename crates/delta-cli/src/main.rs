//! `delta`: the native command-line tool.
//!
//! - `delta check FILE.delta` prints diagnostics like a compiler and fails if there are errors
//!   or failing checks.
//! - `delta json FILE.delta` prints the evaluated `Document` as JSON (what the playground gets).

// `std` is Rust's standard library; `ExitCode` is the status a program returns to the shell.
use std::process::ExitCode;

fn main() -> ExitCode {
    // `.collect::<Vec<String>>()` gathers the iterator's items into a Vec.
    let args: Vec<String> = std::env::args().collect();
    // Slice patterns match on the shape of the argument list.
    match args.as_slice() {
        [_, cmd, file] if cmd == "check" || cmd == "json" => match std::fs::read_to_string(file) {
            Ok(src) => {
                let doc = delta_dsl::evaluate(&src);
                if cmd == "json" {
                    return match serde_json::to_string_pretty(&doc) {
                        Ok(json) => {
                            println!("{json}");
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: {e}");
                            ExitCode::FAILURE
                        }
                    };
                }
                report(file, &src, &doc)
            }
            Err(e) => {
                // `eprintln!` prints to stderr.
                eprintln!("error: cannot read {file}: {e}");
                ExitCode::FAILURE
            }
        },
        // Delta 2: `delta run [--json] FILE`.
        [_, cmd, rest @ ..] if cmd == "run" => {
            // `rest @ ..` binds the remaining arguments as a slice.
            let json = rest.iter().any(|a| a == "--json");
            let Some(file) = rest.iter().find(|a| !a.starts_with("--")) else {
                eprintln!("usage: delta run [--json] FILE.delta");
                return ExitCode::from(2);
            };
            match std::fs::read_to_string(file) {
                Ok(src) => run_v2(file, &src, json),
                Err(e) => {
                    eprintln!("error: cannot read {file}: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        // `_` matches anything that did not match an earlier arm.
        _ => {
            eprintln!(
                "usage: delta run [--json] FILE.delta | delta check FILE.delta | delta json FILE.delta"
            );
            ExitCode::from(2)
        }
    }
}

/// Prints diagnostics as `file:line:col: severity[CODE]: message` and the check results.
fn report(file: &str, src: &str, doc: &delta_dsl::document::Document) -> ExitCode {
    let utf16: Vec<u16> = src.encode_utf16().collect();
    let line_col = |offset: u32| {
        // `.get(..n)` takes the first n items if there are that many.
        let before = utf16.get(..offset as usize).unwrap_or(&utf16);
        let line = before.iter().filter(|&&u| u == u16::from(b'\n')).count() + 1;
        let col = before
            .iter()
            .rev()
            .take_while(|&&u| u != u16::from(b'\n'))
            .count()
            + 1;
        (line, col)
    };
    let mut errors = 0;
    for d in &doc.diagnostics {
        let (l, c) = line_col(d.from);
        println!("{file}:{l}:{c}: {}[{}]: {}", d.severity, d.code, d.message);
        if d.severity == "error" {
            errors += 1;
        }
    }
    let mut failed = 0;
    for r in &doc.results {
        for case in &r.cases {
            let ok = case.expected == case.actual;
            if !ok {
                failed += 1;
            }
            let verb = if case.expected { "accepts" } else { "rejects" };
            println!(
                "  {} {} {verb} \"{}\"",
                if ok { "✓" } else { "✗" },
                r.object,
                case.input
            );
        }
    }
    println!(
        "{} objects, {errors} errors, {failed} failing checks",
        doc.objects.len()
    );
    if errors > 0 || failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Runs a Delta 2 program. Exits nonzero on errors or failed tests (Q-48).
fn run_v2(file: &str, src: &str, json: bool) -> ExitCode {
    // Run on a thread with a large stack: deep user recursion stays within the depth budget.
    let src_owned = src.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || delta_lang::run(&src_owned));
    let out = match handle.map(|h| h.join()) {
        Ok(Ok(v)) => v,
        _ => {
            eprintln!("error: the engine stopped unexpectedly (X999)");
            return ExitCode::FAILURE;
        }
    };
    let failed_tests = out["outputs"]
        .as_array()
        .map(|o| {
            o.iter()
                .filter(|x| x["kind"] == "test" && x["pass"] == false)
                .count()
        })
        .unwrap_or(0);
    let ok = out["ok"] == true;
    if json {
        match serde_json::to_string_pretty(&out) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        for o in out["outputs"].as_array().into_iter().flatten() {
            match o["kind"].as_str() {
                Some("print") => println!("{}", o["text"].as_str().unwrap_or("")),
                Some("test") => {
                    let mark = if o["pass"] == true { "✓" } else { "✗" };
                    let msg = o["message"]
                        .as_str()
                        .map(|m| format!("  ({m})"))
                        .unwrap_or_default();
                    println!(
                        "{mark} line {}: {}{msg}",
                        o["line"],
                        o["name"].as_str().unwrap_or("")
                    );
                }
                _ => println!("{o}"),
            }
        }
        for d in out["diagnostics"].as_array().into_iter().flatten() {
            println!(
                "{file}:{}:{}: {}[{}]: {}",
                d["line"],
                d["col"],
                d["severity"].as_str().unwrap_or(""),
                d["code"].as_str().unwrap_or(""),
                d["message"].as_str().unwrap_or("")
            );
        }
    }
    if ok && failed_tests == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
