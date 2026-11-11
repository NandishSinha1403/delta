---
title: "Automata tools: convert, minimize and simulate online"
description: "Free online tools for the theory of computation: NFA to DFA, regex to NFA, DFA to regex, minimization, equivalence, the pumping lemma, CFG to CNF, CYK, CFG to PDA, Mealy and Moore, and NFA, PDA and Turing machine simulators."
sidebar:
  label: All tools
  order: 0
---

Each tool is a page with a live Delta Pad already loaded with an example. Edit the machine and
the diagram, the checks and the step-by-step run update as you type. Every tool explains the
algorithm it runs, works one example by hand, and links to the tools next to it.

## Convert

| Tool | What it does |
|---|---|
| [NFA to DFA](/tools/nfa-to-dfa/) | The subset construction, one DFA state per reachable set of NFA states |
| [Regex to NFA](/tools/regex-to-nfa/) | Thompson's and Glushkov's constructions, side by side |
| [DFA to regex](/tools/dfa-to-regex/) | State elimination, Arden's equations or Brzozowski–McCluskey |
| [CFG to CNF](/tools/cfg-to-cnf/) | Chomsky normal form, phase by phase |
| [CFG to PDA](/tools/cfg-to-pda/) | A pushdown automaton for a grammar, and back with `to_cfg` |
| [Mealy ↔ Moore](/tools/mealy-moore/) | Convert transducers both ways and compare outputs |

## Analyse

| Tool | What it does |
|---|---|
| [DFA minimization](/tools/dfa-minimization/) | Hopcroft's algorithm and the table-filling method |
| [DFA equivalence](/tools/dfa-equivalence/) | Decide `A ≡ B` and find the shortest word that tells them apart |
| [CYK parser](/tools/cyk-parser/) | The CYK table and parse tree of a word |
| [Pumping lemma](/tools/pumping-lemma/) | Split a word `w = xyz` in a DFA's language and pump `y` |

## Simulate

| Tool | What it does |
|---|---|
| [NFA simulator](/tools/nfa-simulator/) | The set of active states after every symbol |
| [PDA simulator](/tools/pda-simulator/) | Every branch of a pushdown automaton, with its stack |
| [Turing machine simulator](/tools/turing-machine-simulator/) | The tape and head at every step, and the four verdicts |

To learn the ideas behind the tools, start with [Learn](/learn/); to test yourself, try
[Practice](/practice/).
