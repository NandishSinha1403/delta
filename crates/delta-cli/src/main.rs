//! `delta`: the native command-line tool. Phase 0 knows one command, `delta hello`.

// `std` is Rust's standard library; `ExitCode` is the status a program returns to the shell.
use std::process::ExitCode;

fn main() -> ExitCode {
    // `std::env::args()` yields the command-line words; `.nth(1)` skips the program name.
    // It returns `Option<String>`: either `Some(value)` or `None` (Rust has no null).
    let command = std::env::args().nth(1);

    // `match` picks the first arm whose pattern fits, like a `switch` that must cover every case.
    // `.as_deref()` turns `Option<String>` into `Option<&str>` so we can compare with literals.
    match command.as_deref() {
        Some("hello") => match delta_core::hello::hello_json() {
            Ok(json) => {
                // `println!` prints a line to stdout; `{}` is replaced by the value after the comma.
                println!("{}", json);
                ExitCode::SUCCESS
            }
            Err(e) => {
                // `eprintln!` prints to stderr.
                eprintln!("error: {}", e);
                ExitCode::FAILURE
            }
        },
        // `_` matches anything that did not match an earlier arm.
        _ => {
            eprintln!("usage: delta hello");
            ExitCode::from(2)
        }
    }
}
