//! Typed indices. Every collection in the engine is a `Vec` indexed by one of these
//! ("Simple Rust", docs/plan/03-ARCHITECTURE.md §3.1): no references, no lifetimes.

/// A state of some machine: the index into its `states` vector.
// A tuple struct `struct X(pub u32)` wraps one value; the wrapper makes a StateId and a SymId
// different types, so the compiler stops us from mixing them up.
// `Copy` means the value is duplicated on assignment instead of moved (fine for small numbers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(pub u32);

/// A symbol of an alphabet (Σ or Γ): the index into the alphabet's name list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymId(pub u32);

// `impl Type { ... }` attaches functions ("methods") to a type.
impl StateId {
    /// The id as a `Vec` index.
    // `self` (by value, since StateId is Copy) is the value the method is called on.
    pub fn index(self) -> usize {
        // `as` converts between number types.
        self.0 as usize
    }

    /// Builds an id from a `Vec` index. Machines never get close to 2³² states.
    pub fn from_index(i: usize) -> StateId {
        StateId(i as u32)
    }
}

impl SymId {
    /// The id as a `Vec` index.
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Builds an id from a `Vec` index.
    pub fn from_index(i: usize) -> SymId {
        SymId(i as u32)
    }
}
