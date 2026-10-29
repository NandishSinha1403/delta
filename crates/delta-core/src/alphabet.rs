//! Alphabets: "a nonempty finite set" of symbols (Sipser §0.2).
//!
//! Symbols are interned: each distinct name gets a `SymId`, and the engine works with ids only.
//! Names may be multi-character (Example 1.13 uses ⟨RESET⟩ as one symbol).

use crate::ids::SymId;
// `rustc_hash::FxHashMap` is a HashMap with a faster (non-cryptographic) hash function.
use rustc_hash::FxHashMap;

/// An ordered set of symbol names. The order is the order symbols were declared in.
#[derive(Debug, Clone, Default)]
pub struct Alphabet {
    names: Vec<String>,
    index: FxHashMap<String, SymId>,
}

// `impl PartialEq for Alphabet` implements the `==` trait by hand: two alphabets are equal when
// they list the same names in the same order (the index map is derived data).
// A "trait" is a named set of methods a type can implement, like an interface in TypeScript.
impl PartialEq for Alphabet {
    fn eq(&self, other: &Self) -> bool {
        self.names == other.names
    }
}

impl Alphabet {
    /// The empty alphabet (callers add symbols with `intern`).
    pub fn new() -> Self {
        Self::default()
    }

    /// An alphabet with these symbols, in this order (duplicates are ignored).
    // `&[String]` is a "slice": a borrowed view of a run of Strings (from a Vec or an array).
    pub fn from_names(names: &[String]) -> Self {
        let mut a = Self::new();
        for n in names {
            a.intern(n);
        }
        a
    }

    /// Returns the id of `name`, adding it if it is new.
    pub fn intern(&mut self, name: &str) -> SymId {
        if let Some(&id) = self.index.get(name) {
            return id;
        }
        let id = SymId::from_index(self.names.len());
        self.names.push(name.to_string());
        self.index.insert(name.to_string(), id);
        id
    }

    /// The id of `name`, if it is in the alphabet.
    // `Option<T>` is either `Some(value)` or `None`; `.copied()` turns Option<&SymId> into Option<SymId>.
    pub fn get(&self, name: &str) -> Option<SymId> {
        self.index.get(name).copied()
    }

    /// The name of a symbol ("?" for an id from a different alphabet, which is a bug).
    pub fn name(&self, id: SymId) -> &str {
        // `.map(String::as_str)` borrows the String inside the Option as a &str.
        self.names.get(id.index()).map_or("?", String::as_str)
    }

    /// The symbol written the way the DSL accepts it: `a`, or `<RESET>` for multi-char names.
    pub fn display(&self, id: SymId) -> String {
        display_symbol(self.name(id))
    }

    /// |Σ|.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Is Σ = ∅? (Not a valid alphabet, but useful while building one.)
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// All symbol names in declaration order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// All symbol ids in declaration order.
    // `-> impl Iterator<Item = SymId>` returns "some iterator of SymIds" without naming its type.
    pub fn ids(&self) -> impl Iterator<Item = SymId> + use<> {
        (0..self.names.len()).map(SymId::from_index)
    }
}

/// How a symbol is written in Delta code and in printed regexes.
pub fn display_symbol(name: &str) -> String {
    if name.chars().count() == 1 {
        name.to_string()
    } else {
        format!("<{name}>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_is_stable() {
        let mut a = Alphabet::new();
        let zero = a.intern("0");
        let reset = a.intern("RESET");
        assert_eq!(a.intern("0"), zero);
        assert_eq!(a.len(), 2);
        assert_eq!(a.name(reset), "RESET");
        assert_eq!(a.display(reset), "<RESET>");
        assert_eq!(a.get("1"), None);
        assert_eq!(a.ids().count(), 2);
    }
}
