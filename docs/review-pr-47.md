# Independent review: step triggers and intervening-if conditions

PR: https://github.com/altf4/rustmage/pull/47
Implementation reviewed: aaa3d5d against main.

Two independent read-only subagents reviewed the milestone:

- Rules: step-entry timing, event/resolution condition checks, current versus
  last-known tapped source state, incarnations, raid history, all 17 admissions
  and the new token factory, optional targets and complete-text gating.
- Library: step dispatch and forced-action ordering, source-index maintenance,
  public schema 19 history, owned snapshots/continuations, accounting, predicate
  resolution and native/Python integration.

Both reported no actionable findings. Both independently passed 245 Rust tests,
44 script tests and catalogue consistency. The library reviewer also passed all
26 release Python tests and 20 native games with 216 observed step triggers and
2,770 cloned/replayed boundaries or choices. Neither reviewer edited files,
mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the complete Rust/script/Python suites. One hundred supported
Standard games completed naturally: 43,011 decisions, 1,028 observed step triggers,
13,283 cloned/replayed boundaries or choices and 637 committed attack turns.
Native fixtures cover upkeep/draw sequencing, late entrants, optional targets,
changing intervening conditions, competing end-step triggers, current derived
power and land counts, survival source departure/return and announcement LKI,
raid declaration/history/reset, stun and token creation before state-based loss.
A full admitted-card fixture checks index registration, removal and restoration.

Coverage is 599/5,164 Standard identities, six regression cards and 35 token
factories. Delayed triggers, wider history, additional phases, broader modes/costs,
remaining card/rules semantics and full native RL/search integration remain
outstanding. Some exact predicate/event vocabulary has no fully admitted card
until other mechanics are implemented. These reviews do not certify full Standard
conformance or establish an XMage performance comparison.
