//! A Turing machine's tape for the simulator (`sim/tape`): the cells left of the head and the
//! head's cell with everything to its right, plus a rolling hash of each side.
//!
//! The simulator remembers every configuration it has seen, to find loops. Hashing a tape cell
//! by cell costs its whole length at every step, which made long runs quadratic. Instead each
//! side keeps a polynomial hash that a write or a move updates in a few multiplications, so
//! hashing a configuration costs the same however long its tape is. The hash is modulo the prime
//! 2⁶¹ − 1 with two bases. (Modulo 2⁶⁴ is cheaper, but Thue–Morse tapes collide there.)

use crate::value::Sym;
use std::hash::{DefaultHasher, Hash, Hasher};

/// The prime 2⁶¹ − 1.
const P: u64 = (1 << 61) - 1;
/// Two bases below P; a collision has to happen for both at once.
const BASES: [u64; 2] = [0x1F2E_3D4C_5B6A_7988 % P, 0x0B7E_1516_28AE_D2A6 % P];

/// a · b mod P. `u128` holds the full product of two `u64`s.
const fn mul(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % P as u128) as u64
}

/// bᵉ mod P, by squaring. A `const fn` can run at compile time.
const fn pow(mut b: u64, mut e: u64) -> u64 {
    let mut r = 1;
    while e > 0 {
        if e & 1 == 1 {
            r = mul(r, b);
        }
        b = mul(b, b);
        e >>= 1;
    }
    r
}

/// The inverse of each base: b^(P−2) = b⁻¹ mod P, since P is prime.
const INVERSES: [u64; 2] = [pow(BASES[0], P - 2), pow(BASES[1], P - 2)];

/// One cell's value in the hash: its symbol's text, hashed.
fn cell(x: &Sym) -> u64 {
    let mut h = DefaultHasher::new();
    x.hash(&mut h);
    h.finish() % P
}

/// The hash Σ cell(sᵢ)·Bⁱ of a sequence s₀ … sₙ₋₁ for both bases, and Bⁿ to extend it.
#[derive(Clone, Copy, Debug)]
struct Roll {
    h: [u64; 2],
    p: [u64; 2],
}

impl Roll {
    const EMPTY: Roll = Roll { h: [0, 0], p: [1, 1] };

    fn push_back(&mut self, x: &Sym) {
        let c = cell(x);
        for k in 0..2 {
            self.h[k] = (self.h[k] + mul(c, self.p[k])) % P;
            self.p[k] = mul(self.p[k], BASES[k]);
        }
    }

    fn pop_back(&mut self, x: &Sym) {
        let c = cell(x);
        for k in 0..2 {
            self.p[k] = mul(self.p[k], INVERSES[k]);
            self.h[k] = (self.h[k] + P - mul(c, self.p[k])) % P;
        }
    }

    fn push_front(&mut self, x: &Sym) {
        let c = cell(x);
        for k in 0..2 {
            self.h[k] = (mul(self.h[k], BASES[k]) + c) % P;
            self.p[k] = mul(self.p[k], BASES[k]);
        }
    }

    fn pop_front(&mut self, x: &Sym) {
        let c = cell(x);
        for k in 0..2 {
            self.h[k] = mul((self.h[k] + P - c) % P, INVERSES[k]);
            self.p[k] = mul(self.p[k], INVERSES[k]);
        }
    }

    fn replace_front(&mut self, old: &Sym, new: &Sym) {
        let (o, n) = (cell(old), cell(new));
        for k in 0..2 {
            self.h[k] = (self.h[k] + P - o + n) % P;
        }
    }
}

/// A tape. The right side is never empty (the head's cell) and has no blanks at its end beyond
/// the head's cell; the left side keeps its blanks, so a head that walks right over blanks is in
/// a new configuration at every step.
#[derive(Clone, Debug)]
pub struct Tape {
    /// Cells left of the head, nearest last.
    left: Vec<Sym>,
    /// The head's cell and everything to its right.
    right: Vec<Sym>,
    left_hash: Roll,
    right_hash: Roll,
}

// Two tapes are equal when their cells are; the hashes follow from the cells.
impl PartialEq for Tape {
    fn eq(&self, other: &Tape) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl Eq for Tape {}

// Hashing a tape reads its lengths and rolling hashes, never its cells.
impl Hash for Tape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.left.len().hash(state);
        self.right.len().hash(state);
        self.left_hash.h.hash(state);
        self.right_hash.h.hash(state);
    }
}

