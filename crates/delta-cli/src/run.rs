//! `deltac run`: runs a Delta 2 program, prints its outputs, and writes the files it `save`s.

use crate::term::{self, Protocol};
use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// What `deltac run` was asked to do.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Print the whole result as JSON (the output schema) and write no files.
    pub json: bool,
    /// Draw `show`n machines in the terminal (as images when it can).
    pub show: bool,
    /// Also write every `show`n machine to `out` in this format.
    pub format: Option<String>,
    pub out: Option<PathBuf>,
    /// Dark drawings (for dark terminals).
    pub dark: bool,
}

/// The formats `--format` takes, with the file extension each writes.
pub const FORMATS: &[(&str, &str)] = &[
    ("svg", "svg"),
    ("png", "png"),
    ("pdf", "pdf"),
    ("tikz", "tex"),
    ("txt", "txt"),
];

/// Runs the engine on a thread with a large stack: deep user recursion stays within the
/// depth budget instead of overflowing the main thread's stack.
pub fn evaluate(src: &str) -> Option<Json> {
    let src = src.to_string();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || delta_lang::run(&src))
        .ok()?
        .join()
        .ok()
}

/// Runs a program. Exits nonzero on errors or failed tests (Q-48).
pub fn run(file: &str, src: &str, opts: &Options) -> ExitCode {
    let Some(out) = evaluate(src) else {
        eprintln!("error: the engine stopped unexpectedly (X999)");
        return ExitCode::FAILURE;
    };
    let outputs: Vec<Json> = out["outputs"].as_array().cloned().unwrap_or_default();
    let failed_tests = outputs
        .iter()
        .filter(|x| x["kind"] == "test" && x["pass"] == false)
        .count();
    let mut ok = out["ok"] == true;
    if opts.json {
        match serde_json::to_string_pretty(&out) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        let protocol = if opts.show {
            term::detect()
        } else {
            Protocol::None
        };
        for o in &outputs {
            if !print_output(o, opts, protocol) {
                ok = false;
            }
        }
        if let Some(fmt) = &opts.format
            && !export_shown(&outputs, fmt, opts)
        {
            ok = false;
        }
        print_diagnostics(file, &out);
    }
    if ok && failed_tests == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Prints one output the way a person reads it. Returns false if a `save` failed.
pub fn print_output(o: &Json, opts: &Options, protocol: Protocol) -> bool {
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
        Some("show") => {
            let v = &o["value"];
            if v["type"] == "machine" {
                if opts.show {
                    term::show_machine(v, protocol, opts.dark);
                } else {
                    print!("{}", delta_lang::render::text_of(v).unwrap_or_default());
                }
            } else {
                println!("{}", value_text(v));
            }
        }
        Some("trace") => println!(
            "run {} on \"{}\": {}",
            o["machine"].as_str().unwrap_or("?"),
            o["input"].as_str().unwrap_or(""),
            o["verdict"].as_str().unwrap_or("?")
        ),
        Some("save") => return save(o, opts.dark),
        _ => println!("{o}"),
    }
    true
}

/// A value's text (see `delta_lang::render::value_text`).
pub fn value_text(v: &Json) -> String {
    delta_lang::render::value_text(v)
}

/// The bytes of a file in `format` for a `show` value; Err says why not.
pub fn file_bytes(
    format: &str,
    value: &Json,
    text: Option<&str>,
    dark: bool,
) -> Result<Vec<u8>, String> {
    match format {
        "png" | "pdf" => {
            let svg = delta_lang::render::text_file("svg", value, text, dark)?;
            if format == "png" {
                delta_export::png(&svg, 2.0)
            } else {
                delta_export::pdf(&svg)
            }
        }
        other => delta_lang::render::text_file(other, value, text, dark).map(String::into_bytes),
    }
}

/// Writes a `save` output's file inside the current folder.
fn save(o: &Json, dark: bool) -> bool {
    let path = o["path"].as_str().unwrap_or("");
    let format = o["format"].as_str().unwrap_or("");
    let result = file_bytes(format, &o["value"], o["text"].as_str(), dark)
        .and_then(|bytes| write_inside_cwd(path, &bytes));
    match result {
        Ok(p) => {
            println!("saved {}", p.display());
            true
        }
        Err(e) => {
            eprintln!("error: line {}: can't save {path}: {e}", o["line"]);
            false
        }
    }
}

