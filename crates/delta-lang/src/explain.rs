//! Explain profiles (01 §13): citations keyed by algorithm and rule ids, as data. They only add
//! citations to output; they never change a result. Profiles live in `profiles/*.txt` as
//! `id = text` lines.

use crate::interp::Interp;

const SIPSER3E: &str = include_str!("../profiles/sipser3e.txt");

fn profile(name: &str) -> Option<&'static str> {
    match name {
        "sipser3e" => Some(SIPSER3E),
        _ => None,
    }
}

/// The citation for an id under a profile, if the profile has one.
pub fn lookup(profile_name: &str, id: &str) -> Option<String> {
    profile(profile_name)?.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == id).then(|| v.trim().to_string())
    })
}

pub fn known(name: &str) -> bool {
    profile(name).is_some()
}

impl Interp<'_> {
    pub fn citation(&self, id: &str) -> Option<String> {
        lookup(self.explain.as_deref()?, id)
    }
}
