//! `cfg/gnf`: Greibach normal form (SEMANTICS §11.8). Every rule becomes A → a B₁…Bₖ, plus
//! S0 → ε when ε is in the language. Start from the Chomsky normal form, put the variables in a
//! fixed order, make each variable's rules start with a terminal or a later variable (removing
//! immediate left recursion with fresh variables Z1, Z2, …), then substitute back from the last
//! variable to the first, and finally into the Z rules.

use crate::grammar::{GSym, Grammar};
use crate::machine::Provenance;
use delta_core::fa::Step;

pub const ID: &str = "cfg/gnf";

type Rhs = Vec<GSym>;

/// Appends `r` unless it is already there (keeps the first occurrence's order).
fn push_new(list: &mut Vec<Rhs>, r: Rhs) {
    if !list.contains(&r) {
        list.push(r);
    }
}

/// Replaces each rule of `rules[a]` that starts with a variable `v` satisfying `pick` by every
/// rule of v followed by the rest. Fails with `Err(fuel)` once more than `fuel` rules exist.
fn substitute(
    rules: &mut [Vec<Rhs>],
    a: usize,
    pick: &dyn Fn(usize) -> bool,
    fuel: usize,
    total: &mut usize,
) -> Result<bool, usize> {
    let old = rules.get(a).cloned().unwrap_or_default();
    let mut out: Vec<Rhs> = Vec::new();
    let mut changed = false;
    for rhs in old {
        match rhs.first() {
            Some(GSym::V(v)) if pick(*v) && *v != a => {
                changed = true;
                for d in rules.get(*v).cloned().unwrap_or_default() {
                    let mut r = d.clone();
                    r.extend_from_slice(rhs.get(1..).unwrap_or(&[]));
                    push_new(&mut out, r);
                    *total += 1;
                    if *total > fuel {
                        return Err(fuel);
                    }
                }
            }
            _ => push_new(&mut out, rhs),
        }
    }
    if let Some(slot) = rules.get_mut(a) {
        *slot = out;
    }
    Ok(changed)
}

