//! Quotations from Sipser (3rd ed.) that diagnostics cite, shown on hover.

use crate::diag::BookRef;

pub const ALPHABET: BookRef = BookRef {
    item: "§0.2",
    quote: "We define an alphabet to be any nonempty finite set. The members of the alphabet are the symbols of the alphabet.",
};
pub const EMPTY_STRING: BookRef = BookRef {
    item: "§0.2",
    quote: "The string of length zero is called the empty string and is written ε.",
};
pub const SETS: BookRef = BookRef {
    item: "§0.2",
    quote: "The order of describing a set doesn't matter, nor does repetition of its members.",
};
pub const DEF_1_5_Q: BookRef = BookRef {
    item: "Definition 1.5",
    quote: "Q is a finite set called the states.",
};
pub const DEF_1_5_DELTA: BookRef = BookRef {
    item: "Definition 1.5",
    quote: "δ : Q × Σ → Q is the transition function.",
};
pub const DEF_1_5_TOTAL: BookRef = BookRef {
    item: "Definition 1.5 (p. 36)",
    quote: "A finite automaton has exactly one transition arrow exiting every state for each possible input symbol.",
};
pub const DEF_1_5_START: BookRef = BookRef {
    item: "Definition 1.5",
    quote: "q₀ ∈ Q is the start state.",
};
pub const DEF_1_5_ACCEPT: BookRef = BookRef {
    item: "Definition 1.5",
    quote: "F ⊆ Q is the set of accept states.",
};
pub const EMPTY_F: BookRef = BookRef {
    item: "p. 36",
    quote: "F can be the empty set ∅, which means that the machine has no accept states and accepts no strings.",
};
pub const DEF_1_37_DELTA: BookRef = BookRef {
    item: "Definition 1.37",
    quote: "δ : Q × Σ_ε → P(Q) is the transition function.",
};
pub const DFA_IS_NFA: BookRef = BookRef {
    item: "p. 47",
    quote: "Every deterministic finite automaton is automatically a nondeterministic finite automaton.",
};
pub const DEF_1_52: BookRef = BookRef {
    item: "Definition 1.52",
    quote: "R is a regular expression if R is a for some a in the alphabet Σ, ε, ∅, (R₁ ∪ R₂), (R₁ ∘ R₂), or (R₁*).",
};
pub const DEF_1_52_PLUS: BookRef = BookRef {
    item: "p. 65",
    quote: "R⁺ is shorthand for RR*. In other words, whereas R* has all strings that are 0 or more concatenations of strings from R, the language R⁺ has all strings that are 1 or more concatenations of strings from R.",
};
pub const THM_1_39: BookRef = BookRef {
    item: "Theorem 1.39",
    quote: "Every nondeterministic finite automaton has an equivalent deterministic finite automaton.",
};
pub const LEMMA_1_60: BookRef = BookRef {
    item: "Lemma 1.60",
    quote: "If a language is regular, then it is described by a regular expression.",
};
pub const STRINGS: BookRef = BookRef {
    item: "§0.2",
    quote: "A string over an alphabet is a finite sequence of symbols from that alphabet.",
};
