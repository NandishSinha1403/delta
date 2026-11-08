//! The standard prelude (`prelude.delta`): ordinary Delta source, parsed into the models every
//! program starts with.

use crate::ast::StmtKind;
use crate::model::{self, Model};
use std::collections::HashMap;
use std::rc::Rc;

pub const SOURCE: &str = include_str!("prelude.delta");

pub fn models() -> HashMap<String, Rc<Model>> {
    let (prog, _) = crate::parser::parse(SOURCE, &[]);
    let mut out: HashMap<String, Rc<Model>> = HashMap::new();
    for s in &prog.stmts {
        if let StmtKind::Model(decl) = &s.kind
            && let Ok(m) = model::from_decl(decl, &out)
        {
            out.insert(decl.name.clone(), Rc::new(m));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_matches_the_spec() {
        // Every `model` line of 01-LANGUAGE.md §4's prelude block appears here verbatim.
        let spec = include_str!("../../../docs/v2/01-LANGUAGE.md");
        let block = spec.split("### The standard prelude").nth(1).unwrap_or("");
        let lines: Vec<&str> = block.lines().filter(|l| l.starts_with("model ")).collect();
        assert_eq!(lines.len(), 14);
        for l in lines {
            assert!(SOURCE.contains(l), "missing from prelude.delta: {l}");
        }
        assert_eq!(models().len(), 14);
    }
}
