# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 279 executable Standard identities | 4,885 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 11 keywords, typed programs, trigger ordering and regression tests | Wider events/multiple target groups, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Complete owned-state copies, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal; PR #13 programs/triggers; PR #15 targeting; PR #17 counters; PR #18 fixed tokens; PR #20 activations; PR #22 mana/artifact tokens; PR #23 payment windows; PR #25 scry/surveil; PR #27 additive stats, with two subagent reviews each | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work derives additive power/toughness effects from a compact live-source
index and owned temporary deltas. Static anthems/lords follow battlefield changes;
resolving mass bonuses lock their affected set. Combat, lethal assignment,
observations, SBAs, cleanup and simultaneous last-known snapshots share the same
stats. Forty more complete Standard definitions bring coverage to 279/5,164,
with 28 referenced token factories. Full Standard acceptance remains outstanding.

Other dependencies include wider costs/modes/events, multiple distinct target
groups, triggered/restricted/variable mana, infinite-resource handling and
replacement/prevention/layer machinery.
Remaining card-specific mechanics and native RL/search integration still need full
implementations. These priorities do not reduce the all-Standard goal.

Completion remains unproven and the goal remains active. Final acceptance must
audit every supported card/required mechanic, illegal-action paths, every pending
choice's restore/replay, hidden information, native/Python integration and full
workload performance. A zero unsupported-card counter alone will not prove rules
correctness.
