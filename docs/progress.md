# Full-goal progress

Goal: an efficient, in-process Rust library for **all current Standard-legal
cards**, enforcing gameplay legality and supporting complete state branching and
restoration for search in a neural-network RL pipeline. Partial card sets do not
satisfy this goal. The active target is pinned to October 15, 2026.

| Requirement | Current evidence | Remaining work |
| --- | --- | --- |
| Full legal identity pool | 5,164 frozen oracle identities, Java reconciliation and native catalogue tests | Maintain reviewed data versions; implement missing Java-source cards directly |
| All card semantics | Coverage manifest and runtime admission agree: 80 executable Standard identities | 5,084 identities; many reusable mechanics plus card-specific rules |
| Gameplay enforcement | Implemented timing, payment, targets, stack, source-aware damage, strike steps, 11 keywords, typed programs, trigger ordering and regression tests | Wider events/targeted triggers, replacement/prevention, layers, counters, tokens, modes/costs, remaining keywords and mechanics |
| In-process library | Rust rlib and C ABI/cdylib; Python ctypes tests including complete games | Batched/packed training interface and model hooks |
| Branch/restore | Complete owned-state copies, local RNG, reusable snapshots, replay/invariant tests | Cover every new continuation/mechanic; search over information sets |
| Fast simulation | Reproducible initial Freeform benchmark and compact state | Profile full Standard decks as mechanics expand, including inference/encoding |
| Fair observations | Hidden-hand/library projection and privacy tests; catalogue fingerprint | New private choices/zones; belief sampling and neural encoders |
| Reviewed delivery | PR #6 foundation; PR #8 catalogue/templates; PR #10 damage/combat; PR #12 object identity/removal, with two subagent reviews each | Continue issue-managed, reviewed PRs through complete acceptance |

The latest work adds immutable typed effect programs, owned resolving continuations,
real triggered-ability stack entries, APNAP/controller ordering, source context,
self-entry/death/attack and controller-filtered entry watchers, draw/discard/life
effects, and optional draw/discard decisions. Discard selections stay private until
committed. State-based actions wait for complete resolution, including suspended
choices. Exact templates add 33 executable Standard cards (80 total).

The next shared dependencies include targeted ability announcements, reusable
cost/mode choices, tokens, counters and wider event hooks. Damage/life events,
replacement/prevention, continuous layers and the remaining card-specific mechanics
still need full implementations. These priorities do not reduce the all-Standard goal.

Completion remains unproven and the goal remains active. Final acceptance must
audit every supported card/required mechanic, illegal-action paths, every pending
choice's restore/replay, hidden information, native/Python integration and full
workload performance. A zero unsupported-card counter alone will not prove rules
correctness.
