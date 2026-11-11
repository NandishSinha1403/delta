//! Context-free grammars (01-LANGUAGE.md §11): parsing the `cfg { … }` body, membership by the
//! Earley algorithm (`parse/earley`), emptiness (`cfg/generating`) and finiteness
//! (`cfg/finiteness`: a cycle that can grow the word).

use crate::value::{Object, Sym, TAG_GRAMMAR, sym_text};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GSym {
    V(usize),
    T(Sym),
}

#[derive(Debug)]
pub struct Grammar {
    pub name: Option<String>,
    pub vars: Vec<String>,
    pub start: usize,
    pub rules: Vec<(usize, Vec<GSym>)>,
    pub terminals: BTreeSet<Sym>,
    /// The construction that made this grammar (`cnf`, `to_cfg`, …), if any.
    pub provenance: Option<crate::machine::Provenance>,
}

impl Object for Grammar {
    fn tag(&self) -> u8 {
        TAG_GRAMMAR
    }
    fn key(&self) -> String {
        self.text()
    }
    fn type_name(&self) -> &'static str {
        "grammar"
    }
    fn display(&self) -> String {
        format!(
            "cfg {}{{ {} }}",
            self.name
                .as_ref()
                .map(|n| format!("{n} "))
                .unwrap_or_default(),
            self.text().replace('\n', "; ")
        )
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

impl Grammar {
    pub fn var_text(&self, v: usize) -> String {
        let n = self.vars.get(v).cloned().unwrap_or_default();
        if n.chars().count() > 1 {
            format!("<{n}>")
        } else {
            n
        }
    }

    pub fn text(&self) -> String {
        let mut lines = Vec::new();
        for (v, _) in self.vars.iter().enumerate() {
            let alts: Vec<String> = self
                .rules
                .iter()
                .filter(|(l, _)| *l == v)
                .map(|(_, rhs)| {
                    if rhs.is_empty() {
                        crate::notation::eps()
                    } else {
                        rhs.iter()
                            .map(|s| match s {
                                GSym::V(x) => self.var_text(*x),
                                GSym::T(t) => sym_text(t),
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    }
                })
                .collect();
            if !alts.is_empty() {
                lines.push(format!("{} -> {}", self.var_text(v), alts.join(" | ")));
            }
        }
        lines.join("\n")
    }

    // manual:begin grammar-nullable
    /// The nullable variables.
    pub fn nullable(&self) -> Vec<bool> {
        let mut null = vec![false; self.vars.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for (l, rhs) in &self.rules {
                if !null[*l] && rhs.iter().all(|s| matches!(s, GSym::V(x) if null[*x])) {
                    null[*l] = true;
                    changed = true;
                }
            }
        }
        null
    }
    // manual:end grammar-nullable

    /// `cfg/generating`: variables that derive some terminal word.
    pub fn generating(&self) -> Vec<bool> {
        let mut generates = vec![false; self.vars.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for (l, rhs) in &self.rules {
                if !generates[*l]
                    && rhs.iter().all(|s| match s {
                        GSym::T(_) => true,
                        GSym::V(x) => generates[*x],
                    })
                {
                    generates[*l] = true;
                    changed = true;
                }
            }
        }
        generates
    }

    pub fn is_empty(&self) -> bool {
        !self.generating().get(self.start).copied().unwrap_or(false)
    }

    /// Useful variables: generating and reachable from the start through generating rules.
    /// A grammar from its parts; the terminals are read off the rules.
    pub fn build(
        name: Option<String>,
        vars: Vec<String>,
        start: usize,
        rules: Vec<(usize, Vec<GSym>)>,
        provenance: Option<crate::machine::Provenance>,
    ) -> Grammar {
        let terminals = rules
            .iter()
            .flat_map(|(_, r)| r.iter())
            .filter_map(|s| match s {
                GSym::T(t) => Some(t.clone()),
                GSym::V(_) => None,
            })
            .collect();
        // Duplicate rules (from constructions) are dropped, keeping the first.
        let mut seen = HashSet::new();
        let rules = rules
            .into_iter()
            .filter(|r| seen.insert(r.clone()))
            .collect();
        Grammar {
            name,
            vars,
            start,
            rules,
            terminals,
            provenance,
        }
    }

    /// Whether every rule is A → BC (B, C not the start), A → a, or start → ε.
    pub fn is_cnf(&self) -> bool {
        self.rules.iter().all(|(l, r)| match r.as_slice() {
            [GSym::V(a), GSym::V(b)] => *a != self.start && *b != self.start,
            [GSym::T(_)] => true,
            [] => *l == self.start,
            _ => false,
        })
    }

    /// A variable name not used yet: `base`, else `base1`, `base2`, … (01 §12).
    pub fn fresh(vars: &[String], base: &str) -> String {
        if !vars.iter().any(|v| v == base) {
            return base.to_string();
        }
        (1..)
            .map(|i| format!("{base}{i}"))
            .find(|n| !vars.iter().any(|v| v == n))
            .unwrap_or_else(|| base.to_string())
    }

    /// `cfg/useless`: variables that are not generating, or not reachable from the start
    /// through generating rules (SEMANTICS §11.1).
    pub fn useless(&self) -> Vec<usize> {
        let useful = self.useful();
        (0..self.vars.len())
            .filter(|v| !useful.get(*v).copied().unwrap_or(false))
            .collect()
    }

    /// The grammar without its useless variables and the rules that mention them.
    pub fn trim(&self) -> Grammar {
        let useful = self.useful();
        let mut new_ix: Vec<Option<usize>> = vec![None; self.vars.len()];
        let mut vars = Vec::new();
        for (v, keep) in useful.iter().enumerate() {
            if *keep {
                new_ix[v] = Some(vars.len());
                vars.push(self.vars[v].clone());
            }
        }
        let Some(start) = new_ix.get(self.start).copied().flatten() else {
            // The language is empty: keep just the start variable.
            return Grammar::build(
                self.name.clone(),
                vec![
                    self.vars
                        .get(self.start)
                        .cloned()
                        .unwrap_or_else(|| "S".into()),
                ],
                0,
                Vec::new(),
                self.provenance.clone(),
            );
        };
        let rules = self
            .rules
            .iter()
            .filter_map(|(l, rhs)| {
                let l = new_ix.get(*l).copied().flatten()?;
                let rhs: Option<Vec<GSym>> = rhs
                    .iter()
                    .map(|s| match s {
                        GSym::V(x) => new_ix.get(*x).copied().flatten().map(GSym::V),
                        GSym::T(t) => Some(GSym::T(t.clone())),
                    })
                    .collect();
                Some((l, rhs?))
            })
            .collect();
        Grammar::build(
            self.name.clone(),
            vars,
            start,
            rules,
            self.provenance.clone(),
        )
    }

    fn useful(&self) -> Vec<bool> {
        let generates = self.generating();
        let mut reach = vec![false; self.vars.len()];
        if !generates.get(self.start).copied().unwrap_or(false) {
            return reach;
        }
        reach[self.start] = true;
        let mut stack = vec![self.start];
        while let Some(v) = stack.pop() {
            for (l, rhs) in &self.rules {
                let ok = *l == v
                    && rhs
                        .iter()
                        .all(|s| !matches!(s, GSym::V(x) if !generates[*x]));
                if !ok {
                    continue;
                }
                for s in rhs {
                    if let GSym::V(x) = s
                        && !reach[*x]
                    {
                        reach[*x] = true;
                        stack.push(*x);
                    }
                }
            }
        }
        reach
    }

    /// `cfg/finiteness`: infinite iff some useful variable derives itself with something that
    /// can be a nonempty word beside it.
    // Index loops over the variables read best here.
    #[allow(clippy::needless_range_loop)]
    pub fn is_finite(&self) -> bool {
        let useful = self.useful();
        let generates = self.generating();
        let null = self.nullable();
        // Variables that derive a nonempty word.
        let mut nonempty = vec![false; self.vars.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for (l, rhs) in &self.rules {
                if nonempty[*l]
                    || !rhs
                        .iter()
                        .all(|s| !matches!(s, GSym::V(x) if !generates[*x]))
                {
                    continue;
                }
                if rhs.iter().any(|s| match s {
                    GSym::T(_) => true,
                    GSym::V(x) => nonempty[*x],
                }) {
                    nonempty[*l] = true;
                    changed = true;
                }
            }
        }
        let _ = null;
        // Edges A → B with weight 1 when the rest of the rule can be nonempty.
        let n = self.vars.len();
        let mut edges: Vec<Vec<(usize, bool)>> = vec![Vec::new(); n];
        for (l, rhs) in &self.rules {
            if !useful[*l]
                || rhs
                    .iter()
                    .any(|s| matches!(s, GSym::V(x) if !generates[*x]))
            {
                continue;
            }
            for (i, s) in rhs.iter().enumerate() {
                if let GSym::V(b) = s {
                    if !useful[*b] {
                        continue;
                    }
                    let grows = rhs.iter().enumerate().any(|(j, t)| {
                        j != i
                            && match t {
                                GSym::T(_) => true,
                                GSym::V(x) => nonempty[*x],
                            }
                    });
                    edges[*l].push((*b, grows));
                }
            }
        }
        // A growing edge inside a strongly connected component means an infinite language.
        let reach = |from: usize| -> Vec<bool> {
            let mut seen = vec![false; n];
            let mut st = vec![from];
            while let Some(v) = st.pop() {
                for (b, _) in &edges[v] {
                    if !seen[*b] {
                        seen[*b] = true;
                        st.push(*b);
                    }
                }
            }
            seen
        };
        for a in 0..n {
            for (b, grows) in &edges[a] {
                if *grows && (a == *b || reach(*b)[a]) {
                    return false;
                }
            }
        }
        true
    }

    /// `parse/earley`: is `w` derivable? `fuel` bounds the number of chart items.
    pub fn accepts(&self, w: &[Sym], fuel: u64) -> Option<bool> {
        if w.iter().any(|s| !self.terminals.contains(s)) {
            return Some(false);
        }
        let chart = self.earley_chart(w, fuel)?;
        let n = w.len();
        Some(chart.get(n).is_some_and(|items| {
            items.iter().any(|(r, dot, origin)| {
                *origin == 0
                    && self
                        .rules
                        .get(*r)
                        .is_some_and(|(l, rhs)| *l == self.start && *dot == rhs.len())
            })
        }))
    }

    /// The Earley chart: for each position, the items (rule, dot, origin). None if `fuel` runs out.
    pub fn earley_chart(&self, w: &[Sym], fuel: u64) -> Option<Vec<Vec<(usize, usize, usize)>>> {
        let null = self.nullable();
        // An item is (rule, dot, origin).
        let n = w.len();
        let mut chart: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); n + 1];
        let mut seen: Vec<HashSet<(usize, usize, usize)>> = vec![HashSet::new(); n + 1];
        let start_rules: Vec<usize> = self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, (l, _))| *l == self.start)
            .map(|(i, _)| i)
            .collect();
        for r in start_rules {
            if seen[0].insert((r, 0, 0)) {
                chart[0].push((r, 0, 0));
            }
        }
        let mut used = 0u64;
        for i in 0..=n {
            let mut k = 0;
            while k < chart[i].len() {
                used += 1;
                if used > fuel {
                    return None;
                }
                let (r, dot, origin) = chart[i][k];
                k += 1;
                let Some((lhs, rhs)) = self.rules.get(r) else {
                    continue;
                };
                match rhs.get(dot) {
                    Some(GSym::V(b)) => {
                        // Predict, and skip over a nullable variable.
                        for (r2, (l2, _)) in self.rules.iter().enumerate() {
                            if l2 == b && seen[i].insert((r2, 0, i)) {
                                chart[i].push((r2, 0, i));
                            }
                        }
                        if null[*b] && seen[i].insert((r, dot + 1, origin)) {
                            chart[i].push((r, dot + 1, origin));
                        }
                    }
                    Some(GSym::T(t)) => {
                        if i < n && w.get(i) == Some(t) && seen[i + 1].insert((r, dot + 1, origin))
                        {
                            chart[i + 1].push((r, dot + 1, origin));
                        }
                    }
                    None => {
                        // Complete.
                        let waiting: Vec<(usize, usize, usize)> = chart[origin].clone();
                        for (r2, d2, o2) in waiting {
                            if let Some((_, rhs2)) = self.rules.get(r2)
                                && rhs2.get(d2) == Some(&GSym::V(*lhs))
                                && seen[i].insert((r2, d2 + 1, o2))
                            {
                                chart[i].push((r2, d2 + 1, o2));
                            }
                        }
                    }
                }
            }
        }
        Some(chart)
    }
}

