---
title: "Automata tools: convert, minimize and simulate online"
description: "Free online tools for the theory of computation: NFA to DFA, regex to NFA, DFA to regex, DFA minimization, equivalence, and NFA, PDA and Turing machine simulators."
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

## Analyse

| Tool | What it does |
|---|---|
| [DFA minimization](/tools/dfa-minimization/) | Hopcroft's algorithm and the table-filling method |
| [DFA equivalence](/tools/dfa-equivalence/) | Decide `A ≡ B` and find the shortest word that tells them apart |

## Simulate

| Tool | What it does |
|---|---|
| [NFA simulator](/tools/nfa-simulator/) | The set of active states after every symbol |
| [PDA simulator](/tools/pda-simulator/) | Every branch of a pushdown automaton, with its stack |
| [Turing machine simulator](/tools/turing-machine-simulator/) | The tape and head at every step, and the four verdicts |

Grammar tools (CFG to CNF, the CYK parser, CFG to PDA), the pumping lemma and Mealy ↔ Moore
conversion arrive with Delta 2.0. To learn the ideas behind the tools, start with
[Learn](/learn/); to test yourself, try [Practice](/practice/).
