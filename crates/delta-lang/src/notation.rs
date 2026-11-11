//! Notation profiles (01-LANGUAGE.md §10, SEMANTICS §10.6): how the empty word, the blank and
//! regex union are written, when reading and when printing. They never change a value.
//!
//! | profile   | `+` in regexes | empty word | blank | union printed |
//! |-----------|----------------|------------|-------|---------------|
//! | `default` | one-or-more    | ε          | ⊔     | ∪             |
//! | `hmu`     | union          | ε          | B     | +             |
//! | `linz`    | union          | λ          | □     | +             |
//!
//! Every spelling is always accepted as input (`ε`, `λ`, `eps`; `⊔`, `_`, `□`), except `B`,
//! which means the blank only under `hmu` and only in the labels of a machine with a tape (it is
//! an ordinary symbol everywhere else). The program's `use notation` sets the profile for the
//! rest of the run; every printer reads it through the functions here.

// `Cell`/`RefCell` give interior mutability; `thread_local!` makes one copy per thread. The engine
// runs one program at a time per thread, so the profile is set when a run starts and read by the
// printers without passing it through every function.
use std::cell::RefCell;

/// The spellings a profile prints.
#[derive(Clone, Debug, PartialEq)]
pub struct Notation {
    /// The profile's name (`default`, `hmu`, `linz` or `custom`).
    pub name: String,
    pub epsilon: String,
    pub blank: String,
    /// How regex union is printed: `∪`, or `+` when `+` means union.
    pub union: String,
}

impl Notation {
    pub fn default_profile() -> Notation {
        Notation {
            name: "default".to_string(),
            epsilon: "ε".to_string(),
            blank: "⊔".to_string(),
            union: "∪".to_string(),
        }
    }

    /// The profile `use notation "…"` names: `hmu`, `linz`, or a custom
    /// `{ union: "+", epsilon: "λ", blank: "□" }`. Unknown names keep the default spellings.
    pub fn from_directive(p: &str) -> Notation {
        let mut n = Notation::default_profile();
        match p {
            "hmu" => {
                n.name = "hmu".to_string();
                n.blank = "B".to_string();
                n.union = "+".to_string();
            }
            "linz" => {
                n.name = "linz".to_string();
                n.epsilon = "λ".to_string();
                n.blank = "□".to_string();
                n.union = "+".to_string();
            }
            custom if custom.trim_start().starts_with('{') => {
                n.name = "custom".to_string();
                let field = |key: &str| {
                    custom
                        .split(&format!("{key}:"))
                        .nth(1)
                        .and_then(|rest| rest.split('"').nth(1))
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                };
                if let Some(e) = field("epsilon") {
                    n.epsilon = e;
                }
                if let Some(b) = field("blank") {
                    n.blank = b;
                }
                if field("union").as_deref() == Some("+") {
                    n.union = "+".to_string();
                }
            }
            _ => {}
        }
        n
    }
}

thread_local! {
    static CURRENT: RefCell<Notation> = RefCell::new(Notation::default_profile());
}

/// Sets the profile for the rest of this run (`use notation`), or back to the default.
pub fn set(n: Notation) {
    CURRENT.with(|c| *c.borrow_mut() = n);
}

/// The default spellings again (each run starts with them).
pub fn reset() {
    set(Notation::default_profile());
}

/// The profile in use.
pub fn current() -> Notation {
    CURRENT.with(|c| c.borrow().clone())
}

/// How the empty word is printed: ε, or λ under `linz`.
pub fn eps() -> String {
    CURRENT.with(|c| c.borrow().epsilon.clone())
}

/// How the blank is printed: ⊔, B under `hmu`, □ under `linz`.
pub fn blank() -> String {
    CURRENT.with(|c| c.borrow().blank.clone())
}

/// How regex union is printed: ∪, or + where `+` means union.
pub fn union() -> String {
    CURRENT.with(|c| c.borrow().union.clone())
}

/// The symbol a profile reads as the blank in a tape machine's labels, besides ⊔ _ □ (`B` under
/// `hmu`).
pub fn blank_alias() -> Option<String> {
    CURRENT.with(|c| {
        let b = c.borrow().blank.clone();
        (!matches!(b.as_str(), "⊔" | "_" | "□")).then_some(b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_name_their_spellings() {
        let hmu = Notation::from_directive("hmu");
        assert_eq!(
            (hmu.epsilon.as_str(), hmu.blank.as_str(), hmu.union.as_str()),
            ("ε", "B", "+")
        );
        let linz = Notation::from_directive("linz");
        assert_eq!(
            (
                linz.epsilon.as_str(),
                linz.blank.as_str(),
                linz.union.as_str()
            ),
            ("λ", "□", "+")
        );
        let custom = Notation::from_directive(r#"{ union: "+", epsilon: "λ" }"#);
        assert_eq!((custom.epsilon.as_str(), custom.union.as_str()), ("λ", "+"));
        assert_eq!(
            Notation::from_directive("nonsense"),
            Notation::default_profile()
        );
    }

    #[test]
    fn the_printers_follow_the_current_profile() {
        set(Notation::from_directive("linz"));
        assert_eq!(
            (eps(), blank(), union()),
            ("λ".to_string(), "□".to_string(), "+".to_string())
        );
        assert_eq!(blank_alias(), None);
        set(Notation::from_directive("hmu"));
        assert_eq!(blank_alias().as_deref(), Some("B"));
        reset();
        assert_eq!(
            (eps(), blank(), union()),
            ("ε".to_string(), "⊔".to_string(), "∪".to_string())
        );
    }
}
