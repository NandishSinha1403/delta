//! From an automaton to a regex, three ways (they give equivalent results):
//! - `to_regex/state-elimination`: add a new start and accept state, then remove the other
//!   states one by one (in the given order), relabelling the edges around each with regexes.
//! - `to_regex/arden`: write one equation per state, X = Σ a·Y (+ ε if accepting), and solve
//!   them by substitution, using X = A X + B ⇒ X = A* B.
//! - `to_regex/brzozowski-mccluskey`: state elimination, always removing next the state with
//!   the fewest (incoming × outgoing) edges.

use super::regex::Re;
use super::{Fa, Step};

pub const STATE_ELIMINATION: &str = "to_regex/state-elimination";
pub const ARDEN: &str = "to_regex/arden";
pub const BRZOZOWSKI_MCCLUSKEY: &str = "to_regex/brzozowski-mccluskey";

/// A generalized automaton as a matrix of regex labels (`Empty` = no edge).
fn gnfa(fa: &Fa) -> (Vec<Vec<Re>>, usize, usize) {
    let n = fa.n();
    let s = n;
    let f = n + 1;
    let mut m = vec![vec![Re::Empty; n + 2]; n + 2];
    for (p, list) in fa.edges.iter().enumerate() {
        for (a, q) in list {
            let r = a.map(Re::Sym).unwrap_or(Re::Eps);
            let cur = std::mem::replace(&mut m[p][*q], Re::Empty);
            m[p][*q] = Re::alt(cur, r);
        }
    }
    m[s][fa.start] = Re::Eps;
    for q in 0..n {
        if fa.is_accept(q) {
            m[q][f] = Re::Eps;
        }
    }
    (m, s, f)
}

fn rip(m: &mut [Vec<Re>], r: usize, alive: &[bool]) {
    let lp = Re::star(m[r][r].clone());
    let size = m.len();
    for i in 0..size {
        if i == r || !alive[i] || m[i][r] == Re::Empty {
            continue;
        }
        for j in 0..size {
            if j == r || !alive[j] || m[r][j] == Re::Empty {
                continue;
            }
            let through = Re::cat(Re::cat(m[i][r].clone(), lp.clone()), m[r][j].clone());
            let cur = std::mem::replace(&mut m[i][j], Re::Empty);
            m[i][j] = Re::alt(cur, through);
        }
    }
    for i in 0..size {
        m[i][r] = Re::Empty;
        m[r][i] = Re::Empty;
    }
}

/// State elimination in `order` (default: 0, 1, 2, …).
pub fn state_elimination(fa: &Fa, order: Option<&[usize]>, steps: &mut Vec<Step>) -> Re {
    let (mut m, s, f) = gnfa(fa);
    let n = fa.n();
    let mut alive = vec![true; n + 2];
    let default: Vec<usize> = (0..n).collect();
    let order = order.unwrap_or(&default);
    for &r in order {
        if r >= n || !alive[r] {
            continue;
        }
        steps.push(Step::new("Rip", format!("remove state {r}")));
        rip(&mut m, r, &alive);
        alive[r] = false;
    }
    // States not in the order are removed afterwards, in index order.
    for r in 0..n {
        if alive[r] {
            steps.push(Step::new("Rip", format!("remove state {r}")));
            rip(&mut m, r, &alive);
            alive[r] = false;
        }
    }
    m[s][f].clone()
}

pub fn brzozowski_mccluskey(fa: &Fa, steps: &mut Vec<Step>) -> Re {
    let (mut m, s, f) = gnfa(fa);
    let n = fa.n();
    let mut alive = vec![true; n + 2];
    for _ in 0..n {
        let best = (0..n).filter(|r| alive[*r]).min_by_key(|&r| {
            let inc = (0..n + 2)
                .filter(|&i| i != r && alive[i] && m[i][r] != Re::Empty)
                .count();
            let out = (0..n + 2)
                .filter(|&j| j != r && alive[j] && m[r][j] != Re::Empty)
                .count();
            (inc * out, r)
        });
        let Some(r) = best else { break };
        steps.push(Step::new("Rip", format!("remove state {r}")));
        rip(&mut m, r, &alive);
        alive[r] = false;
    }
    m[s][f].clone()
}

pub fn arden(fa: &Fa, steps: &mut Vec<Step>) -> Re {
    let n = fa.n();
    // eq[i] = (coefficients for each X_j, constant term).
    let mut coef: Vec<Vec<Re>> = vec![vec![Re::Empty; n]; n];
    let mut cons: Vec<Re> = (0..n)
        .map(|q| if fa.is_accept(q) { Re::Eps } else { Re::Empty })
        .collect();
    for (p, list) in fa.edges.iter().enumerate() {
        for (a, q) in list {
            let r = a.map(Re::Sym).unwrap_or(Re::Eps);
            let cur = std::mem::replace(&mut coef[p][*q], Re::Empty);
            coef[p][*q] = Re::alt(cur, r);
        }
    }
    // Eliminate X_{n-1}, …, X_1 (keep X_start for last by processing others first).
    let order: Vec<usize> = (0..n)
        .filter(|q| *q != fa.start)
        .rev()
        .chain(std::iter::once(fa.start))
        .collect();
    for (step, &x) in order.iter().enumerate() {
        // Arden's rule on X_x = coef[x][x] X_x + rest  ⇒  X_x = coef[x][x]* rest
        let lp = Re::star(coef[x][x].clone());
        coef[x][x] = Re::Empty;
        for j in 0..n {
            coef[x][j] = Re::cat(lp.clone(), coef[x][j].clone());
        }
        cons[x] = Re::cat(lp, cons[x].clone());
        steps.push(Step::new(
            "Solve",
            format!("solve the equation of state {x}"),
        ));
        if step + 1 == order.len() {
            break;
        }
        // Substitute X_x into every other equation.
        for i in 0..n {
            if i == x || coef[i][x] == Re::Empty {
                continue;
            }
            let c = std::mem::replace(&mut coef[i][x], Re::Empty);
            for j in 0..n {
                if coef[x][j] != Re::Empty {
                    let add = Re::cat(c.clone(), coef[x][j].clone());
                    let cur = std::mem::replace(&mut coef[i][j], Re::Empty);
                    coef[i][j] = Re::alt(cur, add);
                }
            }
            let add = Re::cat(c, cons[x].clone());
            let cur = std::mem::replace(&mut cons[i], Re::Empty);
            cons[i] = Re::alt(cur, add);
        }
    }
    cons[fa.start].clone()
}

#[cfg(test)]
mod tests {
    use super::super::regex::to_nfa_textbook;
    use super::super::{decide, fa_from};
    use super::*;

    #[test]
    fn all_three_give_an_equivalent_regex() {
        // DFA for words over {0,1} ending in 01.
        let d = fa_from(
            2,
            3,
            0,
            &[2],
            &[
                (0, Some(0), 1),
                (0, Some(1), 0),
                (1, Some(0), 1),
                (1, Some(1), 2),
                (2, Some(0), 1),
                (2, Some(1), 0),
            ],
        );
        let mut st = Vec::new();
        for r in [
            state_elimination(&d, None, &mut st),
            arden(&d, &mut st),
            brzozowski_mccluskey(&d, &mut st),
        ] {
            let back = to_nfa_textbook(&r, 2);
            assert_eq!(decide::equivalence(&back, &d, 10_000), Ok(None));
        }
    }
}
