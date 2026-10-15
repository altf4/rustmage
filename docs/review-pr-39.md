# Independent review: explicit spell modes

PR: https://github.com/altf4/rustmage/pull/39
Implementation reviewed: 1d8b7d8 against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 11 admissions, modes before targets/payment, funding and illegal
  action rejection, self-counter exclusion, selected-mode resolution/fizzle/
  countering, owned continuations and schema 15 visibility.
- Library: staged mode/target/payment ownership, funding legality, snapshot
  replay, selected-program dispatch, schema 15 privacy and whole-card admission.

Both reported no actionable findings. Both passed 195 Rust tests, 35 script tests
and catalogue consistency. The library reviewer also rebuilt release, passed all
22 Python tests and completed 20 native games covering 120 mode choices.
Neither reviewer edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 51,504 decisions, 607 mode choices (188 first/419 second),
204 target choices, 1,237 staged payments and 170 library-choice decisions.
Native fixtures cover missing/hexproof targets, exact funding, illegal indices,
shortcut rejection, per-stage clone/restore, payment mana windows, self-counter
rejection, selected program dispatch, fizzle/countering, fresh casts, mode-specific
graveyard/BF predicates and partial surveil replay. Python exercises targeted
Abrade and targetless Spellgyre through the C ABI.

Coverage is 523/5,164 Standard identities, six regression cards and 31 token
factories. Choose-many/repeated modes, modal costs/abilities, broader targeting,
remaining card/rules semantics and full native RL/search acceptance remain
outstanding. These reviews do not certify full Standard conformance.
