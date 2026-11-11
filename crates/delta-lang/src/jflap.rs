//! JFLAP 7 files (`.jff`), both ways (ids `import/jflap` and `export/jflap`).
//!
//! Import turns a file into Delta source for one declaration, which the interpreter then runs as
//! if it had been written in the program; so an imported machine is an ordinary Delta value.
//! Export writes a shown machine or grammar as a `.jff` file. Supported types:
//!
//! | JFLAP `<type>` | Delta           | notes |
//! |----------------|-----------------|-------|
//! | `fa`           | `dfa` or `nfa`  | `dfa` when every state has at most one arrow per symbol and no empty move; a read of several characters becomes a chain of new states |
//! | `pda`          | `pda`           | JFLAP starts with `Z` on the stack: import adds a new start state that pushes it |
//! | `turing`       | `tm`            | one tape; final states accept, halting elsewhere rejects |
//! | `grammar`      | `cfg`           | one character per symbol; upper-case letters are variables |
//!
//! Other types (Mealy, Moore, multi-tape, building blocks) are refused with a message.

use crate::label::Dir;
use crate::label::{self, LOp};
use serde_json::Value as Json;
use std::collections::{BTreeSet, HashMap};

/// One JFLAP transition, as read from the file.
struct Tr {
    from: String,
    to: String,
    read: String,
    pop: String,
    push: String,
    write: Option<String>,
    mv: String,
}

