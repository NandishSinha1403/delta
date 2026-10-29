//! Budgets and engine errors.
//!
//! Subset construction is exponential and Turing machines may never halt, so every algorithm runs
//! under a `Budget` and reports running out of it as a value, never as a freeze or a crash
//! (docs/plan/08-ERRORS.md §1).

/// Limits an algorithm must respect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Budget {
    /// Most states a construction may create.
    pub max_states: u32,
    /// Most trace steps recorded; after this the result is still computed, the trace is cut.
    pub max_trace: u32,
    /// Most symbols a regex may grow to (state elimination can blow up exponentially).
    pub max_regex_nodes: u32,
}

// `impl Default for Budget` provides `Budget::default()` by hand instead of deriving it.
impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_states: 10_000,
            max_trace: 2_000,
            max_regex_nodes: 200_000,
        }
    }
}

/// An error that stops one operation (as opposed to a diagnostic about the user's file).
#[derive(Debug, Clone, PartialEq)]
pub enum TlError {
    /// A precondition was not met; the message explains it to the user.
    Invalid(String),
    /// A budget ran out. `which` names the limit, e.g. "max_states".
    Budget { which: &'static str, limit: u32 },
    /// A bug in the engine (X999). Never the user's fault.
    Internal(String),
}

// `std::fmt::Display` is the trait behind `{}` in format strings; implementing it lets us print
// a TlError as a human sentence.
impl std::fmt::Display for TlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlError::Invalid(msg) => write!(f, "{msg}"),
            TlError::Budget { which, limit } => {
                write!(f, "stopped: the result needs more than {limit} ({which})")
            }
            TlError::Internal(msg) => write!(f, "internal error (a bug in Delta): {msg}"),
        }
    }
}