/// Writes `rel` under the current folder, creating folders; refuses anything that would end up
/// outside it (through `..`, an absolute path or a symbolic link).
pub fn write_inside_cwd(rel: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir()
        .and_then(|d| d.canonicalize())
        .map_err(|e| e.to_string())?;
    // The engine already checked the path (SEMANTICS §10.4); check again before writing.
    if delta_lang::interp::save_path(rel).as_deref() != Some(rel) {
        return Err("the path must be relative and stay inside this folder".into());
    }
    let target = cwd.join(rel);
    write_inside(&cwd, &target, bytes)?;
    Ok(PathBuf::from(rel))
}

/// Writes `target`, which must end up inside `root` (both checked after resolving links).
fn write_inside(root: &Path, target: &Path, bytes: &[u8]) -> Result<(), String> {
    // The deepest folder that exists already must resolve to somewhere inside root.
    let parent = target.parent().ok_or("no folder")?;
    let mut existing = parent;
    while !existing.exists() {
        existing = existing.parent().ok_or("no folder")?;
    }
    let real = existing.canonicalize().map_err(|e| e.to_string())?;
    if !real.starts_with(root) {
        return Err("the path leads outside this folder".into());
    }
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    if target
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err("the file is a symbolic link".into());
    }
    std::fs::write(target, bytes).map_err(|e| e.to_string())
}

/// `--format F --out DIR`: writes every shown machine as `DIR/line-N.ext`.
fn export_shown(outputs: &[Json], format: &str, opts: &Options) -> bool {
    let ext = FORMATS
        .iter()
        .find(|(f, _)| *f == format)
        .map_or(format, |(_, e)| e);
    let dir = opts.out.clone().unwrap_or_else(|| PathBuf::from("."));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: can't create {}: {e}", dir.display());
        return false;
    }
    let mut ok = true;
    for o in outputs
        .iter()
        .filter(|o| o["kind"] == "show" && o["value"]["type"] == "machine")
    {
        let path = dir.join(format!("line-{}.{ext}", o["line"]));
        match file_bytes(format, &o["value"], None, opts.dark)
            .and_then(|b| std::fs::write(&path, b).map_err(|e| e.to_string()))
        {
            Ok(()) => println!("wrote {}", path.display()),
            Err(e) => {
                eprintln!("error: can't write {}: {e}", path.display());
                ok = false;
            }
        }
    }
    ok
}

/// `file:line:col: severity[CODE]: message`, like a compiler.
pub fn print_diagnostics(file: &str, out: &Json) {
    for d in out["diagnostics"].as_array().into_iter().flatten() {
        eprintln!(
            "{file}:{}:{}: {}[{}]: {}",
            d["line"],
            d["col"],
            d["severity"].as_str().unwrap_or(""),
            d["code"].as_str().unwrap_or(""),
            d["message"].as_str().unwrap_or("")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("deltac-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::create_dir_all(&d);
        d.canonicalize().unwrap_or(d)
    }

    #[test]
    fn writes_inside_and_refuses_links_out() {
        let root = temp_dir("inside");
        assert!(write_inside(&root, &root.join("a/b/c.txt"), b"x").is_ok());
        assert_eq!(
            std::fs::read(root.join("a/b/c.txt")).ok(),
            Some(b"x".to_vec())
        );
        #[cfg(unix)]
        {
            let outside = temp_dir("outside");
            let _ = std::os::unix::fs::symlink(&outside, root.join("link"));
            assert!(write_inside(&root, &root.join("link/x.txt"), b"x").is_err());
            assert!(!outside.join("x.txt").exists());
        }
    }

    #[test]
    fn every_format_has_bytes_for_a_machine() {
        let out = delta_lang::run("dfa M {\n start q0; accept q0\n q0 -a-> q0\n}\nshow M");
        let v = &out["outputs"][0]["value"];
        for f in ["svg", "png", "pdf", "tex", "txt", "json"] {
            let b = file_bytes(f, v, None, false);
            assert!(b.as_ref().is_ok_and(|b| !b.is_empty()), "{f}: {b:?}");
        }
        assert!(file_bytes("svg", &serde_json::json!({ "type": "set" }), None, false).is_err());
    }
}
