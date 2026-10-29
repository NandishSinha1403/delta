//! Anything a Delta program can name: a machine or a regular expression.

use crate::alphabet::Alphabet;
use crate::dfa::Dfa;
use crate::gnfa::Gnfa;
use crate::nfa::Nfa;
use crate::regex::{ReArena, ReId};

/// A regular expression together with the arena it lives in and its alphabet.
#[derive(Debug, Clone)]
pub struct RegexObj {
    pub arena: ReArena,
    pub root: ReId,
    pub sigma: Alphabet,
}

impl RegexObj {
    /// The expression printed with minimal parentheses.
    pub fn text(&self) -> String {
        self.arena.print(self.root, &self.sigma)
    }
}

/// A named value in a Delta program.
#[derive(Debug, Clone)]
pub enum Object {
    Dfa(Dfa),
    Nfa(Nfa),
    Gnfa(Gnfa),
    Regex(RegexObj),
}

impl Object {
    /// "dfa", "nfa", "gnfa" or "regex", as written in the DSL.
    pub fn kind(&self) -> &'static str {
        match self {
            Object::Dfa(_) => "dfa",
            Object::Nfa(_) => "nfa",
            Object::Gnfa(_) => "gnfa",
            Object::Regex(_) => "regex",
        }
    }

    /// The alphabet Σ of the object.
    pub fn sigma(&self) -> &Alphabet {
        match self {
            Object::Dfa(m) => &m.sigma,
            Object::Nfa(m) => &m.sigma,
            Object::Gnfa(m) => &m.sigma,
            Object::Regex(r) => &r.sigma,
        }
    }
}
