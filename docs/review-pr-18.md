# Independent review: fixed creature tokens

PR: https://github.com/altf4/rustmage/pull/18
Implementation reviewed: bd58c8b against main.

Two independent read-only subagents reviewed this implementation:

- Rules: all 28 new admissions and 23 factories against frozen Oracle text,
  default/explicit names, colors/subtypes/types, static abilities, cannot-block
  restrictions, simultaneous entry, post-battlefield movement prohibition,
  cessation and incarnation binding.
- Library: free-list and slot ownership, generation-safe reuse, clone/restore,
  stale source/target references, schema 8 privacy, immutable metadata,
  state accounting, compiler exactness/determinism/pruning, deck/card exclusions
  and C/Python integration.

Neither reviewer found actionable issues. Both ran 92 Rust tests, 22
compiler/importer tests and catalogue consistency. The library reviewer also
rebuilt release, passed all 11 Python binding tests and completed 20 native token
games. Neither reviewer edited files or posted externally.

Author validation additionally passed rustfmt, Clippy with warnings denied,
and 100 native supported Standard token games: 45,261 decisions, including
32,728 decisions with tokens present. Focused tests exercise trigger-order and
pending-spell replay, batch entry, tapped artifact death-token creation, counter
triggers, explicit/multicolor names, block restrictions, cessation and slot reuse.
Python tests resolve the same token spell in a native clone and after restore.

Executable Standard-card coverage is 142/5,164, with six legacy regression cards
and 23 token factories counted separately. Broader token abilities/copying,
entry counters, wider costs/modes/events, remaining card semantics and RL
performance acceptance remain incomplete.