/// Parses a grammar body. Variables are the left-hand sides (and `<Name>`s); a token that is not
/// a variable is a terminal; a token of several characters that isn't a variable is read one
/// character at a time.
pub fn parse(name: Option<&str>, body: &str) -> Result<Grammar, (usize, String)> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0;
    for raw in body.split_inclusive(['\n', ';']) {
        let content = raw.split("//").next().unwrap_or("");
        let trimmed = content.trim().trim_end_matches(';').trim();
        if !trimmed.is_empty() {
            lines.push((offset + raw.find(trimmed).unwrap_or(0), trimmed));
        }
        offset += raw.len();
    }
    let strip = |s: &str| -> String {
        s.strip_prefix('<')
            .and_then(|x| x.strip_suffix('>'))
            .unwrap_or(s)
            .to_string()
    };
    let mut vars: Vec<String> = Vec::new();
    let mut start_name: Option<String> = None;
    let mut raw_rules: Vec<(usize, String, &str)> = Vec::new();
    for (off, line) in &lines {
        if let Some(rest) = line.strip_prefix("start ") {
            start_name = Some(strip(rest.trim()));
            continue;
        }
        let Some((lhs, rhs)) = line.split_once("->").or_else(|| line.split_once('→')) else {
            return Err((
                *off,
                format!("expected a rule like `S -> a S b`, found `{line}`"),
            ));
        };
        let lhs = strip(lhs.trim());
        if lhs.is_empty() || lhs.contains(char::is_whitespace) {
            return Err((*off, "a rule's left-hand side is one variable".to_string()));
        }
        if !vars.contains(&lhs) {
            vars.push(lhs.clone());
        }
        raw_rules.push((*off, lhs, rhs));
    }
    if vars.is_empty() {
        return Err((0, "the grammar has no rules".to_string()));
    }
    let var_ix: HashMap<String, usize> = vars
        .iter()
        .enumerate()
        .map(|(i, v)| (v.clone(), i))
        .collect();
    let mut rules = Vec::new();
    let mut terminals = BTreeSet::new();
    for (_, lhs, rhs) in raw_rules {
        let l = var_ix.get(&lhs).copied().unwrap_or(0);
        for alt in split_alts(rhs) {
            let mut syms = Vec::new();
            for tok in alt.split_whitespace() {
                if tok == "ε" || tok == "λ" || tok == "eps" {
                    continue;
                }
                if let Some(q) = tok.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')) {
                    let s = Sym::from(q);
                    terminals.insert(s.clone());
                    syms.push(GSym::T(s));
                    continue;
                }
                let name = strip(tok);
                if let Some(&v) = var_ix.get(&name) {
                    syms.push(GSym::V(v));
                } else if tok.starts_with('<') || tok.chars().count() == 1 {
                    let s = Sym::from(name.as_str());
                    terminals.insert(s.clone());
                    syms.push(GSym::T(s));
                } else {
                    for c in tok.chars() {
                        let cs = c.to_string();
                        if let Some(&v) = var_ix.get(&cs) {
                            syms.push(GSym::V(v));
                        } else {
                            let s = Sym::from(cs.as_str());
                            terminals.insert(s.clone());
                            syms.push(GSym::T(s));
                        }
                    }
                }
            }
            rules.push((l, syms));
        }
    }
    let start = match &start_name {
        Some(s) => *var_ix
            .get(s)
            .ok_or((0, format!("the start variable `{s}` has no rules")))?,
        None => 0,
    };
    Ok(Grammar {
        name: name.map(str::to_string),
        vars,
        start,
        rules,
        terminals,
        provenance: None,
    })
}