/// The text of `<tag>` under `node`, or "" when missing or empty (JFLAP's λ / blank).
fn child_text(node: roxmltree::Node, tag: &str) -> String {
    node.children()
        .find(|c| c.has_tag_name(tag))
        .and_then(|c| c.text())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn has_child(node: roxmltree::Node, tag: &str) -> bool {
    node.children().any(|c| c.has_tag_name(tag))
}

/// A state name Delta can read: JFLAP's own name when it is an identifier and unused, else q<id>.
fn state_names(states: &[(String, String)]) -> HashMap<String, String> {
    let ident = |s: &str| {
        let mut cs = s.chars();
        cs.next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let mut taken = BTreeSet::new();
    let mut out = HashMap::new();
    for (id, name) in states {
        let mut n = if ident(name) && !taken.contains(name) {
            name.clone()
        } else {
            format!("q{id}")
        };
        while taken.contains(&n) {
            n.push('_');
        }
        taken.insert(n.clone());
        out.insert(id.clone(), n);
    }
    out
}

/// A symbol as a label writes it: letters and digits bare, anything else quoted.
fn sym(s: &str) -> String {
    if s.chars().count() == 1 && s.chars().all(char::is_alphanumeric) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', ""))
    }
}

/// Converts a `.jff` file into Delta source declaring `name`.
pub fn to_delta(xml: &str, name: &str) -> Result<String, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("not a JFLAP file: {e}"))?;
    let root = doc.root_element();
    if !root.has_tag_name("structure") {
        return Err("not a JFLAP file: the root element isn't <structure>".to_string());
    }
    let kind = child_text(root, "type");
    if kind == "grammar" {
        return grammar_to_delta(root, name);
    }
    if !matches!(kind.as_str(), "fa" | "pda" | "turing") {
        return Err(format!(
            "JFLAP type \"{kind}\" isn't supported (fa, pda, turing and grammar are)"
        ));
    }
    let auto = root
        .children()
        .find(|c| c.has_tag_name("automaton"))
        .unwrap_or(root);
    if auto.children().any(|c| c.has_tag_name("block")) {
        return Err("JFLAP building blocks aren't supported".to_string());
    }
    if kind == "turing" && child_text(root, "tapes").parse::<u32>().unwrap_or(1) > 1 {
        return Err("multi-tape JFLAP machines aren't supported".to_string());
    }
    let mut states = Vec::new();
    let mut start = None;
    let mut finals = Vec::new();
    for s in auto.children().filter(|c| c.has_tag_name("state")) {
        let id = s.attribute("id").unwrap_or("").to_string();
        let nm = s.attribute("name").unwrap_or(&id).to_string();
        if has_child(s, "initial") {
            start = Some(id.clone());
        }
        if has_child(s, "final") {
            finals.push(id.clone());
        }
        states.push((id, nm));
    }
    let names = state_names(&states);
    let nm = |id: &str| names.get(id).cloned().unwrap_or_else(|| format!("q{id}"));
    let start = start.ok_or("the JFLAP machine has no initial state")?;
    let trs: Vec<Tr> = auto
        .children()
        .filter(|c| c.has_tag_name("transition"))
        .map(|t| Tr {
            from: nm(&child_text(t, "from")),
            to: nm(&child_text(t, "to")),
            read: child_text(t, "read"),
            pop: child_text(t, "pop"),
            push: child_text(t, "push"),
            write: t
                .children()
                .find(|c| c.has_tag_name("write"))
                .map(|c| c.text().unwrap_or("").trim().to_string()),
            mv: child_text(t, "move"),
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    let mut all: Vec<String> = states.iter().map(|(id, _)| nm(id)).collect();
    let fresh = |all: &mut Vec<String>, base: &str| {
        let mut n = base.to_string();
        let mut i = 1;
        while all.contains(&n) {
            n = format!("{base}{i}");
            i += 1;
        }
        all.push(n.clone());
        n
    };
    let mut first = nm(&start);
    let model = match kind.as_str() {
        "fa" => {
            for t in &trs {
                let cs: Vec<char> = t.read.chars().collect();
                if cs.len() <= 1 {
                    let l = cs
                        .first()
                        .map(|c| sym(&c.to_string()))
                        .unwrap_or("ε".to_string());
                    lines.push(format!("{} -{l}-> {}", t.from, t.to));
                } else {
                    // A read of several characters: one new state per inner character.
                    let mut at = t.from.clone();
                    for (i, c) in cs.iter().enumerate() {
                        let next = if i + 1 == cs.len() {
                            t.to.clone()
                        } else {
                            fresh(&mut all, &format!("{}_{}", t.from, i + 1))
                        };
                        lines.push(format!("{at} -{}-> {next}", sym(&c.to_string())));
                        at = next;
                    }
                }
            }
            let mut seen = BTreeSet::new();
            let det = trs.iter().all(|t| {
                t.read.chars().count() == 1 && seen.insert((t.from.clone(), t.read.clone()))
            });
            if det { "dfa" } else { "nfa" }
        }
        "pda" => {
            // JFLAP's stack starts with Z; a new start state pushes it.
            let s0 = fresh(&mut all, "start");
            lines.push(format!("{s0} -ε, ε → Z-> {first}"));
            first = s0;
            for t in &trs {
                let one = |s: &str| {
                    if s.is_empty() {
                        "ε".to_string()
                    } else {
                        s.chars()
                            .map(|c| sym(&c.to_string()))
                            .collect::<Vec<_>>()
                            .join("")
                    }
                };
                if t.read.chars().count() > 1 || t.pop.chars().count() > 1 {
                    return Err(format!(
                        "a JFLAP pushdown move reads or pops several symbols ({} → {}); Delta reads and pops one at a time",
                        t.from, t.to
                    ));
                }
                lines.push(format!(
                    "{} -{}, {} → {}-> {}",
                    t.from,
                    one(&t.read),
                    one(&t.pop),
                    one(&t.push),
                    t.to
                ));
            }
            "pda"
        }
        _ => {
            for t in &trs {
                let cell = |s: &str| {
                    if s.is_empty() {
                        "⊔".to_string()
                    } else {
                        sym(s)
                    }
                };
                let mv = match t.mv.as_str() {
                    "L" => "L",
                    "S" => "S",
                    _ => "R",
                };
                let read = cell(&t.read);
                let write = t.write.as_deref().map(cell).unwrap_or_else(|| read.clone());
                let body = if write == read {
                    format!("{read} → {mv}")
                } else {
                    format!("{read} → {write}, {mv}")
                };
                lines.push(format!("{} -{body}-> {}", t.from, t.to));
            }
            "tm"
        }
    };
    let mut out = format!("{model} {name} {{\n");
    out.push_str(&format!("  start {first}\n"));
    let accept: Vec<String> = finals.iter().map(|f| nm(f)).collect();
    if !accept.is_empty() {
        out.push_str(&format!("  accept {}\n", accept.join(", ")));
    }
    for l in &lines {
        out.push_str(&format!("  {l}\n"));
    }
    out.push('}');
    Ok(out)
}

fn grammar_to_delta(root: roxmltree::Node, name: &str) -> Result<String, String> {
    let mut rules: Vec<(String, String)> = Vec::new();
    for p in root.children().filter(|c| c.has_tag_name("production")) {
        rules.push((child_text(p, "left"), child_text(p, "right")));
    }
    let Some((start, _)) = rules.first() else {
        return Err("the JFLAP grammar has no productions".to_string());
    };
    let start = start.clone();
    // Rules for the same variable are joined with | in first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut alts: HashMap<String, Vec<String>> = HashMap::new();
    for (l, r) in &rules {
        if l.chars().count() != 1 {
            return Err(format!(
                "the JFLAP rule `{l} → {r}` has more than one symbol on the left"
            ));
        }
        if !order.contains(l) {
            order.push(l.clone());
        }
        let rhs = if r.is_empty() {
            "ε".to_string()
        } else {
            r.chars()
                .map(|c| sym(&c.to_string()))
                .collect::<Vec<_>>()
                .join(" ")
        };
        alts.entry(l.clone()).or_default().push(rhs);
    }
    let mut out = format!("cfg {name} {{\n  start {start}\n");
    for l in &order {
        out.push_str(&format!(
            "  {l} -> {}\n",
            alts.get(l).map(|a| a.join(" | ")).unwrap_or_default()
        ));
    }
    out.push('}');
    Ok(out)
}

/// XML text: &, < and > escaped.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A symbol as JFLAP writes it: the blank and ε are empty.
fn jsym(s: &str) -> String {
    if matches!(s, "⊔" | "□" | "ε" | "λ") {
        String::new()
    } else {
        esc(s)
    }
}

/// A shown machine or grammar (its `show` value) as a `.jff` file.
pub fn from_view(view: &Json) -> Result<String, String> {
    if view["type"] == "grammar" || view["type"] == "cfg" {
        return grammar_from_view(view);
    }
    if view["type"] != "machine" {
        return Err("only machines and grammars can be saved as .jff".to_string());
    }
    let model_name = view["model"].as_str().unwrap_or("");
    let jtype = match model_name {
        "dfa" | "nfa" => "fa",
        "pda" | "dpda" => "pda",
        "tm" | "ntm" | "lba" => "turing",
        other => {
            return Err(format!(
                "a {other} can't be saved as .jff (dfa, nfa, pda and tm can)"
            ));
        }
    };
    let models = crate::prelude::models();
    let model = models.get(model_name).ok_or("unknown model")?;
    let strs = |k: &str| -> Vec<String> {
        view[k]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let states = strs("states");
    let accept = strs("accept");
    let start = view["start"].as_str().unwrap_or("");
    let id = |s: &str| states.iter().position(|x| x == s).unwrap_or(0);
    let mut x = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?><!--Created with Delta.-->\n<structure>\n",
    );
    x.push_str(&format!("\t<type>{jtype}</type>\n\t<automaton>\n"));
    // Positions on a circle, so JFLAP opens the file with the states spread out.
    let n = states.len().max(1) as f64;
    for (i, s) in states.iter().enumerate() {
        let a = std::f64::consts::TAU * i as f64 / n;
        x.push_str(&format!(
            "\t\t<state id=\"{i}\" name=\"{}\">\n\t\t\t<x>{:.1}</x>\n\t\t\t<y>{:.1}</y>\n",
            esc(s),
            300.0 + 200.0 * a.cos(),
            300.0 + 200.0 * a.sin()
        ));
        if s == start {
            x.push_str("\t\t\t<initial/>\n");
        }
        if accept.contains(s) {
            x.push_str("\t\t\t<final/>\n");
        }
        x.push_str("\t\t</state>\n");
    }
    let blank = label::blank();
    for e in view["edges"].as_array().into_iter().flatten() {
        let (Some(f), Some(text), Some(t)) = (e[0].as_str(), e[1].as_str(), e[2].as_str()) else {
            continue;
        };
        let labels = label::parse_labels(text, model, &blank)
            .map_err(|m| format!("the label `{text}` can't be written as JFLAP: {m}"))?;
        for l in labels {
            let mut read = String::new();
            let mut pop = String::new();
            let mut push = String::new();
            let mut write = None;
            let mut mv = "S".to_string();
            for op in &l.ops {
                match op {
                    LOp::Read(_, s) => read = jsym(s),
                    LOp::Pop(_, s) => pop = jsym(s),
                    LOp::Push(_, w) => push = w.iter().map(|s| jsym(s)).collect(),
                    LOp::Write(_, s) => write = Some(jsym(s)),
                    LOp::Move(_, d) => {
                        mv = match d {
                            Dir::L => "L",
                            Dir::R => "R",
                            _ => "S",
                        }
                        .to_string()
                    }
                    other => return Err(format!("JFLAP has no equivalent of {other:?}")),
                }
            }
            let tag = |name: &str, v: &str| {
                if v.is_empty() {
                    format!("<{name}/>")
                } else {
                    format!("<{name}>{v}</{name}>")
                }
            };
            x.push_str(&format!(
                "\t\t<transition>\n\t\t\t<from>{}</from>\n\t\t\t<to>{}</to>\n\t\t\t{}\n",
                id(f),
                id(t),
                tag("read", &read)
            ));
            match jtype {
                "pda" => x.push_str(&format!(
                    "\t\t\t{}\n\t\t\t{}\n",
                    tag("pop", &pop),
                    tag("push", &push)
                )),
                "turing" => x.push_str(&format!(
                    "\t\t\t{}\n\t\t\t<move>{mv}</move>\n",
                    tag("write", &write.unwrap_or_else(|| read.clone()))
                )),
                _ => {}
            }
            x.push_str("\t\t</transition>\n");
        }
    }
    x.push_str("\t</automaton>\n</structure>\n");
    Ok(x)
}

fn grammar_from_view(view: &Json) -> Result<String, String> {
    let mut x = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?><!--Created with Delta.-->\n<structure>\n\t<type>grammar</type>\n",
    );
    let vars: Vec<&str> = view["variables"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    // JFLAP tells variables from terminals by case: one upper-case letter is a variable.
    let fits = |s: &str| {
        let upper = s.chars().count() == 1 && s.chars().all(|c| c.is_ascii_uppercase());
        upper == vars.contains(&s) && s.chars().count() == 1
    };
    for r in view["rules"].as_array().into_iter().flatten() {
        let left = r[0].as_str().unwrap_or("");
        let raw: Vec<&str> = r[1]
            .as_array()
            .map(|a| a.iter().filter_map(|s| s.as_str()).collect())
            .unwrap_or_default();
        if let Some(bad) = std::iter::once(left)
            .chain(raw.iter().copied())
            .find(|s| !fits(s))
        {
            return Err(format!(
                "JFLAP grammars write variables as single upper-case letters and terminals as single other characters; `{bad}` doesn't fit"
            ));
        }
        let right: Vec<String> = raw.iter().map(|s| jsym(s)).collect();
        x.push_str(&format!(
            "\t<production>\n\t\t<left>{}</left>\n\t\t<right>{}</right>\n\t</production>\n",
            esc(left),
            right.join("")
        ));
    }
    x.push_str("</structure>\n");
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A program with `m.jff` available, run as `deltac` would.
    fn run_with(file: &str, src: &str) -> Json {
        let mut opts = crate::RunOptions::default();
        opts.files.insert("m.jff".to_string(), file.to_string());
        crate::run_with(src, &opts)
    }

    fn passes(out: &Json) -> bool {
        out["ok"] == true
            && out["outputs"]
                .as_array()
                .is_some_and(|a| a.iter().all(|o| o["kind"] != "test" || o["pass"] == true))
    }

    // The way JFLAP 7.1 writes a finite automaton: comments, positions, an empty <read/> for λ,
    // and a read of two characters.
    const FA: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><!--Created with JFLAP 7.1.--><structure>
	<type>fa</type>
	<automaton>
		<!--The list of states.-->
		<state id="0" name="q0"><x>83.0</x><y>139.0</y><initial/></state>
		<state id="1" name="q 1"><x>200.0</x><y>139.0</y></state>
		<state id="2" name="q2"><x>300.0</x><y>139.0</y><final/></state>
		<!--The list of transitions.-->
		<transition><from>0</from><to>1</to><read>ab</read></transition>
		<transition><from>1</from><to>2</to><read/></transition>
		<transition><from>2</from><to>2</to><read>a</read></transition>
	</automaton>
</structure>"#;

    #[test]
    fn a_jflap_automaton_imports_with_its_names_and_long_reads() {
        let src = to_delta(FA, "M").unwrap_or_default();
        // "q 1" isn't a Delta name, so it becomes q1; "ab" becomes two arrows; λ is an ε-move.
        assert!(src.starts_with("nfa M {"), "{src}");
        assert!(
            src.contains("q0 -a-> q0_1")
                && src.contains("q0_1 -b-> q1")
                && src.contains("q1 -ε-> q2"),
            "{src}"
        );
        let out = run_with(
            FA,
            "import \"m.jff\"\ncheck m accepts \"ab\", \"aba\"\ncheck m rejects \"a\", \"b\"",
        );
        assert!(passes(&out), "{out}");
    }

    #[test]
    fn a_jflap_pda_starts_with_z_on_the_stack() {
        let pda = r#"<structure><type>pda</type><automaton>
            <state id="0" name="q0"><initial/></state><state id="1" name="q1"><final/></state>
            <transition><from>0</from><to>0</to><read>a</read><pop>Z</pop><push>AZ</push></transition>
            <transition><from>0</from><to>0</to><read>a</read><pop>A</pop><push>AA</push></transition>
            <transition><from>0</from><to>1</to><read>b</read><pop>A</pop><push/></transition>
            <transition><from>1</from><to>1</to><read>b</read><pop>A</pop><push/></transition>
        </automaton></structure>"#;
        let out = run_with(
            pda,
            "import \"m.jff\" as P\ncheck P accepts \"ab\", \"aabb\", \"aab\"\ncheck P rejects \"\", \"b\", \"abb\"",
        );
        assert!(passes(&out), "{out}");
    }

    #[test]
    fn a_jflap_turing_machine_reads_empty_cells_as_the_blank() {
        let tm = r#"<structure><type>turing</type><automaton>
            <state id="0" name="q0"><initial/></state><state id="1" name="q1"><final/></state>
            <transition><from>0</from><to>0</to><read>0</read><write>1</write><move>R</move></transition>
            <transition><from>0</from><to>1</to><read/><write/><move>L</move></transition>
        </automaton></structure>"#;
        let out = run_with(
            tm,
            "import \"m.jff\" as T\nassert T.run(\"00\").verdict == :accept\nassert T.run(\"01\").verdict == :reject",
        );
        assert!(passes(&out), "{out}");
    }

    #[test]
    fn what_jflap_cannot_hold_is_refused_with_a_reason() {
        assert!(
            to_delta("<structure><type>mealy</type></structure>", "M")
                .is_err_and(|e| e.contains("mealy"))
        );
        assert!(to_delta("not xml", "M").is_err_and(|e| e.contains("not a JFLAP file")));
        let out = run_with(
            "<structure><type>moore</type></structure>",
            "import \"m.jff\"",
        );
        assert!(out["diagnostics"][0]["code"] == "E519", "{out}");
        let out = crate::run("import \"m.delta\"");
        assert!(out["diagnostics"][0]["code"] == "E517", "{out}");
        let out = crate::run("import \"gone.jff\"");
        assert!(out["diagnostics"][0]["code"] == "E518", "{out}");
    }

    #[test]
    fn saved_jff_files_import_back_to_the_same_machine() {
        let src = "nfa N {\n start p; accept r\n p -a,b-> p\n p -a-> q\n q -b-> r\n}\nsave N to \"n.jff\"";
        let out = crate::run(src);
        let view = &out["outputs"][0]["value"];
        let xml = from_view(view).unwrap_or_default();
        assert!(
            xml.contains("<type>fa</type>") && xml.matches("<transition>").count() == 4,
            "{xml}"
        );
        let back = run_with(
            &xml,
            &format!(
                "{}\nimport \"m.jff\" as M\ncheck M ≡ N",
                src.lines().take(6).collect::<Vec<_>>().join("\n")
            ),
        );
        assert!(passes(&back), "{back}");
        let g = crate::run("cfg G {\n S -> a S b | ε\n}\nsave G to \"g.jff\"");
        let gx = from_view(&g["outputs"][0]["value"]).unwrap_or_default();
        assert!(
            gx.contains("<left>S</left>") && gx.contains("<right>aSb</right>"),
            "{gx}"
        );
        let bad = crate::run("cfg H {\n <Expr> -> a\n}\nsave H to \"h.jff\"");
        assert!(from_view(&bad["outputs"][0]["value"]).is_err());
    }
}
