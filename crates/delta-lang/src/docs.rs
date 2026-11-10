//! Signature records for the library and the error catalogue (02-ENGINE.md §5). One record per
//! built-in feeds the generated library reference, the AI guide and their tested examples.

/// A built-in function.
pub struct Sig {
    pub name: &'static str,
    pub signature: &'static str,
    pub summary: &'static str,
    /// Algorithm ids accepted by `method:` (empty if there is no choice).
    pub methods: &'static [&'static str],
    pub area: &'static str,
    /// A complete program that uses it (run by the docs tests).
    pub example: &'static str,
}

pub const LIBRARY: &[Sig] = &[
    Sig {
        name: "to_nfa",
        signature: "to_nfa(R, method: \"textbook\")",
        summary: "An NFA for a regular expression.",
        methods: &["textbook", "thompson", "glushkov"],
        area: "Regex ↔ automata",
        example: "let N = to_nfa(re\"a b*\", method: \"glushkov\")\ncheck N accepts \"a\", \"abb\"",
    },
    Sig {
        name: "to_regex",
        signature: "to_regex(M, method: \"state-elimination\", order: [...])",
        summary: "A regular expression for a finite automaton; `order` sets which states go first.",
        methods: &["state-elimination", "arden", "brzozowski-mccluskey"],
        area: "Regex ↔ automata",
        example: "let R = to_regex(to_nfa(re\"(a b)*\"), method: \"arden\")\ncheck R ≡ re\"(a b)*\"",
    },
    Sig {
        name: "to_gnfa",
        signature: "to_gnfa(M)",
        summary: "The generalized NFA in special form: new start and accept states, one regex edge between every pair.",
        methods: &[],
        area: "Regex ↔ automata",
        example: "let G = to_gnfa(to_nfa(re\"a ∪ b\"))\ncheck G ≡ re\"a ∪ b\"",
    },
    Sig {
        name: "simplify",
        signature: "simplify(R)",
        summary: "Rewrites a regex with the identities R ∪ ∅ = R, R ε = R, R ∅ = ∅, ∅* = ε, (R*)* = R*.",
        methods: &[],
        area: "Regex ↔ automata",
        example: "check simplify(re\"(0 ∪ ∅) ε\") ≡ re\"0\"",
    },
    Sig {
        name: "determinize",
        signature: "determinize(N)",
        summary: "The subset construction: a DFA whose states are ε-closed sets of N's states (only reachable sets).",
        methods: &["subset"],
        area: "Automata",
        example: "let D = determinize(to_nfa(re\"(a ∪ b)* a\"))\ncheck D ≡ re\"(a ∪ b)* a\"\nassert minimize(D).size == 2",
    },
    Sig {
        name: "minimize",
        signature: "minimize(D, method: \"hopcroft\")",
        summary: "The minimal DFA; a state of the result is the set of the states it merges.",
        methods: &["hopcroft", "moore", "brzozowski", "table"],
        area: "Automata",
        example: "let M = minimize(determinize(to_nfa(re\"(a ∪ b)* a b\")), method: \"moore\")\nassert M.size == 3",
    },
    Sig {
        name: "complete",
        signature: "complete(M)",
        summary: "Adds a dead state `:dead` for every missing transition.",
        methods: &[],
        area: "Automata",
        example: "dfa M (partial) {\n  alphabet a, b\n  start q0\n  accept q1\n  q0 -a-> q1\n}\nassert complete(M).size == 3",
    },
    Sig {
        name: "trim",
        signature: "trim(M)",
        summary: "Keeps only states that are reachable and can reach an accepting state.",
        methods: &[],
        area: "Automata",
        example: "nfa N {\n  start p\n  accept q\n  p -a-> q\n  p -b-> r\n}\nassert trim(N).size == 2",
    },
    Sig {
        name: "remove_epsilon",
        signature: "remove_epsilon(N)",
        summary: "An NFA without ε-moves for the same language.",
        methods: &[],
        area: "Automata",
        example: "let N = remove_epsilon(to_nfa(re\"a* b\"))\ncheck N ≡ re\"a* b\"",
    },
    Sig {
        name: "reverse",
        signature: "reverse(M)",
        summary: "An NFA for the reversed language.",
        methods: &[],
        area: "Automata",
        example: "check reverse(to_nfa(re\"a b\")) ≡ re\"b a\"",
    },
    Sig {
        name: "complement",
        signature: "complement(M)",
        summary: "A DFA for the complement (relative to M's alphabet).",
        methods: &[],
        area: "Automata",
        example: "check complement(to_nfa(re\"a*\" over {'a', 'b'})) accepts \"b\"",
    },
    Sig {
        name: "union",
        signature: "union(A, B)",
        summary: "The product DFA accepting when either accepts; states are pairs.",
        methods: &[],
        area: "Automata",
        example: "check union(to_nfa(re\"a\"), to_nfa(re\"b\")) ≡ re\"a ∪ b\"",
    },
    Sig {
        name: "intersect",
        signature: "intersect(A, B)",
        summary: "The product DFA accepting when both accept.",
        methods: &[],
        area: "Automata",
        example: "check intersect(to_nfa(re\"a*\"), to_nfa(re\"(a a)*\")) ≡ re\"(a a)*\"",
    },
    Sig {
        name: "difference",
        signature: "difference(A, B)",
        summary: "The product DFA accepting when A accepts and B doesn't.",
        methods: &[],
        area: "Automata",
        example: "check difference(to_nfa(re\"a*\"), to_nfa(re\"(a a)*\")) accepts \"a\", \"aaa\"",
    },
    Sig {
        name: "concat",
        signature: "concat(A, B)",
        summary: "An NFA for the concatenation (ε-edges from A's accepting states).",
        methods: &[],
        area: "Automata",
        example: "check concat(to_nfa(re\"a\"), to_nfa(re\"b\")) ≡ re\"a b\"",
    },
    Sig {
        name: "star",
        signature: "star(A)",
        summary: "An NFA for the star (a new accepting start state).",
        methods: &[],
        area: "Automata",
        example: "check star(to_nfa(re\"a b\")) ≡ re\"(a b)*\"",
    },
    Sig {
        name: "product",
        signature: "product(A, B, keep)",
        summary: "The product DFA accepting where `keep(a_accepts, b_accepts)` is true.",
        methods: &[],
        area: "Automata",
        example: "let X = product(to_nfa(re\"a*\"), to_nfa(re\"b*\"), fn(x, y) = x != y)\ncheck X accepts \"a\", \"b\"\ncheck X rejects \"\"",
    },
    Sig {
        name: "equivalent",
        signature: "equivalent(A, B)",
        summary: "Whether two languages are equal (decidable for Finite and Regular).",
        methods: &[],
        area: "Decisions",
        example: "assert equivalent(re\"(a ∪ b)*\", re\"(a* b*)*\")",
    },
    Sig {
        name: "subset_of",
        signature: "subset_of(A, B)",
        summary: "Whether L(A) ⊆ L(B).",
        methods: &[],
        area: "Decisions",
        example: "assert subset_of(re\"a b\", re\"a (a ∪ b)\")",
    },
    Sig {
        name: "nullable",
        signature: "nullable(G)",
        summary: "The variables that derive ε.",
        methods: &[],
        area: "Grammars",
        example: "cfg G {\n  S -> A b | ε\n  A -> a | ε\n}\nassert nullable(G) == {'S', 'A'}",
    },
    Sig {
        name: "useless",
        signature: "useless(G)",
        summary: "The variables that derive no word or can't be reached from the start; trim(G) drops them.",
        methods: &[],
        area: "Grammars",
        example: "cfg G {\n  S -> a | D\n  D -> d D\n}\nassert useless(G) == {'D'}\nassert trim(G).variables == {'S'}",
    },
    Sig {
        name: "cnf",
        signature: "cnf(G)",
        summary: "An equivalent grammar in Chomsky normal form (START, DEL, UNIT, TERM, BIN, then trim); steps(cnf(G)) lists each phase.",
        methods: &[],
        area: "Grammars",
        example: "cfg G {\n  S -> a S b | ε\n}\nlet C = cnf(G)\nassert C.start == '<S0>'\ncheck C ≡ G up to 8",
    },
    Sig {
        name: "parse",
        signature: "parse(G, w, method: \"earley\")",
        summary: "A parse tree of w (T.root, T.children, T.yield, T.text), or none if w ∉ L(G). CYK parses with cnf(G); steps(T) lists the table cells.",
        methods: &["earley", "cyk"],
        area: "Grammars",
        example: "cfg G {\n  S -> a S b | ε\n}\nlet T = parse(G, \"aabb\")\nassert T.yield == \"aabb\"\nassert parse(G, \"aab\", method: \"cyk\") == none",
    },
    Sig {
        name: "derive",
        signature: "derive(G, w)",
        summary: "The sentential forms of a leftmost derivation of w, or none.",
        methods: &[],
        area: "Grammars",
        example: "cfg G {\n  S -> a S b | ε\n}\nassert derive(G, \"ab\") == [\"S\", \"aSb\", \"ab\"]",
    },
    Sig {
        name: "to_pda",
        signature: "to_pda(G)",
        summary: "A pushdown automaton for L(G): states :start, :loop, :accept; the stack holds the rest of a leftmost derivation.",
        methods: &[],
        area: "Grammars",
        example: "cfg G {\n  S -> a S b | ε\n}\ncheck to_pda(G) accepts \"\", \"ab\", \"aabb\"",
    },
    Sig {
        name: "to_cfg",
        signature: "to_cfg(P)",
        summary: "A grammar for a pushdown automaton's language, with variables <p,q> (trimmed).",
        methods: &[],
        area: "Grammars",
        example: "pda P {\n  alphabet a, b\n  start q0\n  accept q1\n  q0 -a, ε → A-> q0\n  q0 -ε, ε → ε-> q1\n  q1 -b, A → ε-> q1\n}\ncheck to_cfg(P) ≡ P up to 6",
    },
    Sig {
        name: "mealy_to_moore",
        signature: "mealy_to_moore(M)",
        summary: "A Moore machine with the same outputs; its states are (q, o).",
        methods: &[],
        area: "Machines",
        example: "mealy M {\n  alphabet a, b\n  start q0\n  q0 -a / 1-> q0; q0 -b / 0-> q0\n}\nassert mealy_to_moore(M).output(\"ab\") == M.output(\"ab\")",
    },
    Sig {
        name: "moore_to_mealy",
        signature: "moore_to_mealy(M)",
        summary: "A Mealy machine whose output is M's without the first λ(q₀).",
        methods: &[],
        area: "Machines",
        example: "moore M {\n  alphabet a\n  start q0\n  λ(q0) = 0\n  λ(q1) = 1\n  q0 -a-> q1; q1 -a-> q0\n}\nassert M.output(\"aa\") == \"0\" ++ moore_to_mealy(M).output(\"aa\")",
    },
    Sig {
        name: "is_empty",
        signature: "is_empty(L)",
        summary: "Whether the language is empty (decidable up to CF).",
        methods: &[],
        area: "Decisions",
        example: "assert is_empty(re\"a\" ∩ re\"b\")",
    },
    Sig {
        name: "is_finite",
        signature: "is_finite(L)",
        summary: "Whether the language is finite (decidable up to CF).",
        methods: &[],
        area: "Decisions",
        example: "assert is_finite(re\"a ∪ a b\")\nassert not is_finite(re\"a*\")",
    },
    Sig {
        name: "is_universal",
        signature: "is_universal(L)",
        summary: "Whether every word over its alphabet is in it (decidable up to DCF).",
        methods: &[],
        area: "Decisions",
        example: "assert is_universal(re\"(a ∪ b)*\")",
    },
    Sig {
        name: "shortest",
        signature: "shortest(L)",
        summary: "The shortlex-least word of a regular language, or none.",
        methods: &[],
        area: "Decisions",
        example: "assert shortest(re\"a a* b\") == \"ab\"",
    },
    Sig {
        name: "words",
        signature: "words(L, upto: n)",
        summary: "Every word of a regular language up to length n.",
        methods: &[],
        area: "Decisions",
        example: "assert words(re\"a*\", upto: 2) == {\"\", \"a\", \"aa\"}",
    },
    Sig {
        name: "count",
        signature: "count(L, length: n) · count(w, s)",
        summary: "The number of words of length n in a regular language; or how often symbol s occurs in word w.",
        methods: &[],
        area: "Decisions",
        example: "assert count(re\"(a ∪ b)*\", length: 3) == 8\nassert count(\"abba\", 'b') == 2",
    },
    Sig {
        name: "steps",
        signature: "steps(x)",
        summary: "The recorded steps of the construction that made x.",
        methods: &[],
        area: "Decisions",
        example: "assert len(steps(determinize(to_nfa(re\"a\")))) > 0",
    },
    Sig {
        name: "len",
        signature: "len(x)",
        summary: "The length of a word, list, set or map.",
        methods: &[],
        area: "Words and sets",
        example: "assert len(\"abc\") == 3 and len({1, 2}) == 2",
    },
    Sig {
        name: "rev",
        signature: "rev(x)",
        summary: "A reversed word or list, or the reversed language.",
        methods: &[],
        area: "Words and sets",
        example: "assert rev(\"abc\") == \"cba\"",
    },
    Sig {
        name: "prefixes",
        signature: "prefixes(w) · suffixes(w) · substrings(w)",
        summary: "The set of prefixes, suffixes or substrings of a word (ε included).",
        methods: &[],
        area: "Words and sets",
        example: "assert prefixes(\"ab\") == {\"\", \"a\", \"ab\"}\nassert len(substrings(\"aab\")) == 6",
    },
    Sig {
        name: "power_set",
        signature: "power_set(S)",
        summary: "Every subset of a set (up to 20 elements).",
        methods: &[],
        area: "Words and sets",
        example: "assert len(power_set({1, 2, 3})) == 8",
    },
    Sig {
        name: "set",
        signature: "set(x) · list(x) · sort(x)",
        summary: "Converts between collections; sort orders canonically.",
        methods: &[],
        area: "Words and sets",
        example: "assert set([2, 1, 2]) == {1, 2}\nassert sort([3, 1, 2]) == [1, 2, 3]",
    },
    Sig {
        name: "min",
        signature: "min(x) · max(x) · sum(x) · abs(n)",
        summary: "Smallest, largest (canonical order) and sum of a collection; absolute value.",
        methods: &[],
        area: "Words and sets",
        example: "assert min({3, 1}) == 1 and sum([1, 2, 3]) == 6 and abs(-4) == 4",
    },
    Sig {
        name: "keys",
        signature: "keys(m) · values(m) · first(x) · last(x)",
        summary: "A map's keys (a set) and values (a list); a collection's first and last element.",
        methods: &[],
        area: "Words and sets",
        example: "let m = { :a: 1, :b: 2 }\nassert keys(m) == {:a, :b} and values(m) == [1, 2]",
    },
];

