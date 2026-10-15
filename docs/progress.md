# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 47 executable Standard identities | 5,117 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 11 keywords and regression tests | General events, triggers, replacement/prevention, layers, counters, tokens, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Complete owned-state copies, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal, with two subagent reviews each | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work centralizes zone transitions, retains owned last-known object
views, tracks generation-aware targets and damage sources, and adds Unsummon,
Fell, Murder and Bilbo’s Deadly Slice. Tests exercise returned/recast objects,
combat removal, library membership, target invalidation and native/Python replay.

The next rules work is reusable immutable effect programs and trigger processing.
Zone transitions return owned events for that work but do not yet dispatch triggers.
Source attribution and suspended choices must remain in snapshot state. Inventory
frequencies show enters-the-battlefield triggers, tokens, counters and draw effects
as major dependencies. These are implementation priorities, not reductions of the
all-Standard goal.

Completion remains unproven and the goal remains active. Final acceptance must
audit every supported card/required mechanic, illegal-action paths, every pending
choice's restore/replay, hidden information, native/Python integration and full
workload performance. A zero unsupported-card counter alone will not prove rules
correctness.
