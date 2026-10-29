//! Running machines on input strings (docs/plan/02-CONVERSIONS.md §C, S1–S2).

pub mod s1_dfa;
pub mod s2_nfa;

use serde::Serialize;
use ts_rs::TS;

/// The configuration after reading `pos` symbols: the active states (one for a DFA, the
/// "fingers" for an NFA).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct RunStepView {
    pub pos: u32,
    pub active: Vec<u32>,
    pub message: String,
}

/// A whole run, ready to animate.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct RunView {
    /// The machine's name in the program.
    pub object: String,
    /// "dfa" or "nfa".
    pub kind: String,
    /// The input, one entry per symbol.
    pub input: Vec<String>,
    /// steps[0] is the start configuration; steps[i] is after reading i symbols.
    pub steps: Vec<RunStepView>,
    pub accepted: bool,
}