/// An error code: its meaning and, where one exists, a program that produces it.
pub struct ErrorDoc {
    pub code: &'static str,
    pub title: &'static str,
    pub explain: &'static str,
    pub example: &'static str,
}

pub const ERRORS: &[ErrorDoc] = &[
    ErrorDoc {
        code: "E001",
        title: "Unterminated word or regex",
        explain: "A word \"…\" or a regex re\"…\" has no closing quote on its line.",
        example: "let w = \"abc",
    },
    ErrorDoc {
        code: "E002",
        title: "Unexpected character",
        explain: "A character that isn't part of Delta's syntax.",
        example: "let x = 1 $ 2",
    },
    ErrorDoc {
        code: "E003",
        title: "Unterminated block comment",
        explain: "A /* comment has no closing */.",
        example: "/* never closed",
    },
    ErrorDoc {
        code: "E004",
        title: "Integer literal too large",
        explain: "int is 64-bit; the literal doesn't fit.",
        example: "let n = 99999999999999999999",
    },
    ErrorDoc {
        code: "E005",
        title: "Bad symbol literal",
        explain: "A symbol is one character, or a name in angle brackets like '<RESET>'.",
        example: "let s = 'ab'",
    },
    ErrorDoc {
        code: "E101",
        title: "Unexpected token",
        explain: "The parser expected something else here; the message says what. Two statements on one line need a `;`.",
        example: "let x 1",
    },
    ErrorDoc {
        code: "E102",
        title: "Reserved word",
        explain: "`choose`, `import` and `reduce` are reserved for later versions of Delta.",
        example: "let choose = 1",
    },
    ErrorDoc {
        code: "E103",
        title: "ASCII arrow inside a bare label",
        explain: "In `-label->` the first `->` ends the edge, so a label containing `->` needs brackets: `-[a, X -> YZ]->`. The quick fix adds them.",
        example: "pda P {\n  start q0\n  q0 -a, X -> YZ-> q1\n}",
    },
    ErrorDoc {
        code: "E104",
        title: "Unbalanced brackets in an edge",
        explain: "A bracketed edge `-[ … ]->` must close with `]->` on the same line, with balanced brackets inside.",
        example: "dfa M {\n  start q0\n  q0 -[a-> q0\n}",
    },
    ErrorDoc {
        code: "E105",
        title: "Unterminated edge",
        explain: "An edge starts with `-` but has no `->` before the target state.",
        example: "dfa M {\n  start q0\n  q0 -a q1\n}",
    },
    ErrorDoc {
        code: "E106",
        title: "Unknown explore clause",
        explain: "explore takes alphabet, start, step, accept and edges clauses.",
        example: "let M = explore {\n  alphabet {'a'}\n  start 0\n  jump q -> q\n}",
    },
    ErrorDoc {
        code: "E107",
        title: "explore without an alphabet",
        explain: "An explore construction needs an `alphabet` clause: the input alphabet of the machine it builds.",
        example: "let M = explore {\n  start 0\n  step q on a -> q\n  accept q if true\n}",
    },
    ErrorDoc {
        code: "E108",
        title: "explore without a start",
        explain: "An explore construction needs a `start` clause.",
        example: "let M = explore {\n  alphabet {'a'}\n  accept q if true\n}",
    },
    ErrorDoc {
        code: "E109",
        title: "Bad pattern",
        explain: "`let` binds a name or a tuple of names.",
        example: "let 1 = 2",
    },
    ErrorDoc {
        code: "E110",
        title: "Expected an expression",
        explain: "Something that isn't an expression appears where a value is needed.",
        example: "let x = )",
    },
    ErrorDoc {
        code: "E201",
        title: "Unknown name",
        explain: "The name is not defined. The quick fix suggests a close name in scope.",
        example: "let total = 1\nprint totl",
    },
    ErrorDoc {
        code: "E202",
        title: "Rebinding a `let`",
        explain: "`let` binds once; declare the name with `var` to rebind it.",
        example: "let x = 1\nx = 2",
    },
    ErrorDoc {
        code: "E203",
        title: "Assigning an undeclared name",
        explain: "Declare a variable with `var x = …` before assigning to it.",
        example: "y = 2",
    },
    ErrorDoc {
        code: "E204",
        title: "Argument of the wrong type",
        explain: "The argument doesn't match the parameter's type annotation.",
        example: "fn twice(k: int) = 2 * k\nlet b = twice(\"a\")",
    },
    ErrorDoc {
        code: "E205",
        title: "Infinite set outside lang",
        explain: "Σ* has every word, so it's allowed only inside `lang`. Use Σ^n or Σ^≤n for a finite set.",
        example: "let Σ = {'a', 'b'}\nlet all = Σ*",
    },
    ErrorDoc {
        code: "E206",
        title: "Wrong number of arguments",
        explain: "The call passes a different number of arguments than the function takes.",
        example: "fn id(a) = a\nlet b = id(1, 2)",
    },
    ErrorDoc {
        code: "E207",
        title: "Calling a model name",
        explain: "Model names start machine literals; conversions are verbs: determinize(N), to_nfa(R), to_gnfa(M).",
        example: "let D = dfa(re\"a\")",
    },
    ErrorDoc {
        code: "E301",
        title: "Unknown model or property",
        explain: "The model, or one of its properties, doesn't exist.",
        example: "model m { input sideways }\nprint m",
    },
    ErrorDoc {
        code: "E302",
        title: "No start state",
        explain: "Every machine needs `start q`.",
        example: "dfa M (partial) {\n  accept q0\n  q0 -a-> q0\n}",
    },
    ErrorDoc {
        code: "E303",
        title: "Undeclared state",
        explain: "With a `states` field, every state used must be declared (this catches typos).",
        example: "dfa M (partial) {\n  states q0\n  start q0\n  accept q0\n  q0 -a-> q1\n}",
    },
    ErrorDoc {
        code: "E304",
        title: "Symbol not in the alphabet",
        explain: "An edge reads a symbol that isn't in the declared alphabet.",
        example: "dfa M (partial) {\n  alphabet a\n  start q0\n  accept q0\n  q0 -b-> q0\n}",
    },
    ErrorDoc {
        code: "E305",
        title: "Nondeterminism in a deterministic model",
        explain: "Two edges from the same state can apply in the same configuration.",
        example: "dfa M (partial) {\n  start q0\n  accept q1\n  q0 -a-> q0\n  q0 -a-> q1\n}",
    },
    ErrorDoc {
        code: "E306",
        title: "Missing transition in a total model",
        explain: "A dfa needs a move for every state and symbol. Quick fixes: add a dead state, or mark the machine (partial).",
        example: "dfa M {\n  alphabet a, b\n  start q0\n  accept q0\n  q0 -a-> q0\n}",
    },
    ErrorDoc {
        code: "E307",
        title: "ε-move in a model without epsilon",
        explain: "dfa (and other models without `epsilon`) read a symbol on every edge.",
        example: "dfa M (partial) {\n  start q0\n  accept q1\n  q0 -ε-> q1\n}",
    },
    ErrorDoc {
        code: "E308",
        title: "Missing field in strict mode",
        explain: "`use strict` requires `states` and `alphabet` to be written.",
        example: "use strict\ndfa M {\n  start q0\n  accept q0\n  q0 -a-> q0\n}",
    },
    ErrorDoc {
        code: "E309",
        title: "Bad label",
        explain: "The label doesn't fit the model's label grammar (see Models).",
        example: "tm T {\n  start q0\n  q0 -0 → x, Q-> q0\n}",
    },
    ErrorDoc {
        code: "E312",
        title: "Unknown option",
        explain: "Machine options are partial, missing: …, accept: by … and initial stack Z.",
        example: "dfa M (fast) {\n  start q0\n  accept q0\n  q0 -a-> q0\n}",
    },
    ErrorDoc {
        code: "E313",
        title: "Initial stack symbol not in Γ",
        explain: "With a declared stack alphabet, `initial stack Z` must use one of its symbols.",
        example: "pda P (initial stack Z) {\n  stack A\n  start q0\n  q0 -a, A → ε-> q0\n}",
    },
    ErrorDoc {
        code: "E401",
        title: "Regex syntax error",
        explain: "The regex is malformed, for example an unbalanced parenthesis.",
        example: "let R = re\"(a\"",
    },
    ErrorDoc {
        code: "E402",
        title: "Σ needs an alphabet",
        explain: "Σ in a regex means any symbol; add `over {…}` when the regex has no other symbols.",
        example: "let R = re\"Σ*\"",
    },
    ErrorDoc {
        code: "E403",
        title: "Grammar syntax error",
        explain: "Each grammar line is a rule `A -> … | …`.",
        example: "let G = cfg { S a }",
    },
    ErrorDoc {
        code: "E501",
        title: "Integer overflow",
        explain: "int is 64-bit.",
        example: "let big = 9_223_372_036_854_775_807 + 1",
    },
    ErrorDoc {
        code: "E502",
        title: "Division by zero",
        explain: "/ and % by 0.",
        example: "let x = 1 / 0",
    },
    ErrorDoc {
        code: "E503",
        title: "Index out of range",
        explain: "Indexes start at 0.",
        example: "let x = [1, 2][5]",
    },
    ErrorDoc {
        code: "E504",
        title: "Wrong kind of value",
        explain: "An operation was applied to a value it doesn't take.",
        example: "let x = 1 ++ 2",
    },
    ErrorDoc {
        code: "E505",
        title: "No such member",
        explain: "The record or object has no member with that name.",
        example: "let r = { a: 1 }\nlet x = r.b",
    },
    ErrorDoc {
        code: "E506",
        title: "Pattern doesn't match",
        explain: "The tuple pattern has a different number of parts.",
        example: "let (a, b) = (1, 2, 3)",
    },
    ErrorDoc {
        code: "E507",
        title: "Wrong arguments",
        explain: "A call with the wrong number of arguments, or an unknown named argument.",
        example: "let f = fn(x) = x\nlet y = f(1, 2)",
    },
    ErrorDoc {
        code: "E508",
        title: "Not a function",
        explain: "Only functions (and parameterized models) can be called.",
        example: "let x = 3\nlet y = x(1)",
    },
    ErrorDoc {
        code: "E509",
        title: "Not available yet",
        explain: "The built-in exists in the language but this version doesn't implement it.",
        example: "let x = myhill_nerode(1)",
    },
    ErrorDoc {
        code: "E512",
        title: "No transition (missing: error)",
        explain: "The machine says a missing transition is an error, and a run reached one.",
        example: "tm T (missing: error) {\n  alphabet 0\n  start q0\n  accept qa\n  q0 -0 → R-> q0\n}\ncheck T accepts \"0\"",
    },
    ErrorDoc {
        code: "E513",
        title: "rename merges states",
        explain: "M.rename(f) must map different states to different values.",
        example: "dfa M {\n  alphabet a\n  start [0]\n  accept [1]\n  [0] -a-> [1]\n  [1] -a-> [0]\n}\nlet N = M.rename(fn(q) = 0)",
    },
    ErrorDoc {
        code: "E514",
        title: "Unknown file format",
        explain: "save picks the format from the file's extension: .svg, .png, .pdf, .tex, .txt or .json.",
        example: "save {1} to \"set.gif\"",
    },
    ErrorDoc {
        code: "E515",
        title: "Path leaves the folder",
        explain: "A save path is relative and stays inside the folder the program runs in: no leading /, no drive letter, no `..`.",
        example: "save {1} to \"../set.txt\"",
    },
    ErrorDoc {
        code: "E516",
        title: "Nothing to draw",
        explain: "Only machines can be saved as .svg, .png, .pdf or .tex. Save other values as .txt or .json.",
        example: "save {1, 2} to \"set.svg\"",
    },
    ErrorDoc {
        code: "B601",
        title: "Step budget reached",
        explain: "The program stopped after the step budget. Outputs so far stay; raise it with budget { steps: … }.",
        example: "budget { steps: 1_000 }\nwhile true { }",
    },
    ErrorDoc {
        code: "B602",
        title: "States budget reached",
        explain: "A construction created more states than the states budget.",
        example: "budget { states: 10 }\nlet M = explore {\n  alphabet {'a'}\n  start 0\n  step n on a -> n + 1\n  accept n if false\n}",
    },
    ErrorDoc {
        code: "B603",
        title: "Depth budget reached",
        explain: "Recursion went deeper than the depth budget.",
        example: "fn f(n) = f(n + 1)\nlet x = f(0)",
    },
    ErrorDoc {
        code: "B604",
        title: "Output budget reached",
        explain: "The program produced more outputs than the output budget.",
        example: "budget { output: 3 }\nfor i ∈ 0..10 { print i }",
    },
    ErrorDoc {
        code: "B605",
        title: "Membership undecided",
        explain: "A run on the word didn't finish within the budget, so membership is unknown.",
        example: "budget { steps: 100 }\ntm T {\n  alphabet a\n  start q0\n  accept qa\n  q0 -a → R-> q1\n  q1 -⊔ → R-> q1\n}\nassert T.accepts(\"a\")",
    },
    ErrorDoc {
        code: "E701",
        title: "Undecidable question",
        explain: "The question is undecidable for the languages' class. The quick fix uses a bounded check: `up to 12`.",
        example: "lang C = cfg { S -> 0 S 1 | ε }\nlang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }\ncheck C ≡ P",
    },
    ErrorDoc {
        code: "E702",
        title: "Decidable, not implemented",
        explain: "The question is decidable for this class, but Delta doesn't implement the algorithm (equivalence of DCF languages). Use a bounded check.",
        example: "dpda D {\n  start q0\n  accept q0\n  q0 -0, ε → A-> q0\n}\nlang L = D\ncheck L ≡ L",
    },
    ErrorDoc {
        code: "E703",
        title: "False class claim",
        explain: "`L as c by E`: the evidence disagrees with the language on some word up to length 8.",
        example: "lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') } as Regular by re\"(0 1)*\"",
    },
    ErrorDoc {
        code: "W401",
        title: "Ambiguous + in a regex",
        explain: "Under the default notation `+` means one-or-more. Quick fixes: `a ∪ b` (union) or `a⁺b` (one-or-more).",
        example: "lang L = re\"a+b\"",
    },
    ErrorDoc {
        code: "X999",
        title: "Internal error",
        explain: "Something went wrong inside the engine. Your code is kept; please report it.",
        example: "",
    },
];

