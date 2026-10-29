//! Conversions between models, one file per construction, named after its id in
//! docs/plan/02-CONVERSIONS.md. Every conversion returns `(result, Trace)`.

pub mod c01_dfa_to_nfa;
pub mod c02_subset;
pub mod c03_regex_to_nfa;
pub mod c04_to_gnfa;
pub mod c05_gnfa_to_regex;
pub mod c06_product;
pub mod c07_nfa_closure;

use crate::bitset::BitSet;
use crate::ids::StateId;

/// A set of states written like the book: {1,3}, or ∅.
pub(crate) fn set_name(names: &[&str], set: &BitSet) -> String {
    if set.is_empty() {
        return "∅".to_string();
    }
    let parts: Vec<&str> = set
        .to_vec()
        .into_iter()
        // `.get(i).copied().unwrap_or("?")` reads an element safely without panicking.
        .map(|i| names.get(i).copied().unwrap_or("?"))
        .collect();
    format!("{{{}}}", parts.join(","))
}

/// The ids in a bitset.
pub(crate) fn ids(set: &BitSet) -> Vec<StateId> {
    set.to_vec().into_iter().map(StateId::from_index).collect()
}

/// A name not already used: `base`, or `base'`, `base''` … (the book's priming style).
// `pub(crate)` makes the function visible inside this crate only.
pub(crate) fn fresh_name(base: &str, taken: &[String]) -> String {
    let mut name = base.to_string();
    while taken.contains(&name) {
        name.push('\'');
    }
    name
}
