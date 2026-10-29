//! A set of small non-negative integers (usually `StateId`s), stored as bits.
//!
//! Used for P(Q), the power set of the states: the subset construction (Theorem 1.39) uses
//! `BitSet`s as hash-map keys, so two equal sets must always compare and hash equal.

/// A growable set of `usize` values, one bit each.
// `Default` lets us write `BitSet::default()` for the empty set; `Hash` lets it be a map key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct BitSet {
    // Invariant: no trailing zero words, so equal sets have identical `words`.
    words: Vec<u64>,
}

impl BitSet {
    /// The empty set ∅.
    // `Self` is shorthand for the type being implemented (here `BitSet`).
    pub fn new() -> Self {
        Self::default()
    }

    /// A set containing exactly the given values.
    // `impl IntoIterator<Item = usize>` accepts anything that can be looped over to get usizes
    // (a Vec, an array, a range ...).
    pub fn of(values: impl IntoIterator<Item = usize>) -> Self {
        // `mut` makes a variable changeable; Rust variables are read-only by default.
        let mut set = Self::new();
        for v in values {
            set.insert(v);
        }
        set
    }

    /// Adds `i`. Returns true if it was not already present.
    // `&mut self` borrows the set mutably: the method may change it.
    pub fn insert(&mut self, i: usize) -> bool {
        let (w, bit) = (i / 64, 1u64 << (i % 64));
        if w >= self.words.len() {
            self.words.resize(w + 1, 0);
        }
        // `if let Some(x) = opt { ... }` runs the block only when the Option holds a value.
        if let Some(word) = self.words.get_mut(w) {
            let was_absent = *word & bit == 0;
            // `*word` follows the reference to change the actual word inside the Vec.
            *word |= bit;
            return was_absent;
        }
        false
    }

    /// Removes `i` if present.
    pub fn remove(&mut self, i: usize) {
        if let Some(word) = self.words.get_mut(i / 64) {
            *word &= !(1u64 << (i % 64));
        }
        self.trim();
    }

    /// Is `i` in the set?
    // `&self` borrows the set read-only.
    pub fn contains(&self, i: usize) -> bool {
        // `.is_some_and(f)` is true when the Option holds a value and f returns true for it.
        self.words
            .get(i / 64)
            .is_some_and(|w| w & (1u64 << (i % 64)) != 0)
    }

    /// self ← self ∪ other.
    pub fn union_with(&mut self, other: &BitSet) {
        if other.words.len() > self.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        // `.iter_mut().zip(...)` walks two sequences side by side, the first one mutably.
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a |= *b;
        }
    }

    /// Is self ∩ other non-empty?
    pub fn intersects(&self, other: &BitSet) -> bool {
        self.words
            .iter()
            .zip(other.words.iter())
            .any(|(a, b)| a & b != 0)
    }

    /// Is self ⊆ other?
    pub fn is_subset(&self, other: &BitSet) -> bool {
        self.words.iter().enumerate().all(|(i, a)| {
            let b = other.words.get(i).copied().unwrap_or(0);
            a & !b == 0
        })
    }

    /// A ∩ B.
    pub fn intersection(&self, other: &BitSet) -> BitSet {
        let words = self
            .words
            .iter()
            .zip(&other.words)
            .map(|(a, b)| a & b)
            .collect();
        BitSet::trimmed(words)
    }

    /// A − B: the members of A that are not in B.
    pub fn difference(&self, other: &BitSet) -> BitSet {
        let words = self
            .words
            .iter()
            .enumerate()
            .map(|(i, a)| a & !other.words.get(i).copied().unwrap_or(0))
            .collect();
        BitSet::trimmed(words)
    }

    /// Restores the no-trailing-zero-words invariant.
    fn trimmed(mut words: Vec<u64>) -> BitSet {
        while words.last() == Some(&0) {
            words.pop();
        }
        BitSet { words }
    }

    /// Is this ∅?
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Number of elements |S|.
    pub fn len(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// The elements in increasing order.
    pub fn to_vec(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.len());
        for (wi, &word) in self.words.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                out.push(wi * 64 + bit);
                w &= w - 1;
            }
        }
        out
    }

    fn trim(&mut self) {
        while self.words.last() == Some(&0) {
            self.words.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_sets_are_equal_after_removals() {
        let mut a = BitSet::of([1, 70]);
        a.remove(70);
        assert_eq!(a, BitSet::of([1]));
        assert_eq!(a.to_vec(), vec![1]);
    }

    #[test]
    fn set_operations() {
        let mut a = BitSet::of([0, 3]);
        let b = BitSet::of([3, 200]);
        assert!(a.intersects(&b));
        a.union_with(&b);
        assert_eq!(a.to_vec(), vec![0, 3, 200]);
        assert_eq!(a.len(), 3);
        assert!(b.is_subset(&a));
        assert!(!a.is_subset(&b));
        assert!(BitSet::new().is_empty());
    }

    #[test]
    fn intersection_and_difference_keep_the_invariant() {
        let a = BitSet::of([1, 70, 130]);
        let b = BitSet::of([1, 130]);
        assert_eq!(a.intersection(&b), b);
        assert_eq!(a.difference(&b), BitSet::of([70]));
        assert_eq!(b.difference(&a), BitSet::new());
    }
}