/// The library reference page (Markdown, for the docs site).
pub fn library_page() -> String {
    let mut out = String::from(
        "---\ntitle: Library\ndescription: Every built-in function, its methods and an example.\n---\n\n<!-- Generated by `deltac gen-docs` from the signature records in crates/delta-lang/src/docs.rs. Do not edit. -->\n\nEvery built-in takes an optional `method:` naming the algorithm, and its result remembers its\nsteps (`steps(x)`). Results of constructions say what their states are: see\n[Conversions](/docs/tour/07-conversions/).\n",
    );
    let mut area = "";
    for s in LIBRARY {
        if s.area != area {
            area = s.area;
            out.push_str(&format!("\n## {area}\n"));
        }
        out.push_str(&format!(
            "\n### `{}`\n\n`{}`\n\n{}\n",
            s.name, s.signature, s.summary
        ));
        if !s.methods.is_empty() {
            let m: Vec<String> = s.methods.iter().map(|m| format!("`\"{m}\"`")).collect();
            out.push_str(&format!("\nMethods: {}.\n", m.join(", ")));
        }
        out.push_str(&format!("\n```delta\n{}\n```\n", s.example));
    }
    out
}

/// The error index and one page per code.
pub fn error_pages() -> Vec<(String, String)> {
    let mut pages = Vec::new();
    let mut index = String::from(
        "---\ntitle: Errors\ndescription: Every diagnostic code, what it means, and an example.\n---\n\n<!-- Generated by `deltac gen-docs`. Do not edit. -->\n\n| Range | Kind |\n|---|---|\n| E0xx | lexical |\n| E1xx | syntax |\n| E2xx | names and types |\n| E3xx | machine and model rules |\n| E4xx | regex and grammar |\n| E5xx | runtime |\n| B6xx | budget outcomes |\n| E7xx | decidability |\n| W4xx | warnings |\n| X999 | internal |\n\n",
    );
    for e in ERRORS {
        index.push_str(&format!(
            "- [{}](/docs/errors/{}/): {}\n",
            e.code,
            e.code.to_lowercase(),
            e.title
        ));
        let mut page = format!(
            "---\ntitle: \"{}: {}\"\ndescription: \"{}\"\n---\n\n<!-- Generated by `deltac gen-docs`. Do not edit. -->\n\n{}\n",
            e.code,
            e.title,
            e.explain.replace('"', "'"),
            e.explain
        );
        if !e.example.is_empty() {
            let kind = match e.code.chars().next() {
                Some('E') => "delta error",
                Some('B') => "delta outcome",
                _ => "delta",
            };
            page.push_str(&format!("\n## Example\n\n```{kind}\n{}\n```\n", e.example));
        }
        pages.push((format!("errors/{}.md", e.code.to_lowercase()), page));
    }
    pages.insert(0, ("errors/index.md".to_string(), index));
    pages
}

