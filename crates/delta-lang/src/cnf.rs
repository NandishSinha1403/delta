//! `cfg/cnf`: Chomsky normal form (SEMANTICS §11.2). The phases, in order: START (a new start
//! variable), DEL (remove ε-rules), UNIT (remove unit rules), TERM (terminals in long rules
//! become variables), BIN (split right sides longer than two), then trim. Each phase records a
//! `GrammarEdit` step with the rules it changed.

use crate::grammar::{GSym, Grammar};
use crate::machine::Provenance;
use delta_core::fa::Step;
// `BTreeSet` keeps elements sorted, so the order of new rules is deterministic.
use std::collections::BTreeSet;

// `type` gives an existing type a shorter name; a tuple `(a, b)` holds values side by side.
type Rule = (usize, Vec<GSym>);

/// Rules as text, for the steps.
fn show(vars: &[String], rules: &[Rule]) -> String {
    let var = |v: usize| {
        let n = vars.get(v).cloned().unwrap_or_default();
        if n.chars().count() > 1 {
            format!("<{n}>")
        } else {
            n
        }
    };
    rules
        .iter()
        .map(|(l, r)| {
            let rhs = if r.is_empty() {
                "ε".to_string()
            } else {
                r.iter()
                    .map(|s| match s {
                        GSym::V(x) => var(*x),
                        GSym::T(t) => crate::value::sym_text(t),
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            format!("{} → {rhs}", var(*l))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Converts `g` to Chomsky normal form. `fuel` bounds the number of rules created.
pub fn cnf(g: &Grammar, fuel: usize) -> Result<Grammar, usize> {
    let mut vars = g.vars.clone();
    let mut rules: Vec<Rule> = g.rules.clone();
    let mut steps = Vec::new();
    let edit = |steps: &mut Vec<Step>, text: String| steps.push(Step::new("GrammarEdit", text));

    // START: S0 → S.
    let s0 = vars.len();
    vars.push(Grammar::fresh(&vars, "S0"));
    rules.insert(0, (s0, vec![GSym::V(g.start)]));
    edit(
        &mut steps,
        format!("START: add {}", show(&vars, &rules[..1])),
    );

    // manual:begin cnf-del
    // DEL: remove ε-rules, adding the rules with nullable occurrences deleted.
    let null = Grammar::build(None, vars.clone(), s0, rules.clone(), None).nullable();
    let mut out: Vec<Rule> = Vec::new();
    for (l, rhs) in &rules {
        let spots: Vec<usize> = (0..rhs.len())
            .filter(|i| matches!(rhs[*i], GSym::V(x) if null[x]))
            .collect();
        if spots.len() > 20 {
            return Err(fuel);
        }
        for mask in 0..(1u64 << spots.len()) {
            let kept: Vec<GSym> = rhs
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    spots
                        .iter()
                        .position(|p| p == i)
                        .is_none_or(|k| mask & (1 << k) == 0)
                })
                .map(|(_, s)| s.clone())
                .collect();
            if kept.is_empty() && *l != s0 {
                continue;
            }
            if !out.contains(&(*l, kept.clone())) {
                out.push((*l, kept));
                if out.len() > fuel {
                    return Err(fuel);
                }
            }
        }
    }
    let removed: Vec<Rule> = rules
        .iter()
        .filter(|(l, r)| r.is_empty() && *l != s0)
        .cloned()
        .collect();
    rules = out;
    // manual:end cnf-del
    edit(
        &mut steps,
        format!(
            "DEL: remove {}",
            if removed.is_empty() {
                "nothing".into()
            } else {
                show(&vars, &removed)
            }
        ),
    );

    // UNIT: A → B becomes A → u for every non-unit B → u reachable through unit rules.
    let is_unit = |r: &[GSym]| matches!(r, [GSym::V(_)]);
    let units: Vec<Rule> = rules.iter().filter(|(_, r)| is_unit(r)).cloned().collect();
    let mut out: Vec<Rule> = Vec::new();
    for a in 0..vars.len() {
        // Variables reachable from a by unit rules (a included).
        let mut reach = vec![a];
        let mut i = 0;
        while let Some(&x) = reach.get(i) {
            for (l, r) in &units {
                if *l == x
                    && let [GSym::V(y)] = r.as_slice()
                    && !reach.contains(y)
                {
                    reach.push(*y);
                }
            }
            i += 1;
        }
        for x in reach {
            for (l, r) in &rules {
                if *l == x && !is_unit(r) && !out.contains(&(a, r.clone())) {
                    out.push((a, r.clone()));
                    if out.len() > fuel {
                        return Err(fuel);
                    }
                }
            }
        }
    }
    // Keep the original order where possible: rules of each variable in first-seen order.
    out.sort_by_key(|(l, _)| *l);
    rules = out;
    edit(
        &mut steps,
        format!(
            "UNIT: remove {}",
            if units.is_empty() {
                "nothing".into()
            } else {
                show(&vars, &units)
            }
        ),
    );

    // TERM: a terminal in a right side of length ≥ 2 becomes U_a → a.
    let mut term_var: Vec<(crate::value::Sym, usize)> = Vec::new();
    let mut added: Vec<Rule> = Vec::new();
    let long_terms: BTreeSet<crate::value::Sym> = rules
        .iter()
        .filter(|(_, r)| r.len() >= 2)
        .flat_map(|(_, r)| r.iter())
        .filter_map(|s| match s {
            GSym::T(t) => Some(t.clone()),
            GSym::V(_) => None,
        })
        .collect();
    for t in long_terms {
        let v = vars.len();
        vars.push(Grammar::fresh(
            &vars,
            &format!("U_{}", crate::value::sym_text(&t)),
        ));
        term_var.push((t.clone(), v));
        added.push((v, vec![GSym::T(t)]));
    }
    for (_, r) in rules.iter_mut() {
        if r.len() >= 2 {
            for s in r.iter_mut() {
                if let GSym::T(t) = s
                    && let Some((_, v)) = term_var.iter().find(|(x, _)| x == t)
                {
                    *s = GSym::V(*v);
                }
            }
        }
    }
    edit(
        &mut steps,
        format!(
            "TERM: add {}",
            if added.is_empty() {
                "nothing".into()
            } else {
                show(&vars, &added)
            }
        ),
    );
    rules.extend(added);

    // BIN: A → B1 … Bk (k ≥ 3) becomes a chain through X1, X2, …
    let mut out: Vec<Rule> = Vec::new();
    let mut split: Vec<Rule> = Vec::new();
    let mut xn = 1;
    for (l, r) in rules {
        if r.len() <= 2 {
            out.push((l, r));
            continue;
        }
        let mut left = l;
        for sym in r.iter().take(r.len() - 2) {
            // X1, X2, … in creation order, skipping names the grammar already uses.
            while vars.iter().any(|v| *v == format!("X{xn}")) {
                xn += 1;
            }
            let x = vars.len();
            vars.push(format!("X{xn}"));
            xn += 1;
            let rule = (left, vec![sym.clone(), GSym::V(x)]);
            split.push(rule.clone());
            out.push(rule);
            left = x;
        }
        let last = (left, r[r.len() - 2..].to_vec());
        split.push(last.clone());
        out.push(last);
    }
    edit(
        &mut steps,
        format!(
            "BIN: {}",
            if split.is_empty() {
                "nothing to split".into()
            } else {
                show(&vars, &split)
            }
        ),
    );

    let prov = Provenance {
        algorithm: "cfg/cnf".to_string(),
        steps: steps.clone(),
    };
    let mut result = Grammar::build(g.name.clone(), vars, s0, out, Some(prov)).trim();
    if let Some(p) = result.provenance.as_mut() {
        p.steps.push(Step::new(
            "GrammarEdit",
            "trim: drop useless variables".to_string(),
        ));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse;

    #[test]
    fn every_rule_is_in_normal_form() {
        let g = parse(None, "S -> A S A | a B\nA -> B | S\nB -> b | ε")
            .unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
        let c = cnf(&g, 100_000).unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
        assert!(!c.rules.is_empty());
        for (l, r) in &c.rules {
            let ok = match r.as_slice() {
                [GSym::V(a), GSym::V(b)] => *a != c.start && *b != c.start,
                [GSym::T(_)] => true,
                [] => *l == c.start,
                _ => false,
            };
            assert!(ok, "{}", c.text());
        }
        for w in ["ab", "aab", "abab", "b", "", "aba"] {
            let word: Vec<crate::value::Sym> = w
                .chars()
                .map(|ch| crate::value::Sym::from(ch.to_string().as_str()))
                .collect();
            assert_eq!(
                g.accepts(&word, 1_000_000),
                c.accepts(&word, 1_000_000),
                "{w}"
            );
        }
    }
}