/// Splits a right-hand side on `|` (not inside quotes).
fn split_alts(rhs: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in rhs.chars() {
        match c {
            '\'' => {
                quoted = !quoted;
                cur.push(c);
            }
            '|' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<Sym> {
        s.chars()
            .map(|c| Sym::from(c.to_string().as_str()))
            .collect()
    }

    #[test]
    fn earley_membership_and_questions() {
        let g = parse(None, "S -> 0 S 1 | ε")
            .unwrap_or_else(|_| parse(None, "S -> a").unwrap_or_else(|_| unreachable_grammar()));
        assert_eq!(g.accepts(&w("0011"), 10_000), Some(true));
        assert_eq!(g.accepts(&w("0101"), 10_000), Some(false));
        assert_eq!(g.accepts(&w(""), 10_000), Some(true));
        assert!(!g.is_empty());
        assert!(!g.is_finite());
        let f = parse(None, "S -> a b | c").unwrap_or_else(|_| unreachable_grammar());
        assert!(f.is_finite());
        let e = parse(None, "S -> a S").unwrap_or_else(|_| unreachable_grammar());
        assert!(e.is_empty());
        let amb = parse(None, "S -> a S b | S S | ε").unwrap_or_else(|_| unreachable_grammar());
        assert_eq!(amb.accepts(&w("abab"), 100_000), Some(true));
    }

    fn unreachable_grammar() -> Grammar {
        Grammar {
            name: None,
            vars: vec!["S".into()],
            start: 0,
            rules: vec![],
            terminals: BTreeSet::new(),
            provenance: None,
        }
    }
}
