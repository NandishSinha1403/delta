//! `deltac`: the Delta command-line tool (named so it doesn't clash with git-delta's `delta`).
//!
//! - `deltac run [--json] [--show] [--format F] [--out DIR] [--theme T] FILE` runs a Delta 2
//!   program (`-` reads it from stdin).
//! - `deltac repl` starts an interactive session.
//! - `deltac check FILE` / `deltac json FILE` are the Delta 1 checker (kept for the v1 Pad).
//! - `deltac gen-docs SITE` writes the generated reference pages.

mod og;
mod repl;
mod run;
mod term;

// `std` is Rust's standard library; `ExitCode` is the status a program returns to the shell.
use std::process::ExitCode;

const USAGE: &str = "\
usage:
  deltac run [options] FILE     run a Delta program (FILE = - reads stdin)
  deltac repl                   type Delta interactively
  deltac check FILE             Delta 1: print diagnostics and check results
  deltac json FILE              Delta 1: print the evaluated document as JSON
  deltac --version | --help

run options:
  --json          print the result as JSON (schema: /schema/output-v2.json); writes no files
  --show          draw shown machines in the terminal (kitty, iTerm2 or sixel; else as text)
  --format F      also write every shown machine as F: svg, png, pdf, tikz or txt
  --out DIR       folder for --format files (default: the current folder)
  --theme T       light (default) or dark drawings; `save … (theme: …)` overrides it

Files named by `save x to \"file\"` are written inside the current folder.";

fn main() -> ExitCode {
    // `.collect::<Vec<String>>()` gathers the iterator's items into a Vec.
    let args: Vec<String> = std::env::args().collect();
    // Slice patterns match on the shape of the argument list.
    match args.as_slice() {
        [_, flag] if flag == "--version" || flag == "-V" => {
            println!("deltac {} (Delta 2 preview)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [_, flag] if flag == "--help" || flag == "-h" || flag == "help" => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [_, cmd] if cmd == "repl" => repl::repl(),
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
        // Generates the docs site's reference pages from the signature records.
        [_, cmd, dir] if cmd == "gen-docs" => match gen_docs(std::path::Path::new(dir)) {
            Ok(n) => {
                println!("wrote {n} files under {dir}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        // `rest @ ..` binds the remaining arguments as a slice.
        [_, cmd, rest @ ..] if cmd == "run" => match parse_run(rest) {
            Ok((file, opts)) => {
                let src = if file == "-" {
                    std::io::read_to_string(std::io::stdin())
                } else {
                    std::fs::read_to_string(&file)
                };
                match src {
                    Ok(src) => run::run(&file, &src, &opts),
                    Err(e) => {
                        eprintln!("error: cannot read {file}: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
            Err(msg) => {
                eprintln!("error: {msg}\n\n{USAGE}");
                ExitCode::from(2)
            }
        },
        // `_` matches anything that did not match an earlier arm.
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Reads `run`'s options by hand (no argument-parsing crate).
fn parse_run(args: &[String]) -> Result<(String, run::Options), String> {
    let mut opts = run::Options::default();
    let mut file = None;
    // `iter()` borrows each item; `next()` takes the following one (the flag's value).
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => opts.json = true,
            "--show" => opts.show = true,
            // `--dark` is the old spelling of `--theme dark` (kept, not listed in the help).
            "--dark" => opts.dark = true,
            "--theme" => {
                opts.dark = match it.next().map(String::as_str) {
                    Some("light") => false,
                    Some("dark") => true,
                    _ => return Err("--theme needs `light` or `dark`".into()),
                };
            }
            "--format" => {
                let f = it.next().ok_or("--format needs a value")?;
                if !run::FORMATS.iter().any(|(name, _)| name == f) {
                    return Err(format!("unknown format `{f}` (svg, png, pdf, tikz, txt)"));
                }
                opts.format = Some(f.clone());
            }
            "--out" => opts.out = Some(it.next().ok_or("--out needs a folder")?.into()),
            flag if flag.starts_with("--") => return Err(format!("unknown option `{flag}`")),
            path => {
                if file.replace(path.to_string()).is_some() {
                    return Err("give one file".into());
                }
            }
        }
    }
    if opts.out.is_some() && opts.format.is_none() {
        opts.format = Some("svg".into());
    }
    file.map(|f| (f, opts))
        .ok_or_else(|| "no file given".into())
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

/// Writes the generated docs: library, errors, AI guide, llms.txt and the output schema.
/// `site` is the docs site's root folder (`site/`).
fn gen_docs(site: &std::path::Path) -> std::io::Result<usize> {
    use delta_lang::docs;
    let docs_dir = site.join("src/content/docs/docs");
    let public = site.join("public");
    let mut files: Vec<(std::path::PathBuf, String)> =
        vec![(docs_dir.join("library.md"), docs::library_page())];
    for (path, text) in docs::error_pages() {
        files.push((docs_dir.join(path), text));
    }
    let guide = docs::ai_guide();
    files.push((public.join("llms.txt"), guide.clone()));
    files.push((
        docs_dir.join("ai.md"),
        format!(
            "---\ntitle: AI\ndescription: \"Using Delta with AI assistants: the guide, llms.txt and the JSON schema.\"\n---\n\n<!-- Generated by `deltac gen-docs`. Do not edit. -->\n\nGive an assistant [`/llms.txt`](/llms.txt) (the text below) and ask it to write Delta. Run its program with\n`deltac run --json FILE`; the output follows [`/schema/output-v2.json`](/schema/output-v2.json).\n\n{}",
            guide.lines().skip(1).collect::<Vec<_>>().join("\n").replace("\n## ", "\n### ")
        ),
    ));
    let schema = serde_json::to_string_pretty(&docs::output_schema()).unwrap_or_default();
    files.push((public.join("schema/output-v2.json"), schema));
    for (path, text) in &files {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)?;
    }
    // Social cards for pages whose first example shows a machine.
    let cards = og::write_cards(site)?;
    Ok(files.len() + cards.len())
}
