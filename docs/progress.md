# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 1,791 executable Standard identities | 3,373 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 15 keywords, typed programs, trigger ordering and regression tests | Wider events/cross-group relationships, replacement/prevention, layers, wider counter mechanics, broader token abilities/copies, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Isolated COW object/library buffers, owned continuations, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding; optimize atomic checkpoint overhead |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | Job Select PR #246, Warp PR #247, actual-payment PR #251 and replacement PR #252 have two independent nonauthor reviews and durable scoped validation records; [replacement review](review-pr-252.md). Earlier milestones remain documented in their feature/review pages. | Continue issue-managed, reviewed PRs through complete acceptance |

This dependent branch adds three complete [single-graveyard bodies](single-graveyard.md)
to standalone Omen parent `72fb6b2`: Feral Deathgorger // Dusk Sight, Arashin
Sunshield and Griffnaut Tracker. Branch coverage is 1,791/5,164, with 3,373 gated,
85 token recipes, one emblem, nineteen Adventure helpers, ten Omen helpers and
schema 78. It has not yet incorporated the separately reviewed Saga/Ward main
composition. Integration with that current parent and two independent final-head
reviews remain required before merge.

All 1,646 native tests pass on the unchanged generated programs, as do all-target
Clippy and the full 351-method compiler/data run before final compiler-only
reminder refinements. Final seven focused compiler/provenance methods and five
Omen compiler methods pass; exact generated comparison verifies that those later
refinements change only provenance fingerprint, preserving every definition and
helper. Twenty paid public cases pass on the earlier strict immutable, followed
by eight fresh final-fingerprint paid cases covering every body in both seats.
The [feature record](single-graveyard.md) gives exact library identities, counts,
scopes and limitations. These Freeform fixture decks do not constitute a new
Standard100 acceptance claim.

All remaining Adventure, Omen, Saga, Ward, ownership/control and other card/rules
semantics remain required by the full goal. The current increment does not
establish complete Standard rules, full-deck performance or an RL training loop.

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