impl Tape {
    /// A tape holding `cells` with the head on the first one.
    pub fn new(cells: Vec<Sym>, blank: &Sym) -> Tape {
        let mut t = Tape {
            left: Vec::new(),
            right: Vec::new(),
            left_hash: Roll::EMPTY,
            right_hash: Roll::EMPTY,
        };
        for x in cells {
            t.right_hash.push_back(&x);
            t.right.push(x);
        }
        t.trim(blank);
        t
    }

    pub fn left(&self) -> &[Sym] {
        &self.left
    }

    pub fn right(&self) -> &[Sym] {
        &self.right
    }

    /// The symbol under the head.
    pub fn head(&self) -> Option<&Sym> {
        self.right.first()
    }

    /// Writes `x` under the head.
    pub fn write(&mut self, x: &Sym, blank: &Sym) {
        if let Some(cell) = self.right.first_mut() {
            self.right_hash.replace_front(cell, x);
            *cell = x.clone();
        }
        self.trim(blank);
    }

    /// Moves the head one cell left; at the left end it stays.
    pub fn move_left(&mut self, blank: &Sym) {
        if let Some(x) = self.left.pop() {
            self.left_hash.pop_back(&x);
            self.right_hash.push_front(&x);
            self.right.insert(0, x);
        }
        self.trim(blank);
    }

    /// Moves the head one cell right, onto a blank past the end.
    pub fn move_right(&mut self, blank: &Sym) {
        let x = if self.right.is_empty() {
            blank.clone()
        } else {
            let x = self.right.remove(0);
            self.right_hash.pop_front(&x);
            x
        };
        self.left_hash.push_back(&x);
        self.left.push(x);
        self.trim(blank);
    }

    /// Drops blanks at the right end beyond the head's cell; an empty right side gets a blank.
    fn trim(&mut self, blank: &Sym) {
        while self.right.len() > 1 && self.right.last() == Some(blank) {
            if let Some(x) = self.right.pop() {
                self.right_hash.pop_back(&x);
            }
        }
        if self.right.is_empty() {
            self.right_hash.push_back(blank);
            self.right.push(blank.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> Sym {
        Sym::from(x)
    }

    /// A tape built by moves and writes hashes like the same cells built directly.
    #[test]
    fn the_rolling_hash_matches_a_fresh_one() {
        let blank = s("⊔");
        let mut t = Tape::new(vec![s("a"), s("b")], &blank);
        t.move_right(&blank);
        t.move_right(&blank);
        t.write(&s("c"), &blank);
        t.move_left(&blank);
        t.write(&blank, &blank);
        t.move_right(&blank);
        t.move_right(&blank);
        // Cells: a ⊔ c ⊔ with the head on the last ⊔.
        let mut fresh = Tape::new(vec![s("⊔")], &blank);
        for x in [s("a"), s("⊔"), s("c")] {
            fresh.left_hash.push_back(&x);
            fresh.left.push(x);
        }
        assert_eq!(t, fresh);
        assert_eq!(t.left_hash.h, fresh.left_hash.h);
        assert_eq!(t.right_hash.h, fresh.right_hash.h);
        assert_eq!(t.right_hash.p, fresh.right_hash.p);
    }

    #[test]
    fn the_inverses_are_inverses() {
        assert_eq!(mul(BASES[0], INVERSES[0]), 1);
        assert_eq!(mul(BASES[1], INVERSES[1]), 1);
    }

    /// Thue–Morse words break polynomial hashes modulo 2⁶⁴; modulo a prime they don't.
    #[test]
    fn thue_morse_tapes_hash_apart() {
        let blank = s("⊔");
        let n = 1 << 12;
        let tm = |i: usize| i.count_ones() % 2 == 1;
        let a: Vec<Sym> = (0..n).map(|i| s(if tm(i) { "1" } else { "0" })).collect();
        let b: Vec<Sym> = (0..n).map(|i| s(if tm(i) { "0" } else { "1" })).collect();
        let (ta, tb) = (Tape::new(a, &blank), Tape::new(b, &blank));
        assert_ne!(ta.right_hash.h, tb.right_hash.h);
    }
}
