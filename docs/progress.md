# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 212 executable Standard identities | 4,952 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 11 keywords, typed programs, trigger ordering and regression tests | Wider events/multiple target groups, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Complete owned-state copies, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal; PR #13 programs/triggers; PR #15 targeting; PR #17 counters; PR #18 fixed tokens; PR #20 activations; PR #22 mana/artifact tokens, with two subagent reviews each | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work adds six-slot mana, distinct colorless costs, immediate activated
production and owned resolution-time mana choices. Predefined artifact tokens
carry their actual abilities. Forty-one new definitions bring coverage to
212/5,164, with 27 referenced token factories counted separately. Native and
Python checks cover immediate mana, normal Food/Clue responses, exact production
legality, source sacrifice, usage limits and replay.

Issue #21 remains open for mana abilities during casting/payment. Current
prefunding does not represent every legal payment sequence. Other dependencies
include wider costs/modes/events, multiple distinct target groups, triggered
mana abilities, spending restrictions and replacement/prevention/layer machinery.
Remaining card-specific mechanics and native RL/search integration still need full
implementations. These priorities do not reduce the all-Standard goal.

Completion remains unproven and the goal remains active. Final acceptance must
audit every supported card/required mechanic, illegal-action paths, every pending
choice's restore/replay, hidden information, native/Python integration and full
workload performance. A zero unsupported-card counter alone will not prove rules
correctness.