/// The AI guide: a compact, complete description of the language for AI assistants.
pub fn ai_guide() -> String {
    let mut out = String::from(
        "# Delta 2: a guide for AI assistants\n\nDelta is a language whose values are machines, languages, words, grammars and computations.\nRun programs at https://delta.nandish.dev/pad/ or with `deltac run --json FILE`.\n\n## Syntax essentials\n\n- One statement per line, or separate with `;`. Comments: `//`, `/* */`.\n- Values: ints (64-bit, Euclidean `/` `%`), symbols `'a'`, words `\"ab\"` (ε = \"\"), atoms `:q0`,\n  sets `{1, 2}`, lists `[1, 2]`, tuples, records `{ from: p }`, maps `{ :q0: 0 }`, functions.\n- `let` binds once; `var` can be rebound. `fn f(x) = …` or `fn f(x) { … return … }`.\n- Set-builder: `{ e | x ∈ S, cond }`; filter: `{ x ∈ S | cond }`.\n- Machines: `dfa M { start q0; accept q1; q0 -a-> q1 }`. Labels: dfa/nfa `a` or `ε`; pda\n  `a, X → YZ`; tm `0 → x, R` or `x → R`; mealy `a / 1`; keyword form `read a; pop X; push YZ`.\n  Inside `-[ … ]->` an ASCII `->` is allowed. A bare state name is an atom (`M.start == :q0`).\n- Options: `dfa M (partial)`, `tm T (missing: error)`, `pda P (accept: by empty-stack)`,\n  `pda P (initial stack Z)`.\n- Languages: `lang L = { w ∈ {a,b}* | cond }`, `lang R = re\"(a ∪ b)* b\"`, `lang C = cfg { S -> a S b | ε }`.\n  Classes: Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any.\n- Tests: `check M accepts \"w\", …`, `check A ≡ B`, `check A ≡ B up to n`, `assert cond`.\n- Output: `print a, b`; `show M` (a diagram); `save M to \"m.svg\"` (.svg .png .pdf .tex .txt .json;\n  relative paths only; `deltac` writes the file, the Pad downloads it).\n  Undecidable questions (≡ for CF, …) are error E701: use `up to n`.\n- Constructions: `explore { alphabet S; start s; step R on a -> e; accept R if c }`,\n  `fixpoint X from init { e }`.\n- Runs end in accept, reject, loops or unknown: `M.run(w).verdict == :accept`.\n\n## Library\n\n",
    );
    for s in LIBRARY {
        out.push_str(&format!("- `{}`: {}", s.signature, s.summary));
        if !s.methods.is_empty() {
            out.push_str(&format!(" Methods: {}.", s.methods.join(", ")));
        }
        out.push('\n');
    }
    out.push_str("\n## Error codes\n\n");
    for e in ERRORS {
        out.push_str(&format!("- {}: {}. {}\n", e.code, e.title, e.explain));
    }
    out
}

