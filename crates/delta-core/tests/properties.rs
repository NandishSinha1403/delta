//! Property-based tests: the safety net under the golden tests
//! (docs/plan/06-ROADMAP-AND-TESTING.md §3.2). Random inputs, checked against the
//! brute-force oracles on every string up to a fixed length.

use delta_core::alphabet::Alphabet;
use delta_core::budget::Budget;
use delta_core::convert::c02_subset::{SubsetMode, subset_construction};
use delta_core::convert::c03_regex_to_nfa::regex_to_nfa;
use delta_core::convert::c04_to_gnfa::dfa_to_gnfa;
use delta_core::convert::c05_gnfa_to_regex::{RipOrder, gnfa_to_regex};
use delta_core::ids::SymId;
use delta_core::object::RegexObj;
use delta_core::oracle;
use delta_core::regex::{ReArena, ReId};
use proptest::prelude::*;

/// A random regex over {a, b}, as a plain tree (built into an arena afterwards).
#[derive(Debug, Clone)]
enum Gen {
    Empty,
    Eps,
    Sym(u32),
    Union(Box<Gen>, Box<Gen>),
    Concat(Box<Gen>, Box<Gen>),
    Star(Box<Gen>),
}

fn gen_regex() -> impl Strategy<Value = Gen> {
    let leaf = prop_oneof![
        1 => Just(Gen::Empty),
        2 => Just(Gen::Eps),
        6 => (0u32..2).prop_map(Gen::Sym),
    ];
    // Depth ≤ 5, at most ~24 nodes.
    leaf.prop_recursive(5, 24, 2, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| Gen::Union(Box::new(a), Box::new(b))),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| Gen::Concat(Box::new(a), Box::new(b))),
            inner.prop_map(|a| Gen::Star(Box::new(a))),
        ]
    })
}

fn build(arena: &mut ReArena, g: &Gen) -> ReId {
    match g {
        Gen::Empty => arena.empty(),
        Gen::Eps => arena.eps(),
        Gen::Sym(a) => arena.sym(SymId(*a)),
        Gen::Union(a, b) => {
            let (x, y) = (build(arena, a), build(arena, b));
            arena.union_raw(x, y)
        }
        Gen::Concat(a, b) => {
            let (x, y) = (build(arena, a), build(arena, b));
            arena.concat_raw(x, y)
        }
        Gen::Star(a) => {
            let x = build(arena, a);
            arena.star_raw(x)
        }
    }
}

fn regex_of(g: &Gen) -> RegexObj {
    let mut arena = ReArena::new();
    let root = build(&mut arena, g);
    RegexObj {
        arena,
        root,
        sigma: Alphabet::from_names(&["a".into(), "b".into()]),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// C3 and C2 preserve the language: R, N(R) and D(N(R)) agree on all strings of length ≤ 6.
    #[test]
    fn regex_nfa_dfa_agree(g in gen_regex()) {
        let r = regex_of(&g);
        let budget = Budget::default();
        let (n, _) = regex_to_nfa(&r, &budget).unwrap();
        let (d, _) = subset_construction(&n, SubsetMode::Reachable, &budget).unwrap();
        for w in oracle::all_strings(2, 6) {
            let want = oracle::regex_matches(&r.arena, r.root, &w);
            prop_assert_eq!(oracle::nfa_accepts(&n, &w), want, "NFA on {:?}", w);
            prop_assert_eq!(oracle::dfa_accepts(&d, &w), want, "DFA on {:?}", w);
        }
    }

    /// Round trip R → NFA → DFA → GNFA → R′ (both modes, two rip orders): L(R′) = L(R).
    #[test]
    fn round_trip_preserves_the_language(g in gen_regex()) {
        let r = regex_of(&g);
        let budget = Budget::default();
        let (n, _) = regex_to_nfa(&r, &budget).unwrap();
        let (d, _) = subset_construction(&n, SubsetMode::Reachable, &budget).unwrap();
        let (gn, _) = dfa_to_gnfa(&d, &budget);
        for (order, faithful) in [(RipOrder::Index, false), (RipOrder::Heuristic, false), (RipOrder::Index, true)] {
            let (back, _) = gnfa_to_regex(&gn, &order, faithful, &budget).unwrap();
            for w in oracle::all_strings(2, 5) {
                prop_assert_eq!(
                    oracle::regex_matches(&back.arena, back.root, &w),
                    oracle::regex_matches(&r.arena, r.root, &w),
                    "R = {}, R′ = {}, w = {:?}", r.text(), back.text(), w
                );
            }
        }
    }
}
