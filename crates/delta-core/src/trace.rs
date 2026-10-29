//! Traces: every algorithm returns the steps it took, not just its result
//! (docs/plan/02-CONVERSIONS.md §T). The UI scrubs through them like the book's figures.
//!
//! A step says which states and arrows of the *final* machine it added. Replaying a prefix of the
//! steps therefore rebuilds the half-finished machine. Algorithms whose machine shrinks (state
//! elimination) attach a full snapshot instead.

use crate::budget::Budget;
use crate::ids::StateId;
use crate::object::Object;

/// One step of an algorithm.
#[derive(Debug, Clone)]
pub struct Step {
    /// The book rule applied, e.g. "Theorem 1.39".
    pub rule: &'static str,
    /// A heading for the steps panel, e.g. "Theorem 1.45 · 0∪1" (defaults to `rule`).
    pub title: Option<String>,
    /// A sentence for the steps panel.
    pub message: String,
    /// States of the final machine that appear at this step.
    pub add_states: Vec<StateId>,
    /// Arrows (from, to) of the final machine that appear at this step.
    pub add_edges: Vec<(StateId, StateId)>,
    /// States to highlight while this step is shown.
    pub focus: Vec<StateId>,
    /// The accept states as of this step, when they change during the construction.
    pub accept: Option<Vec<StateId>>,
    /// The start state as of this step, when it changes during the construction.
    pub start: Option<StateId>,
    /// The whole machine after this step (only for algorithms that remove things).
    // `Box<T>` puts a value on the heap; here it keeps `Step` small when there is no snapshot.
    pub snapshot: Option<Box<Object>>,
}

impl Step {
    /// A step that only explains (adds nothing).
    pub fn new(rule: &'static str, message: impl Into<String>) -> Step {
        Step {
            rule,
            title: None,
            message: message.into(),
            add_states: Vec::new(),
            add_edges: Vec::new(),
            focus: Vec::new(),
            accept: None,
            start: None,
            snapshot: None,
        }
    }

    /// Builder: the heading shown for this step.
    pub fn title(mut self, title: impl Into<String>) -> Step {
        self.title = Some(title.into());
        self
    }

    /// Builder: states added by this step (also focused).
    // Taking `mut self` and returning `Self` lets calls chain: `Step::new(..).states(..).edges(..)`.
    pub fn states(mut self, ids: Vec<StateId>) -> Step {
        self.focus.extend(ids.iter().copied());
        self.add_states = ids;
        self
    }

    /// Builder: arrows added by this step.
    pub fn edges(mut self, edges: Vec<(StateId, StateId)>) -> Step {
        self.add_edges = edges;
        self
    }

    /// Builder: states to highlight.
    pub fn focus(mut self, ids: Vec<StateId>) -> Step {
        self.focus = ids;
        self
    }

    /// Builder: the accept and start states as of this step.
    pub fn status(mut self, start: StateId, accept: Vec<StateId>) -> Step {
        self.start = Some(start);
        self.accept = Some(accept);
        self
    }

    /// Builder: a full snapshot of the machine after this step.
    pub fn snapshot(mut self, object: Object) -> Step {
        self.snapshot = Some(Box::new(object));
        self
    }
}

/// The steps of one algorithm run, capped by the budget.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub steps: Vec<Step>,
    /// True if steps were dropped because of `Budget::max_trace`. The result is still complete.
    pub truncated: bool,
    max: u32,
}

impl Trace {
    /// An empty trace that keeps at most `budget.max_trace` steps.
    pub fn new(budget: &Budget) -> Trace {
        Trace {
            steps: Vec::new(),
            truncated: false,
            max: budget.max_trace,
        }
    }

    /// Records a step, or marks the trace truncated if it is full.
    pub fn push(&mut self, step: Step) {
        if self.steps.len() < self.max as usize {
            self.steps.push(step);
        } else {
            self.truncated = true;
        }
    }

    /// Is there still room for another step? (Lets algorithms skip building expensive messages.)
    pub fn has_room(&self) -> bool {
        self.steps.len() < self.max as usize
    }
}