/// The JSON Schema of `deltac run --json` (02-ENGINE.md §7).
pub fn output_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://delta.nandish.dev/schema/output-v2.json",
        "title": "Delta 2 run output",
        "type": "object",
        "required": ["delta_version", "ok", "diagnostics", "outputs"],
        "properties": {
            "delta_version": { "type": "string" },
            "ok": { "type": "boolean", "description": "No error diagnostics and no budget stop." },
            "diagnostics": { "type": "array", "items": {
                "type": "object",
                "required": ["code", "severity", "line", "col", "from", "to", "message", "fixes"],
                "properties": {
                    "code": { "type": "string" },
                    "severity": { "enum": ["error", "warning", "info"] },
                    "line": { "type": "integer" }, "col": { "type": "integer", "description": "1-based, UTF-16" },
                    "from": { "type": "integer" }, "to": { "type": "integer" },
                    "message": { "type": "string" },
                    "fixes": { "type": "array", "items": { "type": "object", "properties": {
                        "title": { "type": "string" },
                        "edits": { "type": "array", "items": { "type": "object", "properties": {
                            "from": { "type": "integer" }, "to": { "type": "integer" }, "insert": { "type": "string" } } } } } } }
                } } },
            "outputs": { "type": "array", "items": {
                "type": "object", "required": ["kind"],
                "properties": {
                    "kind": { "enum": ["print", "show", "test", "trace", "save"] },
                    "path": { "type": "string", "description": "save: relative, `/`-separated, never leaves the working folder" },
                    "format": { "enum": ["svg", "png", "pdf", "tex", "txt", "json"] },
                    "line": { "type": "integer" },
                    "text": { "type": "string" },
                    "name": { "type": "string" },
                    "pass": { "type": "boolean" },
                    "bounded": { "type": ["integer", "null"] },
                    "witness": { "type": "string" },
                    "verdict": { "enum": ["accept", "reject", "loops", "unknown"] },
                    "value": { "type": "object" }
                } } },
            "stats": { "type": "object", "properties": { "steps": { "type": "integer" } } }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_example_produces_its_code() {
        for e in ERRORS {
            if e.example.is_empty() {
                continue;
            }
            let src = e.example.to_string();
            let out = std::thread::Builder::new()
                .stack_size(256 * 1024 * 1024)
                .spawn(move || crate::run(&src))
                .and_then(|h| h.join().map_err(|_| std::io::Error::other("panicked")))
                .unwrap_or(serde_json::Value::Null);
            let codes: Vec<String> = out["diagnostics"]
                .as_array()
                .map(|d| {
                    d.iter()
                        .filter_map(|x| x["code"].as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            assert!(
                codes.iter().any(|c| c == e.code),
                "{}: got {codes:?}",
                e.code
            );
        }
    }
}