/// Converts `g` to Greibach normal form. `fuel` bounds the number of rules created.
pub fn gnf(g: &Grammar, fuel: usize) -> Result<Grammar, usize> {
    let c = crate::cnf::cnf(g, fuel)?;
    let mut vars = c.vars.clone();
    let mut steps: Vec<Step> = vec![Step::new(
        "GrammarEdit",
        format!("CNF first: {} rules", c.rules.len()),
    )];
    // rules[v] = the right sides of v, in order; the start's ε-rule is kept aside.
    let mut rules: Vec<Vec<Rhs>> = vec![Vec::new(); vars.len()];
    let mut start_eps = false;
    for (l, r) in &c.rules {
        if r.is_empty() && *l == c.start {
            start_eps = true;
        } else if let Some(list) = rules.get_mut(*l) {
            push_new(list, r.clone());
        }
    }
    // The order A₁ … Aₙ: the start first, then the others by index. `rank[v]` is v's position.
    let used: Vec<usize> = (0..vars.len())
        .filter(|v| *v == c.start || rules.get(*v).is_some_and(|r| !r.is_empty()))
        .collect();
    let mut order: Vec<usize> = vec![c.start];
    order.extend(used.iter().copied().filter(|v| *v != c.start));
    let mut rank = vec![usize::MAX; vars.len()];
    for (i, v) in order.iter().enumerate() {
        rank[*v] = i;
    }
    steps.push(Step::new(
        "GrammarEdit",
        format!(
            "order: {}",
            order
                .iter()
                .map(|v| c.var_text(*v))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ));

    let mut total = rules.iter().map(Vec::len).sum::<usize>();
    let mut zs: Vec<usize> = Vec::new();
    let mut zn = 1;
    for (i, &a) in order.iter().enumerate() {
        // Rules of Aᵢ that start with an earlier Aⱼ: substitute until none is left (each pass
        // raises the leading variable's rank, so this ends).
        let earlier = |v: usize| rank.get(v).is_some_and(|r| *r < i);
        while substitute(&mut rules, a, &earlier, fuel, &mut total)? {}
        // manual:begin gnf-left-recursion
        // Immediate left recursion Aᵢ → Aᵢ α.
        let mine = rules.get(a).cloned().unwrap_or_default();
        let (alphas, betas): (Vec<Rhs>, Vec<Rhs>) = mine
            .into_iter()
            .partition(|r| r.first() == Some(&GSym::V(a)));
        if alphas.is_empty() {
            continue;
        }
        while vars.iter().any(|v| *v == format!("Z{zn}")) {
            zn += 1;
        }
        let z = vars.len();
        vars.push(format!("Z{zn}"));
        zn += 1;
        zs.push(z);
        rules.push(Vec::new());
        rank.push(usize::MAX);
        let mut a_rules = Vec::new();
        for b in &betas {
            push_new(&mut a_rules, b.clone());
            let mut bz = b.clone();
            bz.push(GSym::V(z));
            push_new(&mut a_rules, bz);
        }
        let mut z_rules = Vec::new();
        for al in &alphas {
            let rest: Rhs = al.get(1..).unwrap_or(&[]).to_vec();
            push_new(&mut z_rules, rest.clone());
            let mut rz = rest;
            rz.push(GSym::V(z));
            push_new(&mut z_rules, rz);
        }
        total += a_rules.len() + z_rules.len();
        if total > fuel {
            return Err(fuel);
        }
        rules[a] = a_rules;
        rules[z] = z_rules;
        // manual:end gnf-left-recursion
        steps.push(Step::new(
            "GrammarEdit",
            format!(
                "left recursion of {}: new variable {}",
                c.var_text(a),
                vars.get(z).cloned().unwrap_or_default()
            ),
        ));
    }

    // Back-substitution: from the last variable to the first, then the Z rules.
    let any_var = |v: usize| v < usize::MAX;
    for &a in order.iter().rev().chain(zs.iter()) {
        while substitute(&mut rules, a, &any_var, fuel, &mut total)? {}
    }
    steps.push(Step::new(
        "GrammarEdit",
        "substitute back: every rule now starts with a terminal".to_string(),
    ));

    let mut out: Vec<(usize, Rhs)> = Vec::new();
    if start_eps {
        out.push((c.start, Vec::new()));
    }
    for &a in order.iter().chain(zs.iter()) {
        for r in rules.get(a).cloned().unwrap_or_default() {
            out.push((a, r));
        }
    }
    let prov = Provenance {
        algorithm: ID.to_string(),
        steps,
    };
    let mut result = Grammar::build(g.name.clone(), vars, c.start, out, Some(prov)).trim();
    if let Some(p) = result.provenance.as_mut() {
        p.steps.push(Step::new(
            "GrammarEdit",
            "trim: drop useless variables".to_string(),
        ));
    }
    Ok(result)
}

impl Grammar {
    /// Is every rule A → a B₁…Bₖ (variables after the terminal), or S → ε with S on no right side?
    pub fn is_gnf(&self) -> bool {
        let start_on_right = self
            .rules
            .iter()
            .any(|(_, r)| r.contains(&GSym::V(self.start)));
        self.rules.iter().all(|(l, r)| match r.split_first() {
            None => *l == self.start && !start_on_right,
            Some((GSym::T(_), rest)) => rest.iter().all(|s| matches!(s, GSym::V(_))),
            Some((GSym::V(_), _)) => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse;
    use crate::value::Sym;

    fn grammar(text: &str) -> Grammar {
        parse(None, text).unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None))
    }

    fn word(w: &str) -> Vec<Sym> {
        w.chars()
            .map(|c| Sym::from(c.to_string().as_str()))
            .collect()
    }

    /// All words over {a, b} up to length 6.
    fn words() -> Vec<String> {
        let mut out = vec![String::new()];
        let mut layer = vec![String::new()];
        for _ in 0..6 {
            let next: Vec<String> = layer
                .iter()
                .flat_map(|w| ["a", "b"].map(|c| format!("{w}{c}")))
                .collect();
            out.extend(next.iter().cloned());
            layer = next;
        }
        out
    }

    #[test]
    fn same_language_and_every_rule_in_normal_form() {
        for text in [
            "S -> a S b | ε",
            "S -> S S | a S b | ε",
            "S -> A B\nA -> A a | b\nB -> B b | a",
            "E -> E a E | b",
            // Grows to 853 rules: checked on the shorter words only (Earley is slow on it).
            "S -> A S A | a B\nA -> B | S\nB -> b | ε",
        ] {
            let short = text.contains("A S A");
            let g = grammar(text);
            let n =
                gnf(&g, 100_000).unwrap_or_else(|_| Grammar::build(None, vec![], 0, vec![], None));
            assert!(n.is_gnf(), "{text}:\n{}", n.text());
            for w in words().iter().filter(|w| !short || w.len() <= 3) {
                assert_eq!(
                    g.accepts(&word(w), 1_000_000),
                    n.accepts(&word(w), 1_000_000),
                    "{text}: {w}"
                );
            }
        }
    }

    #[test]
    fn a_small_budget_stops_the_construction() {
        assert!(gnf(&grammar("S -> S S | a S b | ε"), 3).is_err());
    }
}
