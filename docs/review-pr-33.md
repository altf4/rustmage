# Independent review: face-up exile and linked returns

PR: https://github.com/altf4/rustmage/pull/33
Implementation reviewed: c79b551 and test-only follow-up de02e0f against main.

Two independent read-only subagents reviewed the milestone:

- Rules: all 18 admissions, linked-return timing and generations, simultaneous
  entry events, tokens, central zone/SBA boundaries, graveyard exile, target
  predicates and funding interactions.
- Library: owned exile links, generations and token reuse, public schema 13
  projections, completed zone batches, snapshot replay, mana/source-cost
  interactions and compiler admission.

Both reported no actionable findings. Both passed 166 Rust tests, 32 script tests
and catalogue consistency. The library reviewer also rebuilt release, passed all
19 Python tests and completed 20 native games. Neither reviewer edited files or
posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and release integration. One hundred supported Standard games
completed naturally: 124,111 decisions, 1,952 target choices, 101,586 decisions
with exiled objects and 88,479 decisions with return links. Native fixtures cover
early departure/reentry, simultaneous mutual entry triggers, source SBA death,
token behavior before/after cessation, stale held objects, predicates,
graveyards, public projections and replay. Python verifies public links and fresh
returned incarnations through clone/restore.

Coverage is 364/5,164 Standard identities, six regression cards and 29 token
factories. Delayed/face-down/re-exile semantics, play/cast permissions, wider
links/targets/replacements, remaining cards/rules and full native RL/search
acceptance remain outstanding. These reviews do not certify full Standard
conformance.
